//! The one thing a client still chooses for itself and the server checks: the colors
//! of its skin and its pet's (PLAYER_PALETTE, PET_PALETTE): how many,
//! whose, and that each look gets one skin a round. Everything about the round is the server's, so
//! there is nothing else to doubt (states::round).

use crate::packet::{Packet, PacketType};
use crate::core::palettes::{BYTES_PER_SKIN_COLOR, PALETTE_TABLES};
use crate::server::Server;

/// A skin goes out as two packets: the colors drawn over, then the ones drawn instead.
const PACKETS_PER_SKIN: u8 = 2;

/// What becomes of a skin packet a client sent.
#[derive(Debug, PartialEq, Eq)]
pub enum SkinVerdict {
    /// On to everyone else.
    Relay,
    /// Not passed on: too many colors, someone else's id, or a second skin for a
    /// look that already has one this round (no skin cycling through colors).
    Drop,
    /// Colors the game has no skin of, while check_skin_colors is on.
    Disconnect,
}

/// PLAYER_PALETTE and PET_PALETTE, in the lobby and in the round alike.
pub fn judge_skin(player_id: u16, packet: &mut Packet, server: &mut Server) -> SkinVerdict {
    let is_pet = packet.packet_type() == Some(PacketType::PET_PALETTE);
    packet.pos = 2;
    let (Some(_is_from), Some(id)) = (packet.read_u8(), packet.read_u16()) else { return SkinVerdict::Drop };
    let name = if is_pet { Some(String::new()) } else { packet.read_str() };
    let (Some(_), Some(size)) = (name, packet.read_u8()) else { return SkinVerdict::Drop };
    let colors = size / BYTES_PER_SKIN_COLOR;
    let max_colors = crate::config::cfg().states.gameplay.palette_check.max_skin_colors;
    if id != player_id || size % BYTES_PER_SKIN_COLOR != 0 || colors == 0 || colors > max_colors {
        return SkinVerdict::Drop;
    }

    let look = look_of(player_id, is_pet, server);
    let Some(peer) = server.find_peer_mut(player_id) else { return SkinVerdict::Drop };
    let sent = &mut peer.plr.skin_packets[look];
    if *sent >= PACKETS_PER_SKIN {
        return SkinVerdict::Drop;
    }
    *sent += 1;

    if !is_pet && crate::config::cfg().states.gameplay.palette_check.check_skin_colors {
        packet.pos = 2;
        if !palette_player_validate(player_id, packet) {
            return SkinVerdict::Disconnect;
        }
    }
    SkinVerdict::Relay
}

/// Passes a skin packet on to everyone else if judge_skin lets it; `reliable` as the
/// lobby (no) and the round (yes) have always sent it.
pub fn relay_skin(player_id: u16, packet: &mut Packet, server: &mut Server, outbox: &mut Vec<crate::server::OutboxMsg>, reliable: bool) {
    match judge_skin(player_id, packet, server) {
        SkinVerdict::Relay => {
            packet.pos = 0;
            outbox.push(crate::server::OutboxMsg::BroadcastEx(packet.data().to_vec(), reliable, player_id));
        }
        SkinVerdict::Drop => log::debug!("skin of player {player_id} dropped"),
        SkinVerdict::Disconnect => outbox.push(crate::server::OutboxMsg::Disconnect(player_id, crate::server::DisconnectReason::Other)),
    }
}

/// Which of a player's looks a skin packet is for: 0 a survivor, 1 a demonized one,
/// 2 an EXE, 3 the pet. Each gets one skin a round.
fn look_of(player_id: u16, is_pet: bool, server: &Server) -> usize {
    let demonized = server.find_peer(player_id).is_some_and(|peer| peer.plr.flags & crate::player::flags::DEMONIZED != 0);
    match (is_pet, server.lobby.is_exe(player_id), demonized) {
        (true, _, _) => 3,
        (false, true, _) => 2,
        (false, false, true) => 1,
        (false, false, false) => 0,
    }
}

const PALETTE_EPSILON: u8 = 10;

fn palette_within_epsilon(reported: u8, expected: u8) -> bool {
    reported >= expected.saturating_sub(PALETTE_EPSILON)
        && reported <= expected.saturating_add(PALETTE_EPSILON)
}

