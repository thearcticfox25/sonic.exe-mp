use crate::config::cfg;
use crate::packet::{Packet, PacketType};
use crate::packet::{ANY_CHARACTER, EXE_COUNT, NO_CHARACTER, SURVIVOR_COUNT};
use crate::server::{send_to_round, DisconnectReason, GameState, OutboxMsg, Server};
use crate::states::game::game_init;
use crate::states::{seconds, time_packet};

use rand::RngExt;

/// After the map vote: the EXE are drawn, and everyone picks their character on the
/// character select stage (character_selection.enable), as in TD2DR. A character
/// picked ahead in the lobby (lobby_preselection) is given at once if it is free.
/// With the stage off, everyone gets what they picked ahead, or a random character,
/// and the round starts at once.
pub fn charselect_init(server: &mut Server, outbox: &mut Vec<OutboxMsg>) {
    log::debug!("Entering CharSelect state...");
    for peer in server.peers.iter_mut() {
        peer.character = NO_CHARACTER;
    }

    let exes = pick_exes(server);
    for &exe_id in &exes {
        if let Some(peer) = server.find_peer(exe_id) {
            log::info!("{} (id {}, c {}) is exe!", crate::colors::colorize(&peer.nickname), exe_id, peer.exe_chance);
        }
    }
    server.lobby.exes = exes.clone();
    let exe_id = server.lobby.main_exe();
    let Some(map) = server.lobby.map else {
        log::error!("Handing out the characters without a chosen map");
        return;
    };

    send_to_round(server, outbox, &exe_packet(&exes, map));

    for peer in server.peers.iter_mut() {
        if exes.contains(&peer.id) {
            peer.exe_chance = 1;
        } else if peer.exe_chance < 100 {
            peer.exe_chance += 1;
        }
    }

    let selection = &cfg().states.lobby_misc.character_selection;
    if !selection.enable {
        let assigned = assign_preferred_characters(&exes, server);
        send_characters(&assigned, &exes, server, outbox);
        // ponytail: the round rules below know one EXE; the others become EXE there with the authoritative round (core)
        game_init(exe_id as i32, map, server, outbox);
        return;
    }

    server.state = GameState::CharSelect;
    server.lobby.countdown = seconds(1.0);
    server.lobby.countdown_sec = selection.charselect_timer;
    send_to_round(server, outbox, &time_packet(PacketType::SERVER_CHAR_TIME_SYNC, server.lobby.countdown_sec));
    log::info!("Server is now in Character Select");

    let wishes: Vec<(u16, u8)> = server.peers.iter()
        .filter(|peer| peer.in_game)
        .map(|peer| (peer.id, if exes.contains(&peer.id) { peer.preferred_exe } else { peer.preferred_survivor }))
        .filter(|&(_, wanted)| !matches!(wanted, NO_CHARACTER | ANY_CHARACTER))
        .collect();
    for (player_id, wanted) in wishes {
        if may_take(player_id, wanted, server) {
            give_character(player_id, wanted, server, outbox);
        }
    }
    start_round_if_everyone_picked(server, outbox);
}

/// SERVER_LOBBY_EXE: who is EXE (the first is the one the round rules know, the
/// others follow) and which map is played.
pub fn exe_packet(exes: &[u16], map: u8) -> Packet {
    let mut pkt = Packet::new(PacketType::SERVER_LOBBY_EXE);
    let _ = pkt.write_u16(exes.first().copied().unwrap_or(0));
    let _ = pkt.write_u16(map as u16);
    let others = exes.get(1..).unwrap_or_default();
    let _ = pkt.write_u8(others.len() as u8);
    for &other_exe in others {
        let _ = pkt.write_u16(other_exe);
    }
    pkt
}

/// SERVER_LOBBY_CHARACTER_CHANGE: a player's character, which everyone's icons show.
fn change_packet(player_id: u16, character: u8) -> Packet {
    let mut pkt = Packet::new(PacketType::SERVER_LOBBY_CHARACTER_CHANGE);
    let _ = pkt.write_u16(player_id);
    let _ = pkt.write_u8(character);
    pkt
}

/// The stage as it stands, for a player who starts watching it in the middle
/// (states::spectate::join_late): who plays, who is EXE where, who has picked what,
/// and last its clock, which tells their client to show the stage.
pub fn send_stage(player_id: u16, server: &Server, outbox: &mut Vec<OutboxMsg>) {
    crate::states::lobby::send_lobby_player_list(player_id, server, outbox);
    let Some(map) = server.lobby.map else { return };
    let mut packets = vec![exe_packet(&server.lobby.exes, map)];
    packets.extend(server.peers.iter().filter(|peer| peer.in_game && peer.character != NO_CHARACTER).map(|peer| change_packet(peer.id, peer.character)));
    packets.push(time_packet(PacketType::SERVER_CHAR_TIME_SYNC, server.lobby.countdown_sec));
    for pkt in packets {
        outbox.push(OutboxMsg::SendTo(player_id, pkt.data().to_vec(), true));
    }
}

