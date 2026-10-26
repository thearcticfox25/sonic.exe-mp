//! Watching a round instead of playing it: from the lobby with the freecam survivor
//! card, or by not being ready when the round starts (`seat_lobby_watchers`), and,
//! with states.gameplay.allow_spectators, by connecting while a round is on its way
//! (`join_late`). Both are the same state from then on.
//!
//! A spectator never joins the round they are watching. They stay off the roster,
//! out of the exe draw, out of the win conditions and out of the anti-cheat, nobody
//! in the round is told they are there, and everything they send about it is dropped
//! rather than being read as a move in it (`plays_no_part`; their chat is let
//! through, as the waiting room's is). `peer.in_game` stays false the whole time,
//! which is what every other stage already asks to decide who is playing.
//!
//! They are shown the round's screens as its players are (PeerData::follows_round,
//! server::send_to_round): the map vote, without a vote in it, the character select
//! stage, the round itself, and its results. The round is sent to them the way it is
//! sent to a player: who is in it, who is exe and on which map, the gameplay
//! numbers, and then the snapshot of every tick (states::round). Their snapshot has
//! no player of their own in it (`Snapshot::own` is None) and nothing is left out of
//! it, while a player is only told about what is near them
//! (states.gameplay.limit_player_view). Their client sees that it has no player in
//! the round and watches it (client/screens/level).
//!
//! The first Lobby entry that lets the waiting room in ends the spectate and they
//! walk in as an ordinary player (lobby_init); a tournament keeps them watching its
//! next round instead (game_init calls `enter_round` again).

use crate::colors::*;
use crate::config::cfg;
use crate::packet::{Packet, PacketType};
use crate::server::{GameState, OutboxMsg, Server};

/// A player connected while the server is past the lobby. With
/// states.gameplay.allow_spectators they watch the round from wherever it is now,
/// and true is returned; without it they wait on the waiting screen, as in the
/// original, and the caller sends them that.
pub fn join_late(player_id: u16, server: &mut Server, outbox: &mut Vec<OutboxMsg>) -> bool {
    if !cfg().states.gameplay.allow_spectators {
        return false;
    }
    let Some(peer) = server.find_peer_mut(player_id) else { return true };
    peer.is_spectating = true;
    log::info!("{} (id {}) watches the round until the next lobby", colorize(&peer.nickname), player_id);
    crate::states::lobby::send_greeting(player_id, server, outbox);
    match server.state {
        GameState::MapVote => crate::states::map_vote::send_vote(player_id, server, outbox),
        GameState::CharSelect => crate::states::char_select::send_stage(player_id, server, outbox),
        GameState::Game => send_round(player_id, server, outbox),
        GameState::Results => crate::states::results::send_results_screen(player_id, server, outbox),
        GameState::Lobby => {}
    }
    true
}

/// Called as the lobby's countdown starts a round: whoever is not ready, or is ready
/// with the freecam card, watches it instead of playing it. Off the roster before the
/// map vote, they have no vote, are not drawn as EXE and are given no character;
/// game_init walks them into the round by enter_round, as it does any spectator.
pub fn seat_lobby_watchers(server: &mut Server, outbox: &mut Vec<OutboxMsg>) {
    for peer in server.peers.iter_mut().filter(|peer| peer.in_game && (!peer.ready || peer.chose_freecam())) {
        peer.in_game = false;
        peer.is_spectating = true;
        if !peer.ready {
            crate::server::send_chat(outbox, peer.id, &format!("{}not ready: watching this round", COLOR_CYAN));
        }
        log::info!("{} (id {}) watches the round from the lobby", colorize(&peer.nickname), peer.id);
    }
}

/// Called by game_init once the round exists: every spectator carried over from the
/// round before, or waiting for this one to start, is walked into it.
pub fn enter_round(server: &Server, outbox: &mut Vec<OutboxMsg>) {
    // The host's ?gui windows watch every round the same way.
    let watching: Vec<u16> = server.peers.iter().filter(|peer| peer.is_spectating).map(|peer| peer.id).chain(server.observers.iter().copied()).collect();
    for player_id in watching {
        send_round(player_id, server, outbox);
    }
}

