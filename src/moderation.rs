//! Bans, the whitelist, timeouts and operators: one TOML file each in
//! PersistentStorage/Server/Moderation, a list of players known by their address (an
//! IPv4 or IPv6 range, `ipv4_cidr`), device id (udid) and/or nickname. An empty key matches
//! nobody. The lists are kept in memory
//! and a list's file is written again after every change to it: they are short, and a
//! server operator can read and edit them by hand while the server is off.

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::net::IpAddr;
use std::path::Path;
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

const FOLDER: &str = "PersistentStorage/Server/Moderation";
const BLACKLIST_FILE: &str = "Blacklist.toml";
const WHITELIST_FILE: &str = "Whitelist.toml";
const TIMEOUTS_FILE: &str = "Timeouts.toml";
const OPERATORS_FILE: &str = "Operators.toml";

/// The operator a new server starts with: whoever runs it on this computer.
const LOCALHOST: &str = "127.0.0.1/32";
const LOCALHOST_LEVEL: u8 = 3;

/// A banned or whitelisted player.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct BlacklistEntry {
    /// An address range, "a.b.c.d/prefix" or "a:b::c/prefix"; a bare address is its
    /// own /32 or /128. The key keeps its old name: renamed, every list written before
    /// would lose its addresses without a word.
    pub ipv4_cidr: String,
    pub udid: String,
    pub nickname: String,
    pub note: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct TimeoutEntry {
    /// An address range, "a.b.c.d/prefix" or "a:b::c/prefix"; a bare address is its
    /// own /32 or /128. The key keeps its old name: renamed, every list written before
    /// would lose its addresses without a word.
    pub ipv4_cidr: String,
    pub udid: String,
    pub nickname: String,
    pub note: String,
    /// When it ends, in seconds since 1970 (UTC).
    pub expires_at: i64,
    /// The short cooldown of a player who just left (leave_cooldown_set): kept in memory
    /// only, so a disconnect writes no file. What the file has is never one.
    #[serde(skip)]
    pub leave_cooldown: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct OpEntry {
    /// An address range, "a.b.c.d/prefix" or "a:b::c/prefix"; a bare address is its
    /// own /32 or /128. The key keeps its old name: renamed, every list written before
    /// would lose its addresses without a word.
    pub ipv4_cidr: String,
    pub udid: String,
    pub nickname: String,
    pub note: String,
    pub level: u8,
}

/// The keys a list entry knows its player by.
trait Player {
    fn keys(&self) -> (&str, &str, &str);
}

impl Player for BlacklistEntry {
    fn keys(&self) -> (&str, &str, &str) {
        (&self.ipv4_cidr, &self.udid, &self.nickname)
    }
}

impl Player for TimeoutEntry {
    fn keys(&self) -> (&str, &str, &str) {
        (&self.ipv4_cidr, &self.udid, &self.nickname)
    }
}

impl Player for OpEntry {
    fn keys(&self) -> (&str, &str, &str) {
        (&self.ipv4_cidr, &self.udid, &self.nickname)
    }
}

/// Whether the entry is this player by any key they share. Pass an empty nickname to
/// match by device and address only.
fn is_player(entry: &impl Player, ip: &str, udid: &str, nickname: &str) -> bool {
    let (entry_cidr, entry_udid, entry_nickname) = entry.keys();
    let same = |key: &str, wanted: &str| !key.is_empty() && key == wanted;
    let in_range = !entry_cidr.is_empty() && !ip.is_empty() && covers(entry_cidr, ip);
    in_range || same(entry_udid, udid) || same(entry_nickname, nickname)
}

/// An address range as the lists write it, "address/prefix": the address as one
/// number, the bits its kind has (32 for IPv4, 128 for IPv6) and the prefix length.
/// A bare address is a range of its own. An IPv4 address written the IPv6 way
/// (::ffff:a.b.c.d) is read as the IPv4 one, so both forms are one player.
fn parse_cidr(text: &str) -> Option<(u128, u32, u32)> {
    let (address, prefix) = text.split_once('/').map_or((text, None), |(address, prefix)| (address, Some(prefix)));
    let (address, bits) = match address.trim().parse::<IpAddr>().ok()?.to_canonical() {
        IpAddr::V4(v4) => (u128::from(u32::from(v4)), 32),
        IpAddr::V6(v6) => (u128::from(v6), 128),
    };
    let prefix = match prefix {
        None => bits,
        Some(prefix) => prefix.trim().parse().ok().filter(|&prefix| prefix <= bits)?,
    };
    Some((address, bits, prefix))
}

/// The first `prefix` bits of an address that has `bits` bits: what a range and its
/// addresses share.
fn network(address: u128, bits: u32, prefix: u32) -> u128 {
    if prefix == 0 { 0 } else { address & (u128::MAX << (bits - prefix)) }
}

/// Whether an entry's range takes in `wanted`, an address or a range of its own, of the
/// same kind (an IPv4 range never takes in an IPv6 address). What is no address at all
/// is compared as written.
fn covers(entry: &str, wanted: &str) -> bool {
    match (parse_cidr(entry), parse_cidr(wanted)) {
        (Some((entry_address, entry_bits, entry_prefix)), Some((wanted_address, wanted_bits, wanted_prefix))) => {
            entry_bits == wanted_bits
                && entry_prefix <= wanted_prefix
                && network(entry_address, entry_bits, entry_prefix) == network(wanted_address, entry_bits, entry_prefix)
        }
        _ => entry == wanted,
    }
}

/// An address or range as the lists write it: with its prefix, /32 or /128 for one address.
fn cidr_text(text: &str) -> String {
    match parse_cidr(text) {
        Some((_, bits, _)) if !text.contains('/') => format!("{text}/{bits}"),
        _ => text.to_string(),
    }
}

/// Takes the player's entries out of a list. Returns whether there were any.
fn remove_player<T: Player>(list: &mut Vec<T>, ip: &str, udid: &str, nickname: &str) -> bool {
    let before = list.len();
    list.retain(|entry| !is_player(entry, ip, udid, nickname));
    list.len() != before
}

/// How a list is written in its file: `[[entry]]` tables, one a player.
#[derive(Serialize, Deserialize)]
struct ListFile<T> {
    #[serde(default = "Vec::new")]
    entry: Vec<T>,
}

#[derive(Default)]
struct Lists {
    blacklist: Vec<BlacklistEntry>,
    whitelist: Vec<BlacklistEntry>,
    timeouts: Vec<TimeoutEntry>,
    ops: Vec<OpEntry>,
}

static LISTS: OnceLock<Mutex<Lists>> = OnceLock::new();

/// The lists, recovering from a poisoned lock instead of panicking: a worker thread
/// that panicked while holding it must not take every other worker's moderation down
/// with it, and the lists themselves are still whole. (`shared.peers` is handled the
/// same way in worker.rs.)
fn lists() -> MutexGuard<'static, Lists> {
    LISTS.get().expect("moderation::init_moderation() not called").lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Reads a list, or None when its file does not exist yet. A file that cannot be read
/// stops the server: carrying on with an empty list would write over the operator's.
fn load_list<T: DeserializeOwned>(file: &str) -> Option<Vec<T>> {
    let path = Path::new(FOLDER).join(file);
    let text = std::fs::read_to_string(&path).ok()?;
    let list: ListFile<T> = toml::from_str(&text).unwrap_or_else(|error| panic!("{} is broken: {error}", path.display()));
    Some(list.entry)
}

/// Writes the list beside its file and then puts it in the file's place: a write cut
/// short (a crash, the power going) leaves the old list whole, where load_list would
/// otherwise find half a file and stop the server.
fn save_list<T: Serialize + Clone>(file: &str, list: &[T]) {
    let path = Path::new(FOLDER).join(file);
    let unfinished = path.with_extension("toml.tmp");
    let text = toml::to_string(&ListFile { entry: list.to_vec() }).expect("moderation lists always serialize");
    if let Err(error) = std::fs::write(&unfinished, text).and_then(|_| std::fs::rename(&unfinished, &path)) {
        log::error!("Failed to save {}: {}", path.display(), error);
    }
}

pub fn init_moderation() {
    if LISTS.get().is_some() {
        return;
    }
    std::fs::create_dir_all(FOLDER).expect("failed to create the moderation folder");
    // The localhost operator is given only to a server without an operators file: if
    // it was removed from the file on purpose, it is not put back.
    let ops = load_list(OPERATORS_FILE).unwrap_or_else(|| {
        let localhost = vec![OpEntry { ipv4_cidr: LOCALHOST.to_string(), note: "localhost".to_string(), level: LOCALHOST_LEVEL, ..OpEntry::default() }];
        save_list(OPERATORS_FILE, &localhost);
        localhost
    });
    let lists = Lists {
        blacklist: load_list(BLACKLIST_FILE).unwrap_or_default(),
        whitelist: load_list(WHITELIST_FILE).unwrap_or_default(),
        timeouts: load_list(TIMEOUTS_FILE).unwrap_or_default(),
        ops,
    };
    LISTS.set(Mutex::new(lists)).ok();
}

/// Bans a player by the keys Config.toml bans by, and takes them off the whitelist
/// and the operators.
pub fn ban_add(nickname: &str, udid: &str, ip: &str, note: &str) -> bool {
    let rules = &crate::config::cfg().states.lobby_misc.moderation;
    let mut lists = lists();
    remove_player(&mut lists.blacklist, ip, udid, nickname);
    let entry = BlacklistEntry {
        ipv4_cidr: cidr_text(asked_if(rules.ban_ip, ip)),
        udid: asked_if(rules.ban_udid, udid).to_string(),
        nickname: asked_if(rules.ban_nickname, nickname).to_string(),
        note: note.to_string(),
    };
    if entry.ipv4_cidr.is_empty() && entry.udid.is_empty() && entry.nickname.is_empty() {
        save_list(BLACKLIST_FILE, &lists.blacklist);
        return true;
    }
    lists.blacklist.push(entry);
    remove_player(&mut lists.whitelist, ip, udid, nickname);
    remove_player(&mut lists.ops, ip, udid, nickname);
    save_list(BLACKLIST_FILE, &lists.blacklist);
    save_list(WHITELIST_FILE, &lists.whitelist);
    save_list(OPERATORS_FILE, &lists.ops);
    true
}

pub fn ban_ip_only(ip: &str, note: &str) -> bool {
    let mut lists = lists();
    remove_player(&mut lists.blacklist, ip, "", "");
    lists.blacklist.push(BlacklistEntry { ipv4_cidr: cidr_text(ip), note: note.to_string(), ..BlacklistEntry::default() });
    remove_player(&mut lists.whitelist, ip, "", "");
    remove_player(&mut lists.ops, ip, "", "");
    save_list(BLACKLIST_FILE, &lists.blacklist);
    save_list(WHITELIST_FILE, &lists.whitelist);
    save_list(OPERATORS_FILE, &lists.ops);
    true
}

pub fn ban_revoke_full(nickname: &str, udid: &str, ip: &str) -> bool {
    let mut lists = lists();
    let removed = remove_player(&mut lists.blacklist, ip, udid, nickname);
    save_list(BLACKLIST_FILE, &lists.blacklist);
    removed
}

pub fn ban_revoke_by_ip(ip: &str) -> bool {
    ban_revoke_full("", "", ip)
}

/// Whether the player is banned by any key Config.toml bans by.
pub fn ban_check(nickname: &str, udid: &str, ip: &str) -> bool {
    let rules = &crate::config::cfg().states.lobby_misc.moderation;
    let (ip, udid, nickname) = (asked_if(rules.ban_ip, ip), asked_if(rules.ban_udid, udid), asked_if(rules.ban_nickname, nickname));
    lists().blacklist.iter().any(|entry| is_player(entry, ip, udid, nickname))
}

/// A key that is banned by, and looked for, only when Config.toml bans by it (an empty
/// key matches nobody).
fn asked_if(ban_by: bool, key: &str) -> &str {
    if ban_by { key } else { "" }
}

/// Newest first.
pub fn list_blacklist() -> Vec<BlacklistEntry> {
    lists().blacklist.iter().rev().cloned().collect()
}

pub fn whitelist_add_full(nickname: &str, udid: &str, ip: &str, note: &str) -> bool {
    let mut lists = lists();
    remove_player(&mut lists.whitelist, ip, udid, nickname);
    lists.whitelist.push(BlacklistEntry { ipv4_cidr: cidr_text(ip), udid: udid.to_string(), nickname: nickname.to_string(), note: note.to_string() });
    save_list(WHITELIST_FILE, &lists.whitelist);
    true
}

pub fn whitelist_revoke(ip: &str) -> bool {
    whitelist_revoke_full("", "", ip)
}

pub fn whitelist_revoke_full(nickname: &str, udid: &str, ip: &str) -> bool {
    let mut lists = lists();
    let removed = remove_player(&mut lists.whitelist, ip, udid, nickname);
    save_list(WHITELIST_FILE, &lists.whitelist);
    removed
}

/// Matched only by udid and ip, never by nickname - see op_check's doc
/// comment. Matching on nickname would let anyone bypass the whitelist gate by
/// connecting under a whitelisted player's unauthenticated display name.
pub fn whitelist_check(udid: &str, ip: &str) -> bool {
    lists().whitelist.iter().any(|entry| is_player(entry, ip, udid, ""))
}

/// A timeout a vote or an operator gave: it outlives a restart of the server.
pub fn timeout_set(nickname: &str, udid: &str, ip: &str, expires_at: u64, note: &str) -> bool {
    put_timeout(nickname, udid, ip, expires_at, note, false);
    true
}

/// The reconnect cooldown of a player who just left (worker.rs): seconds long, so it
/// is not worth a write of the file at every disconnect.
pub fn leave_cooldown_set(nickname: &str, udid: &str, ip: &str, expires_at: u64) {
    put_timeout(nickname, udid, ip, expires_at, nickname, true);
}

fn put_timeout(nickname: &str, udid: &str, ip: &str, expires_at: u64, note: &str, leave_cooldown: bool) {
    let mut lists = lists();
    remove_player(&mut lists.timeouts, ip, udid, nickname);
    let entry = TimeoutEntry { ipv4_cidr: cidr_text(ip), udid: udid.to_string(), nickname: nickname.to_string(), note: note.to_string(), expires_at: expires_at as i64, leave_cooldown };
    lists.timeouts.push(entry);
    if !leave_cooldown {
        save_timeouts(&lists.timeouts);
    }
}

/// The timeouts as their file keeps them: without the cooldowns of players who left.
fn save_timeouts(timeouts: &[TimeoutEntry]) {
    let kept: Vec<TimeoutEntry> = timeouts.iter().filter(|entry| !entry.leave_cooldown).cloned().collect();
    save_list(TIMEOUTS_FILE, &kept);
}

/// How many of the timeouts the file keeps, to tell whether taking some off changed it.
fn kept_count(timeouts: &[TimeoutEntry]) -> usize {
    timeouts.iter().filter(|entry| !entry.leave_cooldown).count()
}

pub fn timeout_revoke(udid: &str, ip: &str) -> bool {
    let mut lists = lists();
    let removed = remove_player(&mut lists.timeouts, ip, udid, "");
    save_timeouts(&lists.timeouts);
    removed
}

/// When the player's timeout ends, if they have one that has not ended yet. An ended
/// one is taken off the list.
pub fn timeout_check(udid: &str, ip: &str) -> Option<u64> {
    let mut lists = lists();
    let ends = lists.timeouts.iter().filter(|entry| is_player(*entry, ip, udid, "")).map(|entry| entry.expires_at).max()?;
    if ends > now_unix_i64() {
        return Some(ends as u64);
    }
    let kept_before = kept_count(&lists.timeouts);
    remove_player(&mut lists.timeouts, ip, udid, "");
    if kept_count(&lists.timeouts) != kept_before {
        save_timeouts(&lists.timeouts);
    }
    None
}

pub fn cleanup_expired_timeouts() {
    let now = now_unix_i64();
    let mut lists = lists();
    let kept_before = kept_count(&lists.timeouts);
    lists.timeouts.retain(|entry| entry.expires_at > now);
    if kept_count(&lists.timeouts) != kept_before {
        save_timeouts(&lists.timeouts);
    }
}

/// The timeouts that have not ended, the soonest to end first.
pub fn list_timeouts() -> Vec<TimeoutEntry> {
    let now = now_unix_i64();
    let mut active: Vec<TimeoutEntry> = lists().timeouts.iter().filter(|entry| entry.expires_at > now).cloned().collect();
    active.sort_by_key(|entry| entry.expires_at);
    active
}

/// Makes the player an operator of Config.toml's default level. An operator already
/// known by the player's udid or ip is updated instead (see op_check for why never by
/// nickname).
pub fn op_add(nickname: &str, udid: &str, ip: &str, note: &str) -> bool {
    let level = crate::config::cfg().states.lobby_misc.moderation.op_default_level;
    op_set_full(nickname, udid, ip, level, note)
}

/// Sets (or, with `level == 0`, removes) the operator with this ip.
pub fn op_set(ip: &str, level: u8, note: &str) -> bool {
    op_set_full("", "", ip, level, note)
}

/// Set (or, with `level == 0`, remove) an operator anchored on udid+ip.
///
/// Privilege is keyed on udid/ip only; `nickname` is stored for display and
/// never used for matching, since the nickname is unauthenticated (see op_check).
pub fn op_set_full(nickname: &str, udid: &str, ip: &str, level: u8, note: &str) -> bool {
    let mut lists = lists();
    if level == 0 {
        let removed = remove_player(&mut lists.ops, ip, udid, "");
        save_list(OPERATORS_FILE, &lists.ops);
        return removed;
    }
    let mut known = false;
    for entry in lists.ops.iter_mut().filter(|entry| is_player(*entry, ip, udid, "")) {
        known = true;
        entry.level = level;
        entry.note = note.to_string();
        if !nickname.is_empty() {
            entry.nickname = nickname.to_string();
        }
    }
    if !known {
        lists.ops.push(OpEntry { ipv4_cidr: cidr_text(ip), udid: udid.to_string(), nickname: nickname.to_string(), note: note.to_string(), level });
    }
    save_list(OPERATORS_FILE, &lists.ops);
    true
}

/// Resolve a connecting player's operator level.
///
/// Operator status is matched **only by udid and ip**, never by nickname.
/// The nickname is an unauthenticated, publicly-visible string from the identity
/// packet - matching on it would let anyone inherit an operator's level simply by
/// connecting under that operator's display name (privilege escalation, including
/// op>=2 which bypasses ip_validation and grants ban/kick). udid is per-device and
/// not publicly displayed, so udid+ip is the trust anchor. The default localhost op
/// (ip 127.0.0.1) is unaffected.
pub fn op_check(udid: &str, ip: &str) -> u8 {
    lists().ops.iter().filter(|entry| is_player(*entry, ip, udid, "")).map(|entry| entry.level).max().unwrap_or(0)
}

/// The highest level first, then by ip.
pub fn list_ops() -> Vec<OpEntry> {
    let mut ops = lists().ops.clone();
    ops.sort_by(|a, b| b.level.cmp(&a.level).then_with(|| a.ipv4_cidr.cmp(&b.ipv4_cidr)));
    ops
}

fn now_unix_i64() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since_epoch| since_epoch.as_secs())
        .unwrap_or(0) as i64
}

