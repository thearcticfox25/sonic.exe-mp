use crate::colors::*;
use crate::config::cfg;
use crate::maps::MAP_LIST;
use crate::packet::{Packet, PacketType};
use crate::server::{broadcast_chat, send_chat, OutboxMsg, PeerData, Server};
use crate::terminal::cmd::parse_cmd;

/// One row of the waiting room's player list.
///
/// `in_round` is what the row means to someone reading it: "this person is over
/// there in the round" rather than "this person is here waiting with you". It is
/// a parameter rather than `peer.in_game` because a spectator is in neither --
/// not on the round's roster, but not on the waiting screen either, and it is
/// the round they are looking at. Rows for the round carry the character the
/// reader may see (the placeholder for anyone who has not picked yet); rows for
/// the waiting room carry that player's lobby icon.
pub fn waiting_player_info(peer: &PeerData, in_round: bool, exe_id: i32, anonymous: bool) -> Packet {
    let mut pkt = Packet::new(PacketType::SERVER_WAITING_PLAYER_INFO);
    let _ = pkt.write_u8(if in_round { 1 } else { 0 });
    let _ = pkt.write_u16(peer.id);
    if anonymous {
        let _ = pkt.write_str("anonymous");
        let _ = pkt.write_u8(0);
    } else {
        let _ = pkt.write_str(&peer.nickname);
        if in_round {
            let is_exe = peer.id as i32 == exe_id;
            let _ = pkt.write_u8(if is_exe { 1 } else { 0 });
            let _ = pkt.write_u8(peer.character);
        } else {
            let _ = pkt.write_u8(peer.lobby_icon);
        }
    }
    pkt
}

/// Whether `viewer_id` is shown real names. Operators always are, even with
/// anonymous_mode on.
pub fn anonymous_for(viewer_id: u16, server: &Server) -> bool {
    let viewer_op = server.find_peer(viewer_id).map(|peer| peer.op).unwrap_or(0);
    cfg().states.lobby_misc.anonymous_mode && viewer_op < 1
}

pub fn send_waiting_player_list(
    target_id: u16,
    exe_id: i32,
    server: &Server,
    outbox: &mut Vec<OutboxMsg>,
) {
    let anonymous = anonymous_for(target_id, server);

    for peer in server.peers.iter().filter(|peer| peer.id != target_id) {
        let pkt = waiting_player_info(peer, peer.in_game, exe_id, anonymous);
        outbox.push(OutboxMsg::SendTo(target_id, pkt.data().to_vec(), true));
    }
}

/// Adds a newcomer's row to everyone else's waiting screen. A player only lands in
/// the waiting room while the server is past Lobby, so the row is always a
/// waiting-room one. The waiting room is the only screen that has such a list, so
/// nobody else is told.
pub fn announce_waiter_joined(player_id: u16, server: &Server, outbox: &mut Vec<OutboxMsg>) {
    let Some(subject) = server.find_peer(player_id) else { return };
    let targets = server.peers.iter().filter(|peer| peer.id != player_id && !peer.in_game);
    for target in targets {
        let pkt = waiting_player_info(subject, false, -1, anonymous_for(target.id, server));
        outbox.push(OutboxMsg::SendTo(target.id, pkt.data().to_vec(), true));
    }
}

pub fn send_waiting_room_greeting(
    player_id: u16,
    map: Option<u8>,
    server: &Server,
    outbox: &mut Vec<OutboxMsg>,
) {
    crate::states::lobby::send_greeting(player_id, server, outbox);
    if let Some(map) = map {
        let idx = map as usize;
        if idx < MAP_LIST.len() {
            send_chat(outbox, player_id, &format!("{}map:~ {}", COLOR_CYAN, MAP_LIST[idx].name));
        }
    }
}

pub fn broadcast_map_announcement(map: u8, outbox: &mut Vec<OutboxMsg>) {
    let idx = map as usize;
    if idx < MAP_LIST.len() {
        broadcast_chat(outbox, &format!("{}map:~ {}", COLOR_CYAN, MAP_LIST[idx].name));
    }
}

pub fn handle_waiter_chat(
    player_id: u16,
    msg: &str,
    server: &mut Server,
    outbox: &mut Vec<OutboxMsg>,
) {
    let trimmed = msg.trim();
    if trimmed.starts_with(':') || trimmed.starts_with('.') {
        if let Some(cmd) = parse_cmd(trimmed) {
            let op = server.find_peer(player_id).map(|peer| peer.op).unwrap_or(0);
            if crate::states::lobby::exec_cmd_pub(&cmd, player_id, op, server, outbox) {
                return;
            }
        }
    }
    crate::server::relay_chat(outbox, player_id, msg);
}