/// Whether this peer is watching the round rather than playing it, which is what
/// makes everything it sends about the round something to drop: its own client sends
/// none of it, and a changed one is answered here. What it may still do is talk (the
/// caller lets its chat through), because it never left the waiting room's chat.
pub fn plays_no_part(player_id: u16, server: &Server) -> bool {
    server.find_peer(player_id).is_some_and(|peer| peer.is_spectating)
}

/// The running round, for one spectator: who is in it, who is exe and where, the
/// numbers it is simulated with, and then the word that starts the watching. All of
/// it is what the round really is -- a spectator is told the truth about a round
/// they are not in, not walked through a join they never made.
pub fn send_round(player_id: u16, server: &Server, outbox: &mut Vec<OutboxMsg>) {
    let send = |outbox: &mut Vec<OutboxMsg>, pkt: Packet| {
        outbox.push(OutboxMsg::SendTo(player_id, pkt.data().to_vec(), true));
    };

    // Who is playing, by the same list the lobby hands anyone who asks for it: the
    // client draws the round's players by the names and icons in it.
    crate::states::lobby::send_lobby_player_list(player_id, server, outbox);

    send(outbox, crate::states::char_select::exe_packet(&server.lobby.exes, server.game.map));

    // The characters, unless the round is played with them hidden: a round watched
    // is the round its players are seeing, not a better-informed version of it.
    if !cfg().states.gameplay.hide_player_characters {
        for (id, character) in round_characters(server) {
            let mut change = Packet::new(PacketType::SERVER_LOBBY_CHARACTER_CHANGE);
            let _ = change.write_u16(id);
            let _ = change.write_u8(character);
            send(outbox, change);
        }
    }

    // The gameplay numbers the round is simulated with, as its players were sent them.
    if let Some(setup) = server.game.round.as_ref().map(crate::states::round::config_packet) {
        outbox.push(OutboxMsg::SendTo(player_id, setup.to_vec(), true));
    }

    // "You are watching this round": the client leaves the waiting screen for the map.
    // It is its own packet rather than the round's start, so it cannot be mistaken for
    // "you are playing" and cannot arrive before the round it belongs to.
    send(outbox, Packet::new(PacketType::SERVER_SPECTATE_START));
}