/// Whether `player_id` may have `character` of their side: it is one, and nobody else
/// of the side has it, unless characters are shared (charselect_mod_unlocked; the
/// only EXE of a round shares with nobody anyway).
fn may_take(player_id: u16, character: u8, server: &Server) -> bool {
    let exes = &server.lobby.exes;
    let is_exe = exes.contains(&player_id);
    let count = if is_exe { EXE_COUNT } else { SURVIVOR_COUNT };
    if !(1..=count).contains(&character) {
        return false;
    }
    let shared = cfg().states.lobby_misc.character_selection.charselect_mod_unlocked;
    if shared {
        return true;
    }
    !server.peers.iter().any(|peer| peer.id != player_id && peer.in_game && exes.contains(&peer.id) == is_exe && peer.character == character)
}

/// The player has their character: they are told, and so is everyone else, whose
/// icons cross it out.
fn give_character(player_id: u16, character: u8, server: &mut Server, outbox: &mut Vec<OutboxMsg>) {
    let is_exe = server.lobby.exes.contains(&player_id);
    let Some(peer) = server.find_peer_mut(player_id) else { return };
    peer.character = character;
    log::info!("{} (id {}) chooses character {}{}!", crate::colors::colorize(&peer.nickname), player_id, if is_exe { "EXE " } else { "" }, character);
    send_choice(player_id, character, is_exe, true, outbox);
    send_to_round(server, outbox, &change_packet(player_id, character));
}

/// LOBBY_CHARACTER or LOBBY_EXECHARACTER: the
/// character asked for, and whether the player got it.
fn send_choice(player_id: u16, character: u8, is_exe: bool, granted: bool, outbox: &mut Vec<OutboxMsg>) {
    let kind = if is_exe { PacketType::LOBBY_EXECHARACTER } else { PacketType::LOBBY_CHARACTER };
    let mut resp = Packet::new(kind);
    let _ = resp.write_u8(character);
    let _ = resp.write_u8(u8::from(granted));
    outbox.push(OutboxMsg::SendTo(player_id, resp.data().to_vec(), true));
}

fn start_round_if_everyone_picked(server: &mut Server, outbox: &mut Vec<OutboxMsg>) {
    if server.peers.iter().filter(|peer| peer.in_game).all(|peer| peer.character != NO_CHARACTER) {
        start_round(server, outbox);
    }
}

fn start_round(server: &mut Server, outbox: &mut Vec<OutboxMsg>) {
    let Some(map) = server.lobby.map else { return };
    let exe_id = server.lobby.main_exe();
    // ponytail: the round rules below know one EXE; the others become EXE there with the authoritative round (core)
    game_init(exe_id as i32, map, server, outbox);
}

/// A player asks for a character of their side (LOBBY_CHARACTER for a
/// survivor, LOBBY_EXECHARACTER for an EXE). Anything else is ignored.
fn request_character(player_id: u16, character: u8, asks_as_exe: bool, server: &mut Server, outbox: &mut Vec<OutboxMsg>) {
    let Some(peer) = server.find_peer(player_id) else { return };
    let is_exe = server.lobby.exes.contains(&player_id);
    if !peer.in_game || peer.character != NO_CHARACTER || is_exe != asks_as_exe {
        return;
    }
    if may_take(player_id, character, server) {
        give_character(player_id, character, server, outbox);
        start_round_if_everyone_picked(server, outbox);
    } else {
        send_choice(player_id, character, is_exe, false, outbox);
    }
}

pub fn charselect_state_join(player_id: u16, server: &mut Server, outbox: &mut Vec<OutboxMsg>) {
    if crate::states::spectate::join_late(player_id, server, outbox) {
        return;
    }
    use crate::states::waiting_room as wr;
    let exe_id = server.lobby.main_exe() as i32;
    wr::send_waiting_player_list(player_id, exe_id, server, outbox);
    wr::announce_waiter_joined(player_id, server, outbox);
    wr::send_waiting_room_greeting(player_id, server.lobby.map, server, outbox);
}

/// A player left while everyone picks: without an EXE or enough players the round is
/// called off, otherwise the others may now be the last to have picked.
pub fn charselect_state_left(player_id: u16, server: &mut Server, outbox: &mut Vec<OutboxMsg>) {
    let mut pkt = Packet::new(PacketType::SERVER_PLAYER_LEFT);
    let _ = pkt.write_u16(player_id);
    outbox.push(OutboxMsg::BroadcastEx(pkt.data().to_vec(), true, player_id));

    let min_to_continue = cfg().states.lobby_misc.min_players_required.max(1) as usize;
    let remaining = server.peers.iter().filter(|peer| peer.in_game && peer.id != player_id).count();
    if remaining < min_to_continue || server.lobby.exes.contains(&player_id) {
        crate::states::lobby::lobby_init(server, outbox);
        crate::states::lobby::lobby_broadcast_init(server, outbox);
        return;
    }
    if let Some(peer) = server.find_peer_mut(player_id) {
        peer.in_game = false;
    }
    start_round_if_everyone_picked(server, outbox);
}

