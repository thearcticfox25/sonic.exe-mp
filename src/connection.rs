use std::collections::{HashMap, HashSet};

use rand::RngExt;

use crate::config::cfg;
use crate::local_link::{HOST_PLAYER, HOST_SLOT};
use crate::packet::{Packet as GamePacket, PacketType};
use crate::server::{
    apply_outbox, DisconnectReason, GameState, OutboxMsg, PeerData, PendingPeer, Server, Transport,
};

/// Everything the worker knows about connections, as opposed to about the round.
///
/// ENet addresses a peer by its slot index, the game protocol addresses it by an
/// id; these maps translate between the two. They always travel together, so they
/// are passed around as one value instead of four.
pub struct PeerRegistry {
    /// Connected, but not identified yet: no nickname or UDID seen so far.
    pub pending: HashMap<usize, PendingPeer>,
    /// IPs and UDIDs currently in use, for the one-connection-per-address rule.
    pub taken_addresses: HashSet<String>,
    pub game_id_of_slot: HashMap<usize, u16>,
    pub slot_of_game_id: HashMap<u16, usize>,
    /// The id given last. ENet hands a freed slot to the next connection at once, so an
    /// id is not the slot's: a newcomer would inherit what the round, a vote or the
    /// results still keep under the id of the player who left.
    last_id: u16,
}

impl PeerRegistry {
    pub fn new() -> Self {
        Self {
            pending:         HashMap::new(),
            taken_addresses: HashSet::new(),
            game_id_of_slot: HashMap::new(),
            slot_of_game_id: HashMap::new(),
            last_id: HOST_PLAYER,
        }
    }

    /// An id no connection of this lobby has now: the host's game is always player 0,
    /// everyone else gets the next one.
    // ponytail: after 65535 connections the ids come round again; a lobby that sees that
    // many in one round would need ids wider than u16 in the protocol.
    pub fn new_id(&mut self, idx: usize) -> u16 {
        if idx == HOST_SLOT {
            return HOST_PLAYER;
        }
        loop {
            self.last_id = self.last_id.wrapping_add(1);
            if self.last_id != HOST_PLAYER && !self.slot_of_game_id.contains_key(&self.last_id) {
                return self.last_id;
            }
        }
    }

    /// Registers an identified player, so packets from ENet slot `idx` can be
    /// routed to game id `game_id` and back.
    pub fn accept(&mut self, idx: usize, game_id: u16, ip: &str, udid: &str) {
        self.register(idx, game_id);
        self.taken_addresses.insert(ip.to_string());
        self.taken_addresses.insert(udid.to_string());
    }

    /// The connection becomes `game_id`, without taking its address: the host's ?gui
    /// window shares the host's own address with the game the host may be playing.
    pub fn register(&mut self, idx: usize, game_id: u16) {
        self.pending.remove(&idx);
        self.game_id_of_slot.insert(idx, game_id);
        self.slot_of_game_id.insert(game_id, idx);
    }
}

/// Drops a connection that was never accepted into the lobby.
fn reject(
    idx: usize,
    net: &mut Transport,
    peers: &mut PeerRegistry,
    reason: DisconnectReason,
) {
    disconnect_idx(idx, net, reason as u32);
    peers.pending.remove(&idx);
}

/// SERVER_IDENTITY_RESPONSE: whether the player enters the lobby now (or waits
/// for the round to end), their player id, and the rules of picking characters:
/// whether players may share a character, how many EXE a round has, and whether
/// this player may pick ahead in the lobby (lobby_preselection, by `op_level`).
pub fn identity_response(in_lobby: bool, player_id: u16, op_level: u8) -> GamePacket {
    let selection = &cfg().states.lobby_misc.character_selection;
    let mut response = GamePacket::new(PacketType::SERVER_IDENTITY_RESPONSE);
    let _ = response.write_u8(u8::from(in_lobby));
    let _ = response.write_u16(player_id);
    let _ = response.write_u8(u8::from(selection.charselect_mod_unlocked));
    let _ = response.write_u8(selection.exe_count);
    let _ = response.write_u8(u8::from(selection.preselects(op_level)));
    response
}

pub fn disconnect_idx(idx: usize, net: &mut Transport, reason: u32) {
    log::info!("Disconnected idx {} (reason {})", idx, reason);
    net.disconnect(idx, reason);
}

