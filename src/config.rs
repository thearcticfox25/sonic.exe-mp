use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

pub static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn cfg() -> &'static Config {
    CONFIG.get().expect("Config not initialized")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub server_config: ServerConfig,
    pub states: StatesConfig,
    pub miscellaneous: MiscellaneousConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    pub networking: Networking,
    pub pairing: Pairing,
    pub window: Window,
}

/// What the server's own window (its terminal or console) shows and takes.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Window {
    pub logging: Logging,
    pub terminal: Terminal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Terminal {
    /// Whether the window takes commands; without, it only shows the log.
    pub enable: bool,
}

impl Default for Terminal {
    fn default() -> Self {
        // A phone has no keyboard to type commands on, nor a terminal to type them in.
        Self { enable: !cfg!(target_os = "android") }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Networking {
    /// The address of this computer the lobbies listen on: 0.0.0.0 for every IPv4 one.
    pub listen_address: std::net::IpAddr,
    pub port: u16,
    pub server_count: u16,
    /// Simulation steps a second. The game runs at the same speed whatever this
    /// is: a tick is worth less movement the more of them there are.
    pub tick_rate: u32,
    /// Snapshots a second each client is sent; the round is still simulated and
    /// judged at tick_rate. Taken as a whole number of ticks between snapshots.
    pub snapshot_rate: u32,
    /// Every this many snapshots a client gets the whole state (a keyframe); the ones
    /// in between carry only what changed since it. 1: every snapshot whole.
    pub keyframe_interval: u32,
    pub vinny: Vinny,
}

/// About the rate the original server passed players on at (a player's data at most
/// every ~43 ms), as a multiple of 12.
const DEFAULT_SNAPSHOT_RATE: u32 = 24;
/// Half a second of snapshots at the default snapshot_rate.
const DEFAULT_KEYFRAME_INTERVAL: u32 = 12;
/// A phone hosting (IServer) works less: the lowest tick_rate the game allows, a
/// snapshot every 6th tick instead of every 3rd, and a keyframe every 1.5 s instead
/// of every 0.5 s.
const ANDROID_TICK_RATE: u32 = crate::core::config::MIN_TICK_RATE;
const ANDROID_SNAPSHOT_RATE: u32 = 8;
const ANDROID_KEYFRAME_INTERVAL: u32 = 12;

impl Networking {
    /// Ticks from one snapshot to the next: tick_rate / snapshot_rate, at least 1.
    pub fn ticks_per_snapshot(&self) -> u32 {
        (self.tick_rate / self.snapshot_rate.max(1)).max(1)
    }
}

/// Per-tick flood cap, raw ENet events/packets. Lives under `networking`
/// since it applies before any game-state processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Vinny {
    pub max_events_per_tick: u32,
    pub max_packets_per_peer_per_tick: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Pairing {
    pub maximum_players_per_lobby: u8,
    pub ip_validation: bool,
    pub ping_limit: u16,
    pub rate_limit_window: u16,
    pub kick_timeout_window: u16,
    pub versioning: Versioning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Versioning {
    pub target_version: char,
    pub disable_version_validating: bool,
}

/// 0: startup banner + basic init report + terminal command replies only
///    (those always show -- an admin needs to see their own command run).
/// 1: level 0 + ERROR logs.
/// 2: level 1 + WARN logs.
/// 3 (default, matches the C version's default verbosity): level 2 + INFO logs.
/// 4: level 3 + DEBUG logs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Logging {
    pub log_level: u8,
    pub log_to_file: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StatesConfig {
    pub lobby_misc: LobbyMisc,
    pub map_selection: MapSelection,
    pub gameplay: Gameplay,
    pub results_misc: ResultsMisc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LobbyMisc {
    pub upper_bracket: String,
    pub hosts_name: String,
    pub lower_bracket: String,
    pub server_location: String,
    pub message_of_the_day: String,
    pub lobby_timeout_timer: u8,
    pub lobby_start_timer: u8,
    /// Strips non-op players of all vote and lobby-start rights. The map and
    /// other lobby decisions are controlled exclusively by operators. (это
    /// самое бесполезное, что я делал...)
    pub authoritarian_mode: bool,
    pub lobby_ready_required_percentage: u8,
    pub kick_unready_before_starting: bool,
    pub anonymous_mode: bool,
    pub min_players_required: u8,
    pub moderation: Moderation,
    pub votekick: Votekick,
    pub character_selection: CharacterSelection,
    pub chat_rate_limit: ChatRateLimit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Moderation {
    pub ban_ip: bool,
    pub ban_udid: bool,
    pub ban_nickname: bool,
    pub op_default_level: u8,
    pub enforce_whitelist: bool,
    pub shy_mode: bool,
    pub aggressive_username_ban: bool,
}

/// Per-peer chat anti-flood (token bucket). A peer may send up to `burst`
/// messages back-to-back, then is limited to `messages_per_second` sustained.
/// Operators at or above `exempt_op_level` bypass the limit entirely.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ChatRateLimit {
    pub enable: bool,
    pub burst: f64,
    pub messages_per_second: f64,
    pub exempt_op_level: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Votekick {
    pub cooldown: u8,
    pub autoban_leavers: bool,
    pub vote_result_delay: u8,
    pub use_legacy_voting: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MapSelection {
    pub enabled: bool,
    pub timer: u8,
    pub map_list: Vec<bool>,
    pub exclude_last_map: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CharacterSelection {
    pub enable: bool,
    pub charselect_mod_unlocked: bool,
    pub exe_count: u8,
    pub charselect_timer: u8,
    pub lobby_preselection: Preselection,
}

/// Who may pick their character ahead in the lobby, on getting ready (the preference
/// cards), before the character select stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Preselection {
    Everyone,
    Operators,
    Nobody,
}

impl CharacterSelection {
    /// Whether a player of this operator level may pick their character ahead.
    pub fn preselects(&self, op_level: u8) -> bool {
        match self.lobby_preselection {
            Preselection::Everyone => true,
            Preselection::Operators => op_level > 0,
            Preselection::Nobody => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Gameplay {
    pub respawn_time: u8,
    pub sudden_death_timer: u8,
    pub ring_appearance_timer: u8,
    pub escape_time: u8,
    pub demonization_percentage: u8,
    pub exe_camp_penalty: bool,
    pub hide_player_characters: bool,
    pub gametimers_ceiling: u16,
    pub ending_timer: u8,
    pub waiting_timeout: u8,
    /// Elimination-tournament progression, off at 0. After the Results of a
    /// round the next round's map vote starts at once, without a Lobby, and the
    /// waiting room is not let in for as long as the tournament still has
    /// another round left, so anyone who joined mid-tournament waits (or, with
    /// allow_spectators, watches) instead of walking into a running bracket.
    /// Without allow_spectators the waiting room also isn't shown the Results
    /// screen for a round that isn't the tournament's last, since it won't be
    /// following the bracket on.
    /// Once elimination would leave too few players to continue (below
    /// lobby_misc.min_players_required), that round is treated as the
    /// tournament's last and the waiting room is let in same as always. After
    /// viewing results, the worst-ranked survivor(s) (by the same ranking used
    /// for the results screen) are kicked before the next round starts. If
    /// every survivor escaped -- no bad performance to rank -- the exe is
    /// eliminated instead.
    ///
    /// 1: kick the single worst player.
    /// 2: kick half the round's participants if that count is even, else fall
    ///    back to mode 1.
    /// 3: same as mode 2 when even; when odd, kick two thirds if the count is
    ///    divisible by 3, else fall back to mode 1.
    pub tournament_mode: u8,
    /// What a player who connects after the lobby sees until the next lobby.
    /// Off: the waiting screen, with its player list, the round clock, the
    /// chat and the minigame, as in the original. On: the round's own screens,
    /// as its players see them -- the map vote (already voted, with no vote of
    /// their own), the character select stage, the round, and the results --
    /// with the waiting music, the chat and the minigame over the first three.
    /// In the round they take no part: they
    /// are not on the roster, cannot be picked as exe, cannot be hurt and
    /// cannot hurt anyone, and nobody in the round is told they are there.
    /// Their camera follows any of the round's players -- survivors, demons
    /// and killers alike, skipping whoever is away in their pause menu -- or
    /// flies over the map on its own, and their chat is at hand while they
    /// watch, where a player's chat is a page of the pause menu.
    pub allow_spectators: bool,
    /// Whether a playing client is only told about what it can see: players
    /// further than the edge of its window (plus a margin for the camera's own
    /// slack) are left out of the snapshots it gets, so a changed client
    /// cannot draw what its player cannot see. What the player list says
    /// about everyone still reaches every client (the killers' side learns
    /// only who stands, is down or is dead), and so do the players the game itself points out with an
    /// arrow wherever they stand (the killer's nearest survivor, a demon's
    /// nearest killer, Eggman's caught victim, an Exeller clone's find).
    /// Someone who is down sees what the survivors still up see, the ones their
    /// camera may watch, or everything while Core/Hurt.toml's dead_watch_everyone
    /// lets them watch anyone; a spectator is never cut. The level's
    /// own objects (rings, the abilities' things) are not cut at all: they are
    /// heard further than they are seen, and cutting them would take those
    /// sounds away.
    pub limit_player_view: bool,
    pub entities_misc: EntitiesMisc,
    pub palette_check: PaletteCheck,
    pub banana: Banana,
    pub gmcycle: GMCycle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EntitiesMisc {
    pub map_specific: MapSpecific,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MapSpecific {
    pub ravine_mist: RavineMistCfg,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RavineMistCfg {
    pub shards: ShardsCfg,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ShardsCfg {
    pub amount: u8,
    pub required_for_exit: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PaletteCheck {
    pub check_skin_colors: bool,
    pub max_skin_colors: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Banana {
    pub disable_timer: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GMCycle {
    pub overhell: bool,
    /// Percentage of non-exe in-game players force-demonized the instant the round
    /// starts, before anyone can be wounded and independent of sudden death. 0
    /// disables it. Rounds to the nearest player count.
    pub ambush_force_demonization_percentage_on_start: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ResultsMisc {
    pub enabled: bool,
    pub timer: u8,
    pub pride: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MiscellaneousConfig {
    pub other: OtherConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OtherConfig {
    pub ignore_inadequate_configuration: bool,
    pub instructor_enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_config: ServerConfig::default(),
            states: StatesConfig::default(),
            miscellaneous: MiscellaneousConfig::default(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            networking: Networking::default(),
            pairing: Pairing::default(),
            window: Window::default(),
        }
    }
}

impl Default for Vinny {
    fn default() -> Self {
        Self {
            max_events_per_tick: 256,
            max_packets_per_peer_per_tick: 16,
        }
    }
}

impl Default for Networking {
    fn default() -> Self {
        let (tick_rate, snapshot_rate, keyframe_interval) = if cfg!(target_os = "android") {
            (ANDROID_TICK_RATE, ANDROID_SNAPSHOT_RATE, ANDROID_KEYFRAME_INTERVAL)
        } else {
            (crate::core::config::DEFAULT_TICKS_PER_SECOND, DEFAULT_SNAPSHOT_RATE, DEFAULT_KEYFRAME_INTERVAL)
        };
        Self { listen_address: std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED), port: crate::packet::DEFAULT_PORT, server_count: 1, tick_rate, snapshot_rate, keyframe_interval, vinny: Vinny::default() }
    }
}

impl Default for Pairing {
    fn default() -> Self {
        Self {
            maximum_players_per_lobby: 7,
            ip_validation: true,
            ping_limit: 250,
            rate_limit_window: 5,
            kick_timeout_window: 60,
            versioning: Versioning::default(),
        }
    }
}

impl Default for Versioning {
    fn default() -> Self {
        Self { target_version: crate::packet::VERSION, disable_version_validating: false }
    }
}

impl Default for Logging {
    fn default() -> Self {
        Self { log_level: 3, log_to_file: false }
    }
}

impl Default for StatesConfig {
    fn default() -> Self {
        Self {
            lobby_misc: LobbyMisc::default(),
            map_selection: MapSelection::default(),
            gameplay: Gameplay::default(),
            results_misc: ResultsMisc::default(),
        }
    }
}

impl Default for LobbyMisc {
    fn default() -> Self {
        Self {
            upper_bracket: "-----\\advanced/server~-----".to_string(),
            hosts_name: "The Arctic Fox".to_string(),
            lower_bracket: "------------------------".to_string(),
            server_location: "Saint Petersburg".to_string(),
            message_of_the_day: "\\mods are disallowed on this server".to_string(),
            lobby_timeout_timer: 25,
            lobby_start_timer: 5,
            authoritarian_mode: false,
            lobby_ready_required_percentage: 100,
            kick_unready_before_starting: false,
            anonymous_mode: false,
            min_players_required: 2,
            moderation: Moderation::default(),
            votekick: Votekick::default(),
            character_selection: CharacterSelection::default(),
            chat_rate_limit: ChatRateLimit::default(),
        }
    }
}

impl Default for ChatRateLimit {
    fn default() -> Self {
        Self {
            enable: true,
            burst: 5.0,
            messages_per_second: 0.7,
            exempt_op_level: 2,
        }
    }
}

impl Default for Moderation {
    fn default() -> Self {
        Self {
            ban_ip: true,
            ban_udid: true,
            ban_nickname: false,
            op_default_level: 3,
            enforce_whitelist: false,
            shy_mode: true,
            aggressive_username_ban: false,
        }
    }
}

impl Default for Votekick {
    fn default() -> Self {
        Self { cooldown: 30, autoban_leavers: false, vote_result_delay: 3, use_legacy_voting: false }
    }
}

impl Default for MapSelection {
    fn default() -> Self {
        Self {
            enabled: true,
            timer: 30,
            map_list: vec![
                false, true, true, true, true, true, true, true, true, true,
                true, true, true, true, true, true, true, true, true, false,
                true, true,
            ],
            exclude_last_map: true,
        }
    }
}

impl Default for CharacterSelection {
    fn default() -> Self {
        Self {
            enable: true,
            charselect_mod_unlocked: false,
            exe_count: 1,
            charselect_timer: 30,
            lobby_preselection: Preselection::Everyone,
        }
    }
}

impl Default for Gameplay {
    fn default() -> Self {
        Self {
            respawn_time: 30,
            sudden_death_timer: 120,
            ring_appearance_timer: 60,
            escape_time: 50,
            demonization_percentage: 50,
            exe_camp_penalty: true,
            hide_player_characters: false,
            gametimers_ceiling: 570,
            ending_timer: 5,
            waiting_timeout: 15,
            tournament_mode: 0,
            allow_spectators: false,
            limit_player_view: false,
            entities_misc: EntitiesMisc::default(),
            palette_check: PaletteCheck::default(),
            banana: Banana::default(),
            gmcycle: GMCycle::default(),
        }
    }
}

impl Default for EntitiesMisc {
    fn default() -> Self {
        Self { map_specific: MapSpecific::default() }
    }
}

impl Default for MapSpecific {
    fn default() -> Self {
        Self { ravine_mist: RavineMistCfg::default() }
    }
}

impl Default for RavineMistCfg {
    fn default() -> Self {
        Self { shards: ShardsCfg::default() }
    }
}

impl Default for ShardsCfg {
    fn default() -> Self {
        Self { amount: 7, required_for_exit: 6 }
    }
}

impl Default for PaletteCheck {
    fn default() -> Self {
        Self {
            check_skin_colors: false,
            max_skin_colors: 32,
        }
    }
}

impl Default for Banana {
    fn default() -> Self {
        Self { disable_timer: false }
    }
}

impl Default for GMCycle {
    fn default() -> Self {
        Self { overhell: false, ambush_force_demonization_percentage_on_start: 0 }
    }
}

impl Default for ResultsMisc {
    fn default() -> Self {
        Self { enabled: true, timer: 15, pride: true }
    }
}

impl Default for MiscellaneousConfig {
    fn default() -> Self {
        Self { other: OtherConfig { ignore_inadequate_configuration: false, instructor_enabled: true } }
    }
}

impl Default for OtherConfig {
    fn default() -> Self {
        Self { ignore_inadequate_configuration: false, instructor_enabled: true }
    }
}

/// (TOML section path, field name) -> comment text to emit above that field when
/// writing a fresh default Config.toml, mirroring the doc comment on the matching
/// struct/field above. `field: None` places the comment above the `[section]`
/// header itself instead of a specific key. Keep in sync by hand when adding or
/// changing a doc comment above -- a stale or missing entry here just means the
/// generated file has no comment for that field, not a broken config.
const CONFIG_TOML_COMMENTS: &[(&str, Option<&str>, &str)] = &[
    // server_config.networking
    ("server_config.networking", Some("listen_address"),
        "The address of this computer the server listens on. 0.0.0.0 takes\nplayers on every IPv4 address it has; 127.0.0.1 only from this computer\nitself; the address of one network card only from that network. An\nIPv6 address works as well: :: takes players on every IPv6 address and,\nas on most systems, every IPv4 one too, but it cannot be opened where IPv6\nis switched off (ipv6.disable=1). One address, not a range: who may join\nis decided by the moderation lists."),
    ("server_config.networking", Some("port"),
        "Changes the port the server listens on (7811 by default, which players\nreach without naming it). Players reach another port by writing it after\n\":\" in the address ([2001:db8::1]:port for IPv6)."),
    ("server_config.networking", Some("server_count"),
        "Number of sub-servers (lobbies). If a lobby is full, players are\nredirected to another one; if none are available, they receive a\n\"Server Full\" error."),
    ("server_config.networking", Some("tick_rate"),
        "How many times a second the server simulates the round and sends what it\nsimulated. The game runs at the same speed whatever this is -- a tick is\nworth proportionally less movement -- but a higher rate sees a hit closer\nto when it happened, the way leagues ran CS:GO at 128 instead of 64. The\nprice is that much more work and that many more packets per player. 300\nis the ceiling, 48 the floor. 72 by default, 48 on Android, where a phone\nhosts."),
    ("server_config.networking", Some("snapshot_rate"),
        "How many times a second each client is sent what the server simulated.\nThe round is still simulated and judged tick_rate times a second; only\nthe sending is rarer. The original server passed players on about 23\ntimes a second. Fewer snapshots: less traffic and less work per player,\nbut moving things that are not players (lifts, snowballs, lava, shots)\nmove in visible steps; players stay smooth. Equal to tick_rate sends\nevery tick. Taken as a whole number of ticks between snapshots, so 24\nat a tick_rate of 72 is every 3rd tick. 24 by default, 8 on Android (every\n6th tick, at 48)."),
    ("server_config.networking", Some("keyframe_interval"),
        "Every this many snapshots a client is sent the whole state, like a\nvideo's keyframe; the snapshots in between carry only what changed since\nit, which is a fraction of the size. The keyframe is sent reliably, so\none that is lost is sent again before the changes after it are used.\nHigher: less traffic, but the changes grow the longer ago the keyframe\nwas. 1: every snapshot whole. 12 at a snapshot_rate of 24 is one\nkeyframe every half second (the default); 12 at 8 is one every 1.5 s (the\ndefault on Android)."),
    ("server_config.networking.vinny", None,
        "Per-tick flood cap, raw ENet events/packets. Lives under `networking`\nsince it applies before any game-state processing."),
    ("server_config.networking.vinny", Some("max_events_per_tick"),
        "Maximum raw ENet events (connect/receive/disconnect) drained per tick,\nacross all peers combined."),
    ("server_config.networking.vinny", Some("max_packets_per_peer_per_tick"),
        "Maximum packets processed from a single peer per tick; that peer's\nremaining packets are dropped for the rest of the tick."),

    // server_config.pairing
    ("server_config.pairing", Some("maximum_players_per_lobby"),
        "Maximum players allowed in a single lobby."),
    ("server_config.pairing", Some("ip_validation"),
        "Prevents players without op (admin) from connecting with two clients\nsharing the same IP address."),
    ("server_config.pairing", Some("ping_limit"),
        "The server periodically calculates each player's rolling average ping.\nIf it exceeds this value (ms), the player is kicked."),
    ("server_config.pairing", Some("rate_limit_window"),
        "If a player reconnects too quickly, their connection is throttled for\nthis many seconds."),
    ("server_config.pairing", Some("kick_timeout_window"),
        "Default kick duration (seconds). Also used as the timeout applied by\n.vk (votekick)."),
    ("server_config.pairing.versioning", Some("target_version"),
        "Client version required to connect: one character, sent as its Unicode\ncode point. This project's game sends \u{3b1}."),
    ("server_config.pairing.versioning", Some("disable_version_validating"),
        "If true, version checking is bypassed and all clients are allowed to\nconnect."),

    // server_config.window
    ("server_config.window.terminal", Some("enable"),
        "If true, the server's window takes terminal commands (?help) on a prompt\nbelow the log. If false, it only shows the log, as AdvancedServer-stable's\ndid, and the server is stopped with Ctrl+C. False by default on Android."),
    ("server_config.window.logging", Some("log_level"),
        "0: startup banner + basic init report + terminal command replies only\n   (those always show -- an admin needs to see their own command run).\n1: level 0 + ERROR logs.\n2: level 1 + WARN logs.\n3 (default, matches the C version's default verbosity): level 2 + INFO logs.\n4: level 3 + DEBUG logs."),
    ("server_config.window.logging", Some("log_to_file"),
        "Saves log output to a file on disk."),

    // states.lobby_misc
    ("states.lobby_misc", Some("upper_bracket"),
        "Decorative line framing the top of the welcome message shown when a\nplayer connects -- an \"opening bracket,\" in the style old games used to\nframe their connection banners."),
    ("states.lobby_misc", Some("server_location"),
        "Display-only label shown in the welcome window as \"%server_location%,\nmax ping: %max_acceptable_ping%ms\", helping players decide if the server\nsuits them by region and latency. Has no effect on server logic."),
    ("states.lobby_misc", Some("hosts_name"),
        "The name displayed for the host."),
    ("states.lobby_misc", Some("lower_bracket"),
        "Decorative line framing the bottom of the welcome message -- the\nmatching \"closing bracket\" to `upper_bracket`."),
    ("states.lobby_misc", Some("message_of_the_day"),
        "Custom message shown after the welcome splash."),
    ("states.lobby_misc", Some("lobby_timeout_timer"),
        "Time (seconds) a non-op player can remain in the lobby with an unready\nstatus without taking any action before being auto-kicked for inactivity."),
    ("states.lobby_misc", Some("lobby_start_timer"),
        "Countdown duration (seconds) once the required percentage of players\nis ready."),
    ("states.lobby_misc", Some("authoritarian_mode"),
        "Strips non-op players of all vote and lobby-start rights. The map and\nother lobby decisions are controlled exclusively by operators. (это самое\nбесполезное, что я делал...)"),
    ("states.lobby_misc", Some("lobby_ready_required_percentage"),
        "Percentage of ready players required to trigger the game-start countdown."),
    ("states.lobby_misc", Some("kick_unready_before_starting"),
        "If true, kicks unready players when the countdown expires. Only\nrelevant when `lobby_ready_required_percentage` is below 100. If false,\nthey watch the round instead of playing it."),
    ("states.lobby_misc", Some("anonymous_mode"),
        "Replaces all player usernames with \"anonymous\". Does not apply to\nmoderators."),
    ("states.lobby_misc", Some("min_players_required"),
        "Minimum number of ready players (not counting those ready with the\nfreecam card) for the game-start countdown, on top of\n`lobby_ready_required_percentage`. Whoever is not ready when the round\nstarts, or chose the freecam card, watches it instead of playing it."),

    ("states.lobby_misc.moderation", Some("ban_ip"),
        "Enables IP-based banning."),
    ("states.lobby_misc.moderation", Some("ban_udid"),
        "Enables UDID-based banning."),
    ("states.lobby_misc.moderation", Some("ban_nickname"),
        "Enables nickname-based banning."),
    ("states.lobby_misc.moderation", Some("op_default_level"),
        "Default operator level granted by the ?op command. The in-lobby .op\ncommand always assigns this default level."),
    ("states.lobby_misc.moderation", Some("enforce_whitelist"),
        "Inverts the banned/allowed player logic. Players are checked against a\nseparate whitelist table and must be present in it to connect, rather\nthan absent from the ban list."),
    ("states.lobby_misc.moderation", Some("shy_mode"),
        "Does not notify a player when their admin status is revoked -- they\nwill discover it after attempting a command or on their next reconnect.\nGranting op still sends a notification."),
    ("states.lobby_misc.moderation", Some("aggressive_username_ban"),
        "Leftover from early testing. When enabled, a nickname ban also chains\nto nicknames indirectly associated with the banned player through prior\nname reuse. Left disabled by default: since nicknames aren't unique and\nany player can take another player's name, this could turn moderation\ninto a joke -- or into a \"terminator\" that bans unrelated players in a\nchain reaction from a single flagged nickname. May also no longer\nfunction as intended after later changes to the ban system; treat as\nunsupported/experimental."),

    ("states.lobby_misc.votekick", Some("cooldown"),
        "Duration of the votekick vote (seconds)."),
    ("states.lobby_misc.votekick", Some("autoban_leavers"),
        "If true, a player who disconnects while a votekick is active against\nthem is treated as a deserter and the kick is counted as successful,\nrather than cancelling the vote. Still only applies the normal timed\nkick (kick_timeout_window), not a permanent ban."),
    ("states.lobby_misc.votekick", Some("vote_result_delay"),
        "Delay (seconds) between vote completion and execution of the result,\ngiving players time to read the outcome."),
    ("states.lobby_misc.votekick", Some("use_legacy_voting"),
        "Switches .votekick and .practice to the older voting: a votekick passes\nearly only when everyone but its target agrees (.n takes a yes back) or\nafter 15 seconds, and a practice vote needs every player."),

    ("states.lobby_misc.chat_rate_limit", Some("enable"),
        "Enables chat rate limiting -- an anti-spam / chat throttling mechanism.\nUnverified by the developer; treat its behavior as unconfirmed until\nyou've checked it in practice."),
    ("states.lobby_misc.chat_rate_limit", Some("burst"),
        "Number of messages a player can send in a quick burst before\nthrottling kicks in."),
    ("states.lobby_misc.chat_rate_limit", Some("messages_per_second"),
        "Sustained message rate (messages/sec) a player is limited to once\ntheir burst allowance is used up."),
    ("states.lobby_misc.chat_rate_limit", Some("exempt_op_level"),
        "Operators at or above this level are exempt from the chat rate limit."),

    ("states.lobby_misc.character_selection", Some("enable"),
        "If true, after the map vote everyone picks their character on the\ncharacter select stage. If false, the stage is skipped: everyone gets\nwhat they picked ahead in the lobby (lobby_preselection), or a random\ncharacter."),
    ("states.lobby_misc.character_selection", Some("charselect_mod_unlocked"),
        "Lets any number of players be the same character. While false, a\ncharacter someone already has, picked ahead or on the character select\nstage, is refused to others (and crossed out on their icon; EXE\ncharacters only when exe_count is more than 1), and the server refuses to\nstart if maximum_players_per_lobby is more than the distinct characters\n(6 survivors and exe_count EXE)."),
    ("states.lobby_misc.character_selection", Some("charselect_timer"),
        "Seconds the character select stage lasts. Whoever has not picked by\nthen is kicked for being away; with lobby_preselection most players\nhave their character before the stage even starts."),
    ("states.lobby_misc.character_selection", Some("lobby_preselection"),
        "Who may pick their character ahead, in the lobby, on getting ready:\n\"everyone\", \"operators\" or \"nobody\". A character picked ahead is\ngiven at once when the character select stage starts, if it is free.\nThe survivor card also offers \"freecam\", to watch the round instead."),
    ("states.lobby_misc.character_selection", Some("exe_count"),
        "How many players are EXE in a round, leaving at least one survivor.\nWhile charselect_mod_unlocked is false every EXE plays a different EXE\ncharacter, so at most 4. The round rules know only the first EXE."),

    // states.map_selection
    ("states.map_selection", Some("enabled"),
        "If false, a random map from the enabled pool is chosen instantly and\nthe round starts with it, bypassing Map Vote."),
    ("states.map_selection", Some("timer"),
        "Duration of the map voting phase (seconds)."),
    ("states.map_selection", Some("map_list"),
        "Map pool. Entries correspond to maps in their in-game index order:\n0 Level 1 (the cut level of the 12.2022 alpha) -- disabled by default,\n1 Hide 'n Seek 2, 2 Ravine Mist, 3 ..., 4 Desert Town, 5 You Can't Run,\n6 Limb City, 7 Not Perfect, 8 Kind & Fair, 9 Act 9, 10 Nasty Paradise,\n11 Priceless Freedom, 12 Volcano Valley, 13 Hill, 14 Majin Forest,\n15 Hide 'n Seek / Angel Island, 16 Torture Cave, 17 Dark Tower,\n18 Haunting Dream, 19 Practice Test Map (Codename: Fart Zone) -- disabled\nby default, 20 Mystic Wood (Codename: Weed Zone), 21 Echidna Ruins\n(Codename: Marijuana)."),
    ("states.map_selection", Some("exclude_last_map"),
        "Removes the previously played map from the vote pool."),

    // states.gameplay
    ("states.gameplay", Some("respawn_time"),
        "Time (seconds) teammates have to revive a killed player."),
    ("states.gameplay", Some("sudden_death_timer"),
        "Time (seconds) after which players can no longer be revived. Note: the\ngame client always displays the Sudden Death warning at 2 minutes -- if\nyou change this value, inform players via motd or server banner."),
    ("states.gameplay", Some("ring_appearance_timer"),
        "Time (seconds) at which the exit ring silhouette appears on the map."),
    ("states.gameplay", Some("escape_time"),
        "Time (seconds) at which the actual exit ring appears. This is the\nwindow players have to jump in and win."),
    ("states.gameplay", Some("demonization_percentage"),
        "Percentage of players who become EXE pawns on death; beyond this\nthreshold, subsequent deaths are permanent."),
    ("states.gameplay", Some("exe_camp_penalty"),
        "If true, pauses the respawn timer while Sonic.EXE is camping a dying\nplayer's body."),
    ("states.gameplay", Some("hide_player_characters"),
        "Replaces all player characters with the Placeholder character."),
    ("states.gameplay", Some("gametimers_ceiling"),
        "Maximum round duration (seconds). The game adds extra time per\nadditional player -- this cap prevents excessively long matches with\nlarge player counts."),
    ("states.gameplay", Some("ending_timer"),
        "Duration (seconds) of the end-of-game cutscene before the Results\nstage begins."),
    ("states.gameplay", Some("waiting_timeout"),
        "Time (seconds) the server waits for clients to finish loading the game\nscene before kicking those who failed to prepare in time."),
    ("states.gameplay", Some("tournament_mode"),
        "Elimination-tournament progression, off at 0. After the Results of a\nround the next round's map vote starts at once, without a Lobby, and the\nwaiting room is not let in for as long as the tournament still has\nanother round left, so anyone who joined mid-tournament waits (or, with\nallow_spectators, watches) instead of walking into a running bracket.\nWithout allow_spectators the waiting room also isn't shown the Results\nscreen for a round that isn't the tournament's last, since it won't be\nfollowing the bracket on. Once\nelimination would leave too few players to continue (below\nlobby_misc.min_players_required), that round is treated as the\ntournament's last and the waiting room is let in same as always. After\nviewing results, the worst-ranked survivor(s) (by the same ranking used\nfor the results screen) are kicked before the next round starts. If\nevery survivor escaped -- no bad performance to rank -- the exe is\neliminated instead.\n\n1: kick the single worst player.\n2: kick half the round's participants if that count is even, else fall\n   back to mode 1.\n3: same as mode 2 when even; when odd, kick two thirds if the count is\n   divisible by 3, else fall back to mode 1."),
    ("states.gameplay", Some("allow_spectators"),
        "What a player who connects after the lobby sees until the next lobby.\nOff: the waiting screen, with its player list, the round clock, the\nchat and the minigame, as in the original. On: the round's own screens,\nas its players see them -- the map vote (already voted, with no vote of\ntheir own), the character select stage, the round, and the results --\nwith the waiting music, the chat and the minigame over the first three.\nIn the round they take no part: they\nare not on the roster, cannot be picked as exe, cannot be hurt and\ncannot hurt anyone, and nobody in the round is told they are there.\nTheir camera follows any of the round's players -- survivors, demons\nand killers alike, skipping whoever is away in their pause menu -- or\nflies over the map on its own, and their chat is at hand while they\nwatch, where a player's chat is a page of the pause menu."),
    ("states.gameplay", Some("limit_player_view"),
        "Whether a playing client is only told about what it can see: players\nfurther than the edge of its window (plus a margin for the camera's own\nslack) are left out of the snapshots it gets, so a changed client\ncannot draw what its player cannot see. What the player list says\nabout everyone still reaches every client (the killers' side learns\nonly who stands, is down or is dead), and so do the players the game itself points out with an\narrow wherever they stand (the killer's nearest survivor, a demon's\nnearest killer, Eggman's caught victim, an Exeller clone's find).\nSomeone who is down sees what the survivors still up see, the ones their\ncamera may watch, or everything while Core/Hurt.toml's dead_watch_everyone\nlets them watch anyone; a spectator is never cut. The level's\nown objects (rings, the abilities' things) are not cut at all: they are\nheard further than they are seen, and cutting them would take those\nsounds away."),

    // states.gameplay.entities_misc
    ("states.gameplay.entities_misc.map_specific.ravine_mist.shards", Some("amount"),
        "Total number of shards that spawn."),
    ("states.gameplay.entities_misc.map_specific.ravine_mist.shards", Some("required_for_exit"),
        "Number of shards survivors must collect to open the exit."),

    // states.gameplay.palette_check
    ("states.gameplay.palette_check", Some("check_skin_colors"),
        "Rejects color palettes that are not among the game's skins. A skin is\ncolors of red, green and blue, each 0..255; one with a channel more\nthan 10 away from every skin of the game gets its player disconnected.\nA custom skin is checked by the colors it starts from, not by its hue."),
    ("states.gameplay.palette_check", Some("max_skin_colors"),
        "The most colors a player's skin or pet may have. A skin with more is\nnot passed on to the other players. The biggest skin of the game has 18;\na skin packet cannot carry more than 63."),


    // states.gameplay.banana / gmcycle
    ("states.gameplay.banana", Some("disable_timer"),
        "Reimplementation of the No Timer Mod Server by Banana 7800."),
    ("states.gameplay.gmcycle", Some("overhell"),
        "Reimplementation of the OverHell game mode from the Gamemode Cycle\nmod by IcedCoffee & ColdestTea."),
    ("states.gameplay.gmcycle", Some("ambush_force_demonization_percentage_on_start"),
        "Percentage of non-exe in-game players force-demonized the instant the round\nstarts, before anyone can be wounded and independent of sudden death. 0\ndisables it. Rounds to the nearest player count."),

    // states.results_misc
    ("states.results_misc", Some("enabled"),
        "Shows the post-game results screen."),
    ("states.results_misc", Some("timer"),
        "Duration of the results screen (seconds)."),
    ("states.results_misc", Some("pride"),
        "Displays camp tease messages on the results screen."),

    // miscellaneous.other
    ("miscellaneous.other", Some("ignore_inadequate_configuration"),
        "Forces the server to run even with settings that may cause crashes or\nunfinishable games."),
    ("miscellaneous.other", Some("instructor_enabled"),
        "Enables the instructor / tutorial system."),
];

fn comment_prefix(comment: &str) -> String {
    comment.lines()
        .map(|line| if line.is_empty() { "#\n".to_string() } else { format!("# {}\n", line) })
        .collect()
}

fn navigate_table<'a>(root: &'a mut toml_edit::Table, path: &str) -> Option<&'a mut toml_edit::Table> {
    let mut table = root;
    for seg in path.split('.') {
        table = table.get_mut(seg)?.as_table_mut()?;
    }
    Some(table)
}

/// Prepends `comment` to whatever prefix decor is already there (e.g. the blank
/// line toml_edit puts before a new `[section]`), so the existing spacing is kept
/// rather than replaced.
fn prepend_comment(decor: &mut toml_edit::Decor, comment: &str) {
    let existing = decor.prefix().and_then(|raw| raw.as_str()).unwrap_or("");
    decor.set_prefix(format!("{}{}", existing, comment_prefix(comment)));
}

/// Attaches a `#` comment, via toml_edit's own decor API, above the matching
/// `[section]` header or `key = value` line of a freshly-generated default
/// Config.toml, per CONFIG_TOML_COMMENTS.
fn annotate_default_toml(raw: &str) -> String {
    let Ok(mut doc) = raw.parse::<toml_edit::DocumentMut>() else { return raw.to_string(); };

    for (path, field, comment) in CONFIG_TOML_COMMENTS {
        let Some(table) = navigate_table(doc.as_table_mut(), path) else { continue };
        match field {
            None => { prepend_comment(table.decor_mut(), comment); }
            Some(key) => {
                if let Some(mut key_mut) = table.key_mut(*key) {
                    prepend_comment(key_mut.leaf_decor_mut(), comment);
                }
            }
        }
    }

    doc.to_string()
}

pub fn load_config() -> anyhow::Result<Config> {
    const CONFIG_FILE: &str = "PersistentStorage/Server/Config.toml";
    if let Ok(data) = std::fs::read_to_string(CONFIG_FILE) {
        let mut cfg: Config = toml::from_str(&data)?;

        cfg.states.map_selection.map_list.resize(crate::maps::MAP_LIST.len(), false);
        log::debug!("{} loaded.", CONFIG_FILE);
        Ok(cfg)
    } else {
        log::warn!("{} not found, using defaults.", CONFIG_FILE);
        let cfg = Config::default();

        if let Ok(out) = toml::to_string_pretty(&cfg) {
            let written = std::path::Path::new(CONFIG_FILE)
                .parent()
                .map_or(Ok(()), std::fs::create_dir_all)
                .and_then(|_| std::fs::write(CONFIG_FILE, annotate_default_toml(&out)));
            if let Err(error) = written {
                log::warn!("could not write {}: {}", CONFIG_FILE, error);
            }
        }
        Ok(cfg)
    }
}

impl Config {
    pub fn verify(&self) -> bool {
        let gameplay = &self.states.gameplay;
        let ravine_mist = &gameplay.entities_misc.map_specific.ravine_mist;
        if !gameplay.banana.disable_timer && !gameplay.gmcycle.overhell && gameplay.ring_appearance_timer < gameplay.escape_time {
            log::error!("ring_appearance_timer must be >= escape_time when timer is enabled");
            return false;
        }
        if ravine_mist.shards.amount > 12 {
            log::error!("ravine_mist.shards.amount must be <= 12");
            return false;
        }
        if ravine_mist.shards.amount < ravine_mist.shards.required_for_exit {
            log::error!("shards.amount must be >= required_for_exit");
            return false;
        }
        if self.states.lobby_misc.lobby_ready_required_percentage > 100 {
            log::error!("lobby_ready_required_percentage must be <= 100");
            return false;
        }
        if gameplay.gmcycle.ambush_force_demonization_percentage_on_start > 100 {
            log::error!("gmcycle.ambush_force_demonization_percentage_on_start must be <= 100");
            return false;
        }
        if gameplay.tournament_mode > 3 {
            log::error!("tournament_mode must be 0-3");
            return false;
        }
        let rate_limit = &self.states.lobby_misc.chat_rate_limit;
        if rate_limit.enable && (rate_limit.burst < 1.0 || rate_limit.messages_per_second <= 0.0) {
            log::error!("chat_rate_limit: burst must be >= 1.0 and messages_per_second > 0.0 when enabled");
            return false;
        }
        if self.server_config.networking.server_count == 0 {
            log::error!("networking.server_count must be >= 1");
            return false;
        }
        let tick_rate = self.server_config.networking.tick_rate;
        if !(crate::core::config::MIN_TICK_RATE..=crate::core::config::MAX_TICK_RATE).contains(&tick_rate) {
            log::error!(
                "networking.tick_rate must be between {} and {}",
                crate::core::config::MIN_TICK_RATE,
                crate::core::config::MAX_TICK_RATE
            );
            return false;
        }
        let snapshot_rate = self.server_config.networking.snapshot_rate;
        if snapshot_rate == 0 || snapshot_rate > tick_rate {
            log::error!("networking.snapshot_rate must be between 1 and tick_rate ({})", tick_rate);
            return false;
        }
        if self.server_config.networking.keyframe_interval == 0 {
            log::error!("networking.keyframe_interval must be at least 1");
            return false;
        }
        let selection = &self.states.lobby_misc.character_selection;
        if selection.exe_count == 0 {
            log::error!("states.lobby_misc.character_selection.exe_count must be at least 1");
            return false;
        }
        // Without duplicates every character can be played once: 6 survivors, 4 EXE.
        const SURVIVOR_CHARACTERS: u8 = 6;
        const EXE_CHARACTERS: u8 = 4;
        if !selection.charselect_mod_unlocked {
            if selection.exe_count > EXE_CHARACTERS {
                log::error!("states.lobby_misc.character_selection.exe_count must be <= {} unless charselect_mod_unlocked is true", EXE_CHARACTERS);
                return false;
            }
            let distinct_characters = SURVIVOR_CHARACTERS + selection.exe_count;
            if self.server_config.pairing.maximum_players_per_lobby > distinct_characters {
                log::error!(
                    "maximum_players_per_lobby must be <= {} (6 survivors and exe_count EXE) unless states.lobby_misc.character_selection.charselect_mod_unlocked is true",
                    distinct_characters
                );
                return false;
            }
        }
        if self.server_config.pairing.ping_limit < 3 {
            log::error!("pairing.ping_limit must be >= 3");
            return false;
        }
        true
    }
}

/// The config of every test in the process: CONFIG can be set only once, and tests run
/// together in one process, so they share this one instead of each setting its own.
/// The defaults, with spectating allowed for the tests of states::spectate.
#[cfg(test)]
pub fn set_test_config() {
    let mut config = Config::default();
    config.states.gameplay.allow_spectators = true;
    let _ = CONFIG.set(config);
}