pub fn charselect_state_handle(player_id: u16, packet: &mut Packet, server: &mut Server, outbox: &mut Vec<OutboxMsg>) {
    let Some(packet_type) = packet.packet_type() else { return };
    packet.pos = 2;
    match packet_type {
        PacketType::LOBBY_CHARACTER | PacketType::LOBBY_EXECHARACTER => {
            let Some(character) = packet.read_u8() else { return };
            request_character(player_id, character, packet_type == PacketType::LOBBY_EXECHARACTER, server, outbox);
        }
        PacketType::CHAT_MESSAGE => {
            if !server.chat_rate_allow(player_id) { return; } // anti-flood
            let _sender = packet.read_u16();
            let Some(msg) = packet.read_str() else { return };
            let in_game = server.find_peer(player_id).map(|peer| peer.in_game).unwrap_or(false);
            if !in_game {
                crate::states::waiting_room::handle_waiter_chat(player_id, &msg, server, outbox);
                return;
            }
            let nick = server.find_peer(player_id).map(|peer| peer.nickname.clone()).unwrap_or_default();
            log::info!("{} (id {}): {}", crate::colors::colorize(&nick), player_id, msg);
            crate::server::relay_chat(outbox, player_id, &msg);
        }
        PacketType::PING => {
            let pkt = Packet::new(PacketType::PONG);
            outbox.push(OutboxMsg::SendTo(player_id, pkt.data().to_vec(), false));
        }
        _ => {}
    }
}

/// Once a second the clock goes down. At 0 whoever has not picked is kicked for being
/// away, and a second later the rest get random characters and the round starts.
pub fn charselect_state_tick(server: &mut Server, outbox: &mut Vec<OutboxMsg>) {
    server.lobby.countdown -= 1.0;
    if server.lobby.countdown > 0.0 {
        return;
    }
    server.lobby.countdown += seconds(1.0);
    if server.lobby.countdown_sec == 0 {
        let exes = server.lobby.exes.clone();
        let assigned = assign_preferred_characters(&exes, server);
        send_characters(&assigned, &exes, server, outbox);
        start_round(server, outbox);
        return;
    }
    server.lobby.countdown_sec -= 1;
    if server.lobby.countdown_sec == 0 {
        let away: Vec<u16> = server.peers.iter().filter(|peer| peer.in_game && peer.character == NO_CHARACTER).map(|peer| peer.id).collect();
        for id in away {
            outbox.push(OutboxMsg::Disconnect(id, DisconnectReason::AfkTimeout));
        }
    }
    send_to_round(server, outbox, &time_packet(PacketType::SERVER_CHAR_TIME_SYNC, server.lobby.countdown_sec));
}

/// Tells the `assigned` players their own character, and everyone else theirs.
fn send_characters(assigned: &[u16], exes: &[u16], server: &Server, outbox: &mut Vec<OutboxMsg>) {
    for peer in server.peers.iter().filter(|peer| assigned.contains(&peer.id)) {
        send_choice(peer.id, peer.character, exes.contains(&peer.id), true, outbox);
        send_to_round(server, outbox, &change_packet(peer.id, peer.character));
    }
}

/// Everyone who has no character yet gets what their preference card asked for, or
/// a random one. A card without a wish, or with a character that is taken while
/// characters are not shared, gets a random character instead. Returns who got one.
fn assign_preferred_characters(exes: &[u16], server: &mut Server) -> Vec<u16> {
    let duplicates_allowed = cfg().states.lobby_misc.character_selection.charselect_mod_unlocked;
    // With one EXE nobody else can hold its character.
    let exes_share = duplicates_allowed || exes.len() == 1;
    let mut free_survivors = Free::new(SURVIVOR_COUNT);
    let mut free_exes = Free::new(EXE_COUNT);
    for peer in server.peers.iter().filter(|peer| peer.in_game) {
        let free = if exes.contains(&peer.id) { &mut free_exes } else { &mut free_survivors };
        free.take(peer.character);
    }
    let mut rng = rand::rng();
    let mut assigned = Vec::new();

    for peer in server.peers.iter_mut().filter(|peer| peer.in_game && peer.character == NO_CHARACTER) {
        let (wanted, free, shared) = if exes.contains(&peer.id) {
            (peer.preferred_exe, &mut free_exes, exes_share)
        } else {
            (peer.preferred_survivor, &mut free_survivors, duplicates_allowed)
        };
        let granted = free.is_character(wanted) && (shared || free.is_free(wanted));
        peer.character = if granted { wanted } else { free.random(shared, &mut rng) };
        free.take(peer.character);
        assigned.push(peer.id);
    }
    assigned
}