pub fn now_unix() -> u64 {
    now_unix_i64() as u64
}

/// The lists of a new server, without reading or writing their files: the tests run
/// in the project folder, whose PersistentStorage is not theirs to write.
#[cfg(test)]
pub fn init_new_server_moderation() {
    let ops = vec![OpEntry { ipv4_cidr: LOCALHOST.to_string(), note: "localhost".to_string(), level: LOCALHOST_LEVEL, ..OpEntry::default() }];
    LISTS.set(Mutex::new(Lists { blacklist: Vec::new(), whitelist: Vec::new(), timeouts: Vec::new(), ops })).ok();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_player_is_matched_by_any_key_they_share_but_never_by_an_empty_one() {
        let entry = BlacklistEntry { ipv4_cidr: "1.2.3.4".to_string(), nickname: "fox".to_string(), ..BlacklistEntry::default() };
        assert!(is_player(&entry, "1.2.3.4", "", ""));
        assert!(is_player(&entry, "", "", "fox"));
        assert!(!is_player(&entry, "", "", ""), "an empty udid matches no empty udid");
        assert!(!is_player(&entry, "5.6.7.8", "some-device", ""), "by device and address only");
    }

    #[test]
    fn a_list_survives_its_file() {
        let ops = vec![OpEntry { ipv4_cidr: LOCALHOST.to_string(), note: "localhost".to_string(), level: LOCALHOST_LEVEL, ..OpEntry::default() }];
        let text = toml::to_string(&ListFile { entry: ops.clone() }).unwrap();
        assert!(text.contains("[[entry]]") && text.contains("level = 3"), "{text}");
        let read: ListFile<OpEntry> = toml::from_str(&text).unwrap();
        assert_eq!(read.entry, ops);
        let empty: ListFile<TimeoutEntry> = toml::from_str("").unwrap();
        assert!(empty.entry.is_empty(), "an empty file is an empty list");
    }

    /// A disconnect's cooldown stays out of Timeouts.toml, and a timeout read back from
    /// the file is a kept one, written again with the next change.
    #[test]
    fn only_the_timeouts_of_votes_and_operators_reach_the_file() {
        let kick = TimeoutEntry { udid: "kicked".to_string(), expires_at: 1, ..TimeoutEntry::default() };
        let cooldown = TimeoutEntry { udid: "left".to_string(), expires_at: 1, leave_cooldown: true, ..TimeoutEntry::default() };
        assert_eq!(kept_count(&[kick.clone(), cooldown]), 1);
        let text = toml::to_string(&ListFile { entry: vec![kick] }).unwrap();
        assert!(!text.contains("leave_cooldown"), "{text}");
        let read: ListFile<TimeoutEntry> = toml::from_str(&text).unwrap();
        assert_eq!(kept_count(&read.entry), 1);
    }

    #[test]
    fn ipv6_ranges_work_like_ipv4_ones_and_never_mix_with_them() {
        assert!(covers("2001:db8::/32", "2001:db8:1::5"));
        assert!(!covers("2001:db8::/32", "2001:db9::5"));
        assert!(covers("2001:db8::/32", "2001:db8:ff00::/40"), "a narrower range inside it");
        assert!(covers("::/0", "::1"), "every IPv6 address");
        assert!(!covers("0.0.0.0/0", "::1"), "an IPv4 range takes in no IPv6 address");
        assert!(covers("127.0.0.1/32", "::ffff:127.0.0.1"), "an IPv4 player reaching a server on ::");
        assert_eq!(cidr_text("2001:db8::5"), "2001:db8::5/128");
        assert_eq!(cidr_text("1.2.3.4"), "1.2.3.4/32");
        assert_eq!(cidr_text("not an address"), "not an address");
    }

    #[test]
    fn a_range_takes_in_its_addresses_and_narrower_ranges() {
        assert!(covers("10.0.0.0/8", "10.20.30.40"));
        assert!(covers("10.0.0.0/8", "10.1.0.0/16"), "a narrower range inside it");
        assert!(!covers("10.1.0.0/16", "10.0.0.0/8"), "not a wider one");
        assert!(covers("1.2.3.4", "1.2.3.4") && !covers("1.2.3.4", "1.2.3.5"), "a bare address is /32");
        assert!(covers("0.0.0.0/0", "8.8.8.8"), "/0 is everyone");
        assert!(covers("::1", "::1") && !covers("::1", "127.0.0.1"), "IPv6 is compared as written");
        assert_eq!(cidr_text("1.2.3.4"), "1.2.3.4/32");
        assert_eq!(cidr_text("1.2.0.0/16"), "1.2.0.0/16");
        assert_eq!(cidr_text(""), "");
    }
}
