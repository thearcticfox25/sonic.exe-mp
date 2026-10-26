//! The host's ?gui window: the game started by the server's terminal (terminal
//! commands.rs open_gui) to watch its own lobby. It connects like a player, with a key
//! only this server run has, and from then on is no player at all: it is not among
//! `server.peers`, so no list, vote, count or round has it, and nobody is told it is
//! there. It is sent what a spectator is (the running round, states::spectate) and,
//! when it asks, who is connected. What it does to a player it does not send here: it
//! hands the terminal a terminal command (?select-player), as if the host typed it.

use crate::packet::{Packet, PacketType};
use crate::server::{GameState, OutboxMsg, Server};
use std::sync::OnceLock;

/// Bytes of the key, written out as hex.
const KEY_BYTES: usize = 16;

/// The key this server run gives its ?gui windows on their command line.
pub fn key() -> &'static str {
    static KEY: OnceLock<String> = OnceLock::new();
    KEY.get_or_init(|| (0..KEY_BYTES).map(|_| format!("{:02x}", rand::random::<u8>())).collect())
}

/// Whether a connection that said `key` in its IDENTITY is the host's own ?gui window.
pub fn is_host_window(key_said: &str, ip: &str) -> bool {
    let from_this_machine = ip.parse::<std::net::IpAddr>().is_ok_and(|address| address.is_loopback());
    key_said == key() && from_this_machine
}

/// The window has connected: it waits as the waiting room does, watches the round at
/// once if one is running, and is told who is here.
pub fn join(observer_id: u16, server: &mut Server, outbox: &mut Vec<OutboxMsg>) {
    server.observers.push(observer_id);
    log::info!("{} the host's window is watching (id {})", crate::server::lobby_tag(server.id), observer_id);
    let response = crate::connection::identity_response(false, observer_id, 0);
    outbox.push(OutboxMsg::SendTo(observer_id, response.data().to_vec(), true));
    crate::states::lobby::send_lobby_player_list(observer_id, server, outbox);
    if server.state == GameState::Game && server.game.round.is_some() {
        crate::states::spectate::send_round(observer_id, server, outbox);
    }
}

/// A connection went away. Returns whether it was a ?gui window, which leaves nothing
/// behind to tell anyone about.
pub fn leave(id: u16, server: &mut Server) -> bool {
    let was_window = server.observers.contains(&id);
    server.observers.retain(|&observer| observer != id);
    was_window
}

/// The only thing a ?gui window asks the server for: who is connected now.
pub fn handle(observer_id: u16, packet: &mut Packet, server: &mut Server, outbox: &mut Vec<OutboxMsg>) {
    if packet.packet_type() == Some(PacketType::LOBBY_PLAYER) {
        crate::states::lobby::send_lobby_player_list(observer_id, server, outbox);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::PeerData;

    #[test]
    fn only_the_key_from_this_machine_opens_the_host_window() {
        assert!(is_host_window(key(), "127.0.0.1"));
        assert!(!is_host_window(key(), "10.0.0.2"), "the key leaked to another machine");
        assert!(!is_host_window("", "127.0.0.1"), "a player who sends no key");
    }

    /// The window is no player: it is in no list of players, and it is told about
    /// everyone connected, the waiting room included.
    #[test]
    fn the_host_window_sees_everyone_and_nobody_sees_it() {
        crate::config::set_test_config();
        let mut server = Server::new(0);
        let mut playing = PeerData::new(1, String::from("10.0.0.1"));
        playing.in_game = true;
        server.peers.push(playing);
        server.peers.push(PeerData::new(2, String::from("10.0.0.2")));

        let mut outbox = Vec::new();
        join(9, &mut server, &mut outbox);
        assert_eq!(server.peers.len(), 2, "the window is not among the players");
        let listed: Vec<u16> = outbox
            .iter()
            .filter_map(|message| match message {
                OutboxMsg::SendTo(9, data, _) if data.get(1) == Some(&(PacketType::LOBBY_PLAYER as u8)) => Some(u16::from_le_bytes([data[2], data[3]])),
                _ => None,
            })
            .collect();
        assert_eq!(listed, vec![1, 2], "the one in the lobby and the one waiting");

        assert!(leave(9, &mut server));
        assert!(server.observers.is_empty());
        assert!(!leave(1, &mut server), "a player is not the window");
    }
}