/// Each player in the round and the character byte the client expects for them:
/// the exe character as-is for the exe, and the 1-based survivor character for
/// everyone else, exactly as char_select announces them.
fn round_characters(server: &Server) -> Vec<(u16, u8)> {
    server
        .peers
        .iter()
        .filter(|peer| peer.in_game)
        .map(|peer| {
            (peer.id, peer.character)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::PeerData;

    /// What a spectator is sent is the round, addressed to them alone and ending with
    /// the word that starts the watching, and what they are not is a player of it.
    #[test]
    fn watching_sends_the_round_to_the_watcher_alone_and_joins_nothing() {
        crate::config::set_test_config();

        let mut server = Server::new(0);
        server.state = GameState::Game;
        server.game.exe = 1;
        server.lobby.exes = vec![1];
        let mut playing = PeerData::new(1, String::from("10.0.0.1"));
        playing.in_game = true;
        playing.nickname = String::from("player");
        server.peers.push(playing);
        let mut waiting = PeerData::new(2, String::from("10.0.0.2"));
        waiting.nickname = String::from("watcher");
        server.peers.push(waiting);

        let mut outbox = Vec::new();
        assert!(join_late(2, &mut server, &mut outbox), "spectators are allowed");
        let watcher = server.find_peer(2).expect("the watcher is still connected");
        assert!(watcher.is_spectating && !watcher.in_game, "watching the round, not playing it");
        assert!(plays_no_part(2, &server), "nothing they send is a move in the round");
        assert!(watcher.follows_round(), "but they follow its screens");
        assert_eq!(server.ingame_count(), 1, "the roster is the one player it was");

        outbox.clear();
        send_round(2, &server, &mut outbox);
        let addressed: Vec<u16> = outbox
            .iter()
            .map(|message| match message {
                OutboxMsg::SendTo(id, ..) => *id,
                _ => panic!("the round is sent to the one watcher, not broadcast"),
            })
            .collect();
        assert!(addressed.iter().all(|&id| id == 2), "nobody in the round is told: {addressed:?}");

        let kinds: Vec<PacketType> = outbox
            .iter()
            .filter_map(|message| match message {
                OutboxMsg::SendTo(_, data, _) => PacketType::from_u8(*data.get(1)?),
                _ => None,
            })
            .collect();
        assert!(kinds.contains(&PacketType::LOBBY_PLAYER), "who is in the round: {kinds:?}");
        assert!(kinds.contains(&PacketType::SERVER_LOBBY_EXE), "who is exe and where: {kinds:?}");
        assert_eq!(kinds.last(), Some(&PacketType::SERVER_SPECTATE_START), "the watching starts last: {kinds:?}");
    }

    /// Coming in during the vote: its maps, votes and clock are sent at once, and from
    /// then on the vote reaches them as it reaches its players, but not the waiting
    /// screen of someone who is neither.
    #[test]
    fn a_late_watcher_is_shown_the_vote() {
        crate::config::set_test_config();

        let mut server = Server::new(0);
        server.state = GameState::MapVote;
        for id in [1, 2, 3] {
            let mut peer = PeerData::new(id, format!("10.0.0.{id}"));
            peer.in_game = id == 1;
            server.peers.push(peer);
        }

        let mut outbox = Vec::new();
        assert!(join_late(2, &mut server, &mut outbox));
        let kinds: Vec<PacketType> = outbox
            .iter()
            .filter_map(|message| match message {
                OutboxMsg::SendTo(2, data, _) => PacketType::from_u8(*data.get(1)?),
                _ => None,
            })
            .collect();
        for kind in [PacketType::SERVER_VOTE_MAPS, PacketType::VOTE, PacketType::SERVER_VOTE_TIME_SYNC] {
            assert!(kinds.contains(&kind), "{kind:?} in {kinds:?}");
        }

        outbox.clear();
        crate::server::send_to_round(&server, &mut outbox, &Packet::new(PacketType::SERVER_VOTE_TIME_SYNC));
        let reached: Vec<u16> = outbox.iter().filter_map(|message| match message {
            OutboxMsg::SendTo(id, ..) => Some(*id),
            _ => None,
        }).collect();
        assert_eq!(reached, vec![1, 2], "the player and the watcher, not the waiting screen");
    }

    /// As the lobby starts a round, the freecam card and not being ready both make a
    /// watcher of the round, and only the ready players with a character card play it.
    #[test]
    fn the_unready_and_the_freecam_card_watch_the_round() {
        crate::config::set_test_config();

        let mut server = Server::new(0);
        for (id, ready, survivor_card) in [(1, true, crate::packet::ANY_CHARACTER), (2, true, crate::packet::NO_CHARACTER), (3, false, 1)] {
            let mut peer = PeerData::new(id, format!("10.0.0.{id}"));
            (peer.in_game, peer.ready, peer.preferred_survivor) = (true, ready, survivor_card);
            server.peers.push(peer);
        }

        seat_lobby_watchers(&mut server, &mut Vec::new());
        let playing: Vec<u16> = server.peers.iter().filter(|peer| peer.in_game).map(|peer| peer.id).collect();
        let watching: Vec<u16> = server.peers.iter().filter(|peer| peer.is_spectating).map(|peer| peer.id).collect();
        assert_eq!((playing, watching), (vec![1], vec![2, 3]));
    }
}