/// Which characters of one side nobody holds yet, by character number (from 1).
struct Free(Vec<bool>);

impl Free {
    fn new(count: u8) -> Free {
        Free(vec![true; count as usize])
    }

    fn is_character(&self, character: u8) -> bool {
        (1..=self.0.len()).contains(&(character as usize))
    }

    fn is_free(&self, character: u8) -> bool {
        self.is_character(character) && self.0[character as usize - 1]
    }

    fn take(&mut self, character: u8) {
        if self.is_character(character) {
            self.0[character as usize - 1] = false;
        }
    }

    /// A random character; only a free one unless characters are shared. The config
    /// check keeps enough characters free; the first one if not.
    fn random(&self, shared: bool, rng: &mut impl RngExt) -> u8 {
        let choices: Vec<u8> = (1..=self.0.len() as u8).filter(|&character| shared || self.is_free(character)).collect();
        if choices.is_empty() {
            return 1;
        }
        choices[rng.random_range(0..choices.len())]
    }
}


/// The round's EXE: exe_count of them, but at least one survivor stays. Each is the
/// original weighted draw among the players not drawn yet; players whose card
/// says "not me" are drawn only when nobody else is left.
fn pick_exes(server: &Server) -> Vec<u16> {
    let players = server.peers.iter().filter(|peer| peer.in_game).count();
    let count = (cfg().states.lobby_misc.character_selection.exe_count as usize).min(players.saturating_sub(1)).max(1);
    let mut exes = Vec::with_capacity(count);
    while exes.len() < count {
        exes.push(pick_exe(server, &exes));
    }
    exes
}

fn pick_exe(server: &Server, already_exe: &[u16]) -> u16 {
    let not_drawn: Vec<&crate::server::PeerData> = server.peers.iter()
        .filter(|peer| peer.in_game && !already_exe.contains(&peer.id))
        .collect();
    let willing: Vec<&crate::server::PeerData> = not_drawn.iter().copied().filter(|peer| !peer.refuses_exe).collect();
    let mut ingame = if willing.is_empty() { not_drawn } else { willing };

    if ingame.is_empty() {
        return server.peers.first().map(|peer| peer.id).unwrap_or(1);
    }

    for peer in &ingame {
        if peer.exe_chance >= 100 {
            return peer.id;
        }
    }

    let eligible: Vec<&crate::server::PeerData> = ingame.iter()
        .copied()
        .collect();
    if !eligible.is_empty() {
        ingame = eligible;
    }

    let total_weight: u32 = ingame.iter().map(|peer| peer.exe_chance as u32).sum();
    let weight = if total_weight == 0 { 1 } else { total_weight };

    let mut rng = rand::rng();
    let mut roll: u32 = rng.random_range(0..weight);

    for peer in &ingame {
        if roll < peer.exe_chance as u32 {
            return peer.id;
        }
        roll -= peer.exe_chance as u32;
    }

    ingame.last().map(|peer| peer.id).unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::PeerData;

    /// Player 1 is drawn as EXE; survivors 2 and 3 both picked Knuckles ahead.
    #[test]
    fn stage_gives_free_wishes_and_refuses_taken_characters() {
        crate::config::set_test_config();
        crate::status::init_empty_status();
        let mut server = Server::new(0);
        server.lobby.map = Some(0);
        for id in [1, 2, 3] {
            let mut peer = PeerData::new(id, format!("10.0.0.{id}"));
            peer.in_game = true;
            peer.preferred_survivor = 2;
            peer.exe_chance = if id == 1 { 100 } else { 1 };
            server.peers.push(peer);
        }
        let mut outbox = Vec::new();
        charselect_init(&mut server, &mut outbox);
        assert_eq!(server.state, GameState::CharSelect);
        let character = |server: &Server, id| server.find_peer(id).unwrap().character;
        let late = if character(&server, 2) == 2 { 3 } else { 2 };
        assert_eq!(character(&server, late), NO_CHARACTER, "one Knuckles only");

        request_character(late, 2, false, &mut server, &mut outbox);
        assert_eq!(character(&server, late), NO_CHARACTER, "taken: refused");
        request_character(late, 3, false, &mut server, &mut outbox);
        request_character(1, 2, false, &mut server, &mut outbox);
        assert_eq!(character(&server, 1), NO_CHARACTER, "the EXE asks for a survivor: ignored");
        assert_eq!(server.state, GameState::CharSelect, "the EXE has not picked yet");

        request_character(1, 2, true, &mut server, &mut outbox);
        assert_eq!(server.state, GameState::Game, "everyone picked: the round starts");
    }
}