fn palette_matches(colors: &[[u8; 3]], palette: &[u32]) -> bool {
    let count = colors.len().min(palette.len());
    if count == 0 {
        return false;
    }
    let mut match_count = 0usize;
    for slot in 0..count {
        let rgb = palette[slot];
        let red   = ((rgb >> 16) & 0xFF) as u8;
        let green = ((rgb >> 8)  & 0xFF) as u8;
        let blue  = (rgb        & 0xFF) as u8;
        if palette_within_epsilon(colors[slot][0], red)
            && palette_within_epsilon(colors[slot][1], green)
            && palette_within_epsilon(colors[slot][2], blue)
        {
            match_count += 1;
        }
    }
    match_count != 0 && match_count == count
}

pub fn palette_player_validate(player_id: u16, packet: &mut Packet) -> bool {
    let (Some(from), Some(id), Some(name), Some(size)) = (packet.read_u8(), packet.read_u16(), packet.read_str(), packet.read_u8()) else {
        return false;
    };

    if size == 0 { return false; }
    if id != player_id { return false; }

    let count = (size / 4) as usize;
    if count > 19 { return false; }

    let is_custom = name.starts_with("custom");

    let mut colors = [[0u8; 3]; 19];
    for slot in 0..count {
        let (Some(red), Some(green), Some(blue)) = (packet.read_u8(), packet.read_u8(), packet.read_u8()) else {
            return false;
        };
        let _ =       packet.read_u8();
        colors[slot] = [red, green, blue];
    }
    let colors = &colors[..count];

    // The skin tables are shared with the game (crate::core::palettes).
    let known: Vec<&[u32]> = match (from != 0, is_custom) {
        (true, true) => PALETTE_TABLES.iter().map(|table| table.custom_colors).collect(),
        (true, false) => PALETTE_TABLES.iter().flat_map(|table| table.skins.iter().map(|skin| skin.from)).collect(),
        (false, true) => return true,
        (false, false) => PALETTE_TABLES.iter().flat_map(|table| table.skins.iter().map(|skin| skin.to)).collect(),
    };
    known.iter().any(|palette| palette_matches(colors, palette))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// PLAYER_PALETTE as the game sends it (client net.rs send_skin).
    fn skin_packet(id: u16, is_from: bool, colors: usize) -> Packet {
        let mut packet = Packet::passthrough(PacketType::PLAYER_PALETTE);
        let _ = packet.write_u8(u8::from(is_from));
        let _ = packet.write_u16(id);
        let _ = packet.write_str("classic");
        let _ = packet.write_u8((colors * BYTES_PER_SKIN_COLOR as usize) as u8);
        for _ in 0..colors * BYTES_PER_SKIN_COLOR as usize {
            let _ = packet.write_u8(0);
        }
        Packet::from_data(packet.data())
    }

    fn server_with_player() -> Server {
        crate::config::set_test_config();
        let mut server = Server::new(0);
        server.peers.push(crate::server::PeerData::new(1, String::from("10.0.0.1")));
        server
    }

    /// A look's skin is sent once a round: a client that keeps sending new colors
    /// (a rainbow) is not passed on after the first skin.
    #[test]
    fn a_look_gets_one_skin_a_round() {
        let mut server = server_with_player();
        let mut judge = |is_from: bool| judge_skin(1, &mut skin_packet(1, is_from, 2), &mut server);
        assert_eq!((judge(true), judge(false)), (SkinVerdict::Relay, SkinVerdict::Relay));
        assert_eq!((judge(true), judge(false)), (SkinVerdict::Drop, SkinVerdict::Drop));

        // Demonized, the player has a new look and so one more skin.
        server.peers[0].plr.flags |= crate::player::flags::DEMONIZED;
        assert_eq!(judge_skin(1, &mut skin_packet(1, true, 2), &mut server), SkinVerdict::Relay);
    }

    #[test]
    fn too_many_colors_and_someone_else_s_skin_are_dropped() {
        let mut server = server_with_player();
        let max_colors = crate::config::cfg().states.gameplay.palette_check.max_skin_colors as usize;
        assert_eq!(judge_skin(1, &mut skin_packet(1, true, max_colors + 1), &mut server), SkinVerdict::Drop);
        assert_eq!(judge_skin(1, &mut skin_packet(2, true, 2), &mut server), SkinVerdict::Drop);
        assert_eq!(judge_skin(1, &mut skin_packet(1, true, max_colors), &mut server), SkinVerdict::Relay);
    }
}