/// Handles the IDENTITY packet: the client's one chance to say who it is.
/// Everything that can turn a connection away (wrong version, ban, timeout, full
/// lobby, duplicate address) is decided here, and only then does the connection
/// become a `PeerData` in `server.peers`.
pub fn handle_identity(
    idx: usize,
    data: &[u8],
    net: &mut Transport,
    server: &mut Server,
    peers: &mut PeerRegistry,
    all_shared: &[std::sync::Arc<crate::server::ServerShared>],
    base_port: u16,
) {
    let cfg = cfg();
    let mut pkt = GamePacket::from_data(data);
    pkt.pos = 2;

    let version = match pkt.read_char() {
        Some(found) => found,
        None => { log::debug!("Identity failed for idx {}: truncated or unreadable version", idx); return; }
    };
    let _server_index = pkt.read_i32();
    let nickname = match pkt.read_str() {
        Some(text) => text,
        None => { log::debug!("Identity failed for idx {}: truncated nickname", idx); return; }
    };
    let udid = match pkt.read_str() {
        Some(text) => text,
        None => { log::debug!("Identity failed for idx {}: truncated udid", idx); return; }
    };
    let lobby_icon = pkt.read_u8().unwrap_or(0);
    let pet = pkt.read_u8().unwrap_or(0);

    let Some(ip) = peers.pending.get(&idx).map(|pending| pending.ip.clone()) else { return };
    // Only the host's own ?gui window knows the key, and it connects from this machine.
    let observer_key = pkt.read_str().unwrap_or_default();
    if crate::states::host_gui::is_host_window(&observer_key, &ip) {
        let game_id = peers.new_id(idx);
        peers.register(idx, game_id);
        let mut outbox: Vec<OutboxMsg> = Vec::new();
        crate::states::host_gui::join(game_id, server, &mut outbox);
        apply_outbox(net, peers, &mut outbox);
        return;
    }

    // Limit by Unicode characters, not bytes - matches the upstream "30 characters
    // max" rule (C's string_length counts codepoints) so multi-byte nicks aren't
    // rejected early. read_str already caps the raw field at 128 bytes.
    if crate::packet::str_unicode_len(&nickname) >= 30 {
        reject(idx, net, peers, DisconnectReason::Other);
        return;
    }

    if !cfg.server_config.pairing.versioning.disable_version_validating
        && version != cfg.server_config.pairing.versioning.target_version
    {
        reject(idx, net, peers, DisconnectReason::VersionMismatch);
        return;
    }

    if let Some(ts) = crate::moderation::timeout_check(&udid, &ip) {
        if ts > crate::moderation::now_unix() {
            log::info!("{} is rate-limited (id {}, ip {})", crate::colors::colorize(&nickname), idx, ip);
            reject(idx, net, peers, DisconnectReason::RateLimited);
            return;
        }
    }

    let max = cfg.server_config.pairing.maximum_players_per_lobby as usize;
    if server.total_count() >= max {
        // Before rejecting, check sibling lobbies in this same process for
        // room and redirect there instead. No disconnect on this path --
        // just tell the client where to go and leave the pending connection
        // be; it's cleaned up by the ordinary pending-timeout if the client
        // doesn't follow.
        for (sibling_id, sibling) in all_shared.iter().enumerate() {
            if sibling_id as u16 == server.id { continue; }
            let count = sibling.peers.read().map(|guard| guard.len()).unwrap_or(usize::MAX);
            if count < max {
                let target_port = base_port + sibling_id as u16;
                let mut redirect = GamePacket::new(PacketType::SERVER_LOBBY_CHANGELOBBY);
                let _ = redirect.write_u32(target_port as u32);
                net.send(idx, redirect.data(), true);
                log::debug!("Redirecting idx {} to another free server: {}", idx, sibling_id);
                return;
            }
        }
        reject(idx, net, peers, DisconnectReason::LobbyFull);
        return;
    }

    let op_level = crate::moderation::op_check(&udid, &ip);

    if op_level < 2
        && cfg.server_config.pairing.ip_validation
        && (peers.taken_addresses.contains(&ip) || peers.taken_addresses.contains(&udid))
    {
        reject(idx, net, peers, DisconnectReason::IpInUse);
        return;
    }

    if crate::moderation::ban_check(&nickname, &udid, &ip) {
        log::info!("{} banned by host (id {}, ip {})", crate::colors::colorize(&nickname), idx, ip);
        reject(idx, net, peers, DisconnectReason::BannedByHost);
        return;
    }

    if cfg.states.lobby_misc.moderation.enforce_whitelist
        && !crate::moderation::whitelist_check(&udid, &ip)
    {
        reject(idx, net, peers, DisconnectReason::BannedByHost);
        return;
    }

    let game_id = peers.new_id(idx);
    peers.accept(idx, game_id, &ip, &udid);

    let mut peer = PeerData::new(game_id, ip);
    peer.nickname = nickname;
    peer.udid = udid;
    peer.lobby_icon = lobby_icon;
    peer.pet = pet;
    peer.verified = true;
    peer.op = op_level;
    peer.should_timeout = true;

    peer.in_game = server.state == GameState::Lobby;

    peer.exe_chance = 1 + rand::rng().random_range(0u8..4u8);

    server.peers.push(peer);

    let resp = identity_response(server.state == GameState::Lobby, game_id, op_level);
    net.send(idx, resp.data(), true);

    let last_peer = server.peers.last();
    log::info!(
        "{} Player '{}' joined (id={})",
        crate::server::lobby_tag(server.id),
        crate::colors::colorize(last_peer.map(|peer| peer.nickname.as_str()).unwrap_or("?")),
        game_id,
    );
    log::info!("  IP: {}", last_peer.map(|peer| peer.ip.as_str()).unwrap_or("?"));
    log::info!("  UID: {}", last_peer.map(|peer| peer.udid.as_str()).unwrap_or("?"));

    let mut outbox: Vec<OutboxMsg> = Vec::new();
    crate::states::state_join(game_id, server, &mut outbox);
    apply_outbox(net, peers, &mut outbox);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ENet gives a freed slot to the next connection: that one must not get the id of
    /// the player who left, which the round may still know.
    #[test]
    fn a_freed_slot_gets_a_new_id_and_the_host_stays_player_zero() {
        let mut peers = PeerRegistry::new();
        let first = peers.new_id(0);
        peers.register(0, first);
        peers.game_id_of_slot.remove(&0);
        peers.slot_of_game_id.remove(&first);
        let second = peers.new_id(0);
        assert_ne!(first, second);
        assert_ne!(second, HOST_PLAYER);
        assert_eq!(peers.new_id(HOST_SLOT), HOST_PLAYER);
        // When the ids come round, 0 (the host's game, and the server's own chat and
        // console) is skipped.
        peers.last_id = u16::MAX;
        assert_eq!(peers.new_id(0), 1);
    }
}
