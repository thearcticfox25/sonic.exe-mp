//! The packets the server and the game exchange: packet types and the byte
//! layout of GameMaker buffers (little endian, zero-terminated strings).
//! Every packet starts with a "passthrough" byte and the packet type.

/// The version of this project, one character: the game sends it, the server expects it
/// and calls itself by it. It travels as its Unicode code point in a u16.
pub const VERSION: char = '\u{3b1}'; // greek small letter alpha
/// The server's port when its config names none, and the game's when the address
/// names none ("host:port" names another).
pub const DEFAULT_PORT: u16 = 7811;
/// A player's character as the packets carry it: 0 for nobody, then the characters of
/// their side counted from 1 (a survivor: Tails 1 .. Sally 6; an EXE: EXE 1 .. Exeller
/// 4). Which side a player is on is told apart (the round's list of EXE). In a
/// preference card, 255 asks for any character, and 0 on the EXE card says "not me".
pub const NO_CHARACTER: u8 = 0;
pub const ANY_CHARACTER: u8 = 255;
pub const SURVIVOR_COUNT: u8 = 6;
pub const EXE_COUNT: u8 = 4;
/// What LOBBY_CHOOSE_PLAYER is for, its first byte: the server sends the byte alone
/// to open the player picker, the game sends it back with the picked player's id.
pub const CHOOSE_VOTEKICK: u8 = 0;
pub const CHOOSE_KICK: u8 = 1;
pub const CHOOSE_BAN: u8 = 2;
pub const CHOOSE_OP: u8 = 3;
/// A packet's type names what it is about. SERVER_ and CLIENT_ say who sends it when it
/// only ever goes one way; a type without them goes both ways with the same purpose, each
/// side knowing which it is (CHAT_MESSAGE, LOBBY_READY_STATE: a client says its own, the
/// server tells everyone whose). PING and PONG stay a pair, as the original named them.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types, dead_code)]
pub enum PacketType {
    IDENTITY = 0,
    SERVER_IDENTITY_RESPONSE,
    SERVER_PLAYER_JOINED,
    SERVER_PLAYER_LEFT,
    SERVER_WAITING_PLAYER_INFO,
    LOBBY_READY_STATE,
    SERVER_LOBBY_EXE,
    SERVER_LOBBY_COUNTDOWN,
    SERVER_LOBBY_CHARACTER_CHANGE,
    LOBBY_CHARACTER,
    LOBBY_EXECHARACTER,
    SERVER_LOBBY_GAME_START,
    LOBBY_PLAYER,
    SERVER_LOBBY_EXE_CHANCE,
    SERVER_LOBBY_CORRECT,
    LOBBY_CHOOSE_PLAYER,
    SERVER_LOBBY_CHANGELOBBY,
    SERVER_CHAR_TIME_SYNC,
    SERVER_VOTE_MAPS,
    VOTE,
    SERVER_VOTE_TIME_SYNC,
    SERVER_GAME_PLAYERS_READY,
    SERVER_GAME_EXE_WINS,
    SERVER_GAME_SURVIVOR_WIN,
    SERVER_GAME_SPAWN_RING,
    SERVER_GAME_PLAYER_ESCAPED,
    SERVER_GAME_BACK_TO_LOBBY,
    SERVER_GAME_TIME_SYNC,
    SERVER_GAME_TIME_OVER,
    SERVER_GAME_PING,
    SERVER_GAME_DEATHTIMER_TICK,
    SERVER_GAME_DEATHTIMER_END,
    PING,
    PONG,
    SERVER_REVIVAL_PROGRESS,
    SERVER_REVIVAL_STATUS,
    SERVER_REVIVAL_RINGSUB,
    SERVER_REVIVAL_REVIVED,
    SERVER_PLAYER_ESCAPED,
    CHAT_MESSAGE,
    PLAYER_PALETTE,
    PET_PALETTE,
    SERVER_RESULTS,
    RESULTS_DATA,
    /// Authoritative rounds: the buttons of the latest ticks, newest last. The only thing a client says about its player.
    CLIENT_ROUND_INPUT,
    /// Authoritative rounds: the simulated state of the players (crate::core::snapshot),
    /// whole: the keyframe the deltas after it are taken from.
    SERVER_ROUND_SNAPSHOT,
    /// Authoritative rounds: a snapshot as what changed since the keyframe (snapshot::delta).
    SERVER_ROUND_DELTA,
    /// Authoritative rounds: the gameplay numbers the server simulates with.
    SERVER_ROUND_CONFIG,
    /// Authoritative rounds: the player opened or closed their pause menu. Everyone
    /// else is shown that they are away (snapshot flag AFK); the round runs on.
    CLIENT_ROUND_PAUSE,
    /// The server started sudden death: the clients show its words. The original
    /// guessed the moment from the clock reading 2:01, which is the server's to say.
    SERVER_GAME_SUDDEN_DEATH,
    /// This client is watching the running round from the waiting room
    /// (states::spectate): it has no player in it and follows it from the outside.
    SERVER_SPECTATE_START,
}

impl PacketType {
    /// Largest valid discriminant. The enum is `#[repr(u8)]` and contiguous from 0,
    /// so any byte in `0..=MAX` maps to a variant.
    const MAX: u8 = PacketType::SERVER_SPECTATE_START as u8;

    /// Safe `u8 -> PacketType`. Replaces a former `mem::transmute`, which would have
    /// become UB the moment anyone introduced a gap in the discriminants. This relies
    /// only on the layout being contiguous for validity (no `unsafe`).
    pub fn from_u8(raw: u8) -> Option<PacketType> {
        if raw <= Self::MAX {
            Some(PACKET_TYPE_TABLE[raw as usize])
        } else {
            None
        }
    }
}

/// Lookup table from raw byte to `PacketType`, materialized at compile time (zero
/// runtime cost). Entry `i` is the variant whose discriminant is `i`. Construction
/// `assert!`s contiguity, so a future gap in the enum is a compile error rather than
/// a silently wrong/invalid variant (the failure mode of the old `mem::transmute`).
static PACKET_TYPE_TABLE: [PacketType; PacketType::MAX as usize + 1] = build_packet_type_table();

const fn build_packet_type_table() -> [PacketType; PacketType::MAX as usize + 1] {
    // All variants in discriminant order. If a variant is added, append it here;
    // the asserts below catch any mismatch between this list and the repr.
    const VARIANTS: &[PacketType] = &[
        PacketType::IDENTITY,
        PacketType::SERVER_IDENTITY_RESPONSE,
        PacketType::SERVER_PLAYER_JOINED,
        PacketType::SERVER_PLAYER_LEFT,
        PacketType::SERVER_WAITING_PLAYER_INFO,
        PacketType::LOBBY_READY_STATE,
        PacketType::SERVER_LOBBY_EXE,
        PacketType::SERVER_LOBBY_COUNTDOWN,
        PacketType::SERVER_LOBBY_CHARACTER_CHANGE,
        PacketType::LOBBY_CHARACTER,
        PacketType::LOBBY_EXECHARACTER,
        PacketType::SERVER_LOBBY_GAME_START,
        PacketType::LOBBY_PLAYER,
        PacketType::SERVER_LOBBY_EXE_CHANCE,
        PacketType::SERVER_LOBBY_CORRECT,
        PacketType::LOBBY_CHOOSE_PLAYER,
        PacketType::SERVER_LOBBY_CHANGELOBBY,
        PacketType::SERVER_CHAR_TIME_SYNC,
        PacketType::SERVER_VOTE_MAPS,
        PacketType::VOTE,
        PacketType::SERVER_VOTE_TIME_SYNC,
        PacketType::SERVER_GAME_PLAYERS_READY,
        PacketType::SERVER_GAME_EXE_WINS,
        PacketType::SERVER_GAME_SURVIVOR_WIN,
        PacketType::SERVER_GAME_SPAWN_RING,
        PacketType::SERVER_GAME_PLAYER_ESCAPED,
        PacketType::SERVER_GAME_BACK_TO_LOBBY,
        PacketType::SERVER_GAME_TIME_SYNC,
        PacketType::SERVER_GAME_TIME_OVER,
        PacketType::SERVER_GAME_PING,
        PacketType::SERVER_GAME_DEATHTIMER_TICK,
        PacketType::SERVER_GAME_DEATHTIMER_END,
        PacketType::PING,
        PacketType::PONG,
        PacketType::SERVER_REVIVAL_PROGRESS,
        PacketType::SERVER_REVIVAL_STATUS,
        PacketType::SERVER_REVIVAL_RINGSUB,
        PacketType::SERVER_REVIVAL_REVIVED,
        PacketType::SERVER_PLAYER_ESCAPED,
        PacketType::CHAT_MESSAGE,
        PacketType::PLAYER_PALETTE,
        PacketType::PET_PALETTE,
        PacketType::SERVER_RESULTS,
        PacketType::RESULTS_DATA,
        PacketType::CLIENT_ROUND_INPUT,
        PacketType::SERVER_ROUND_SNAPSHOT,
        PacketType::SERVER_ROUND_DELTA,
        PacketType::SERVER_ROUND_CONFIG,
        PacketType::CLIENT_ROUND_PAUSE,
        PacketType::SERVER_GAME_SUDDEN_DEATH,
        PacketType::SERVER_SPECTATE_START,
    ];

    let variant_count = PacketType::MAX as usize + 1;
    assert!(
        VARIANTS.len() == variant_count,
        "PACKET_TYPE_TABLE variant list out of sync with PacketType repr"
    );
    let mut table = [PacketType::IDENTITY; PacketType::MAX as usize + 1];
    let mut type_id = 0;
    while type_id < variant_count {
        let variant = VARIANTS[type_id];
        assert!(
            variant as usize == type_id,
            "PacketType discriminants are not contiguous from 0 (gap detected)"
        );
        table[type_id] = variant;
        type_id += 1;
    }
    table
}

pub const PACKET_MAXSIZE: usize = 256;

#[derive(Clone)]
pub struct Packet {
    pub buf: [u8; PACKET_MAXSIZE],
    pub pos: usize,
    pub len: usize,
}

impl Packet {
    pub fn new(ptype: PacketType) -> Self {
        let mut packet = Self { buf: [0u8; PACKET_MAXSIZE], pos: 0, len: 0 };
        packet.write_u8(0).ok();
        packet.write_u8(ptype as u8).ok();
        packet
    }

    /// A packet the server passes on to the other players (cpacket_tcp(type, true)).
    pub fn passthrough(ptype: PacketType) -> Self {
        let mut packet = Self::new(ptype);
        packet.buf[0] = 1;
        packet
    }

    pub fn from_data(data: &[u8]) -> Self {
        let len = data.len().min(PACKET_MAXSIZE);
        let mut packet = Self { buf: [0u8; PACKET_MAXSIZE], pos: 0, len };
        packet.buf[..len].copy_from_slice(&data[..len]);
        packet
    }

    pub fn packet_type(&self) -> Option<PacketType> {
        if self.len >= 2 { PacketType::from_u8(self.buf[1]) } else { None }
    }

    pub fn data(&self) -> &[u8] {
        &self.buf[..self.len]
    }

    pub fn read_u8(&mut self) -> Option<u8> {
        if self.pos < self.len {
            let value = self.buf[self.pos];
            self.pos += 1;
            Some(value)
        } else {
            None
        }
    }

    pub fn read_u16(&mut self) -> Option<u16> {
        if self.pos + 2 > self.len { return None; }
        let value = u16::from_le_bytes([self.buf[self.pos], self.buf[self.pos + 1]]);
        self.pos += 2;
        Some(value)
    }

    /// The character written by `write_char`; None when the code point is not one
    /// (a lone surrogate).
    pub fn read_char(&mut self) -> Option<char> {
        char::from_u32(u32::from(self.read_u16()?))
    }

    pub fn read_u32(&mut self) -> Option<u32> {
        if self.pos + 4 > self.len { return None; }
        let value = u32::from_le_bytes(self.buf[self.pos..self.pos + 4].try_into().unwrap());
        self.pos += 4;
        Some(value)
    }

    pub fn read_i32(&mut self) -> Option<i32> {
        self.read_u32().map(|dword| dword as i32)
    }

    pub fn read_u64(&mut self) -> Option<u64> {
        if self.pos + 8 > self.len { return None; }
        let value = u64::from_le_bytes(self.buf[self.pos..self.pos + 8].try_into().unwrap());
        self.pos += 8;
        Some(value)
    }

    pub fn read_f32(&mut self) -> Option<f32> {
        if self.pos + 4 > self.len { return None; }
        let value = f32::from_le_bytes(self.buf[self.pos..self.pos + 4].try_into().unwrap());
        self.pos += 4;
        Some(value)
    }

    pub fn read_f64(&mut self) -> Option<f64> {
        self.read_u64().map(f64::from_bits)
    }

    pub fn read_str(&mut self) -> Option<String> {
        let mut bytes = Vec::new();
        loop {
            if self.pos >= self.len { return None; }
            if bytes.len() >= 128 { return None; }
            let ch = self.buf[self.pos];
            self.pos += 1;
            if ch == 0 { break; }
            bytes.push(ch);
        }
        Some(String::from_utf8_lossy(&bytes).into_owned())
    }

    pub fn write_u8(&mut self, value: u8) -> Result<(), ()> {
        if self.pos + 1 >= PACKET_MAXSIZE { return Err(()); }
        if self.pos + 1 >= self.len { self.len += 1; }
        self.buf[self.pos] = value;
        self.pos += 1;
        Ok(())
    }

    pub fn write_u16(&mut self, value: u16) -> Result<(), ()> {
        if self.pos + 2 >= PACKET_MAXSIZE { return Err(()); }
        if self.pos + 2 >= self.len { self.len += 2; }
        let bytes = value.to_le_bytes();
        self.buf[self.pos] = bytes[0];
        self.buf[self.pos + 1] = bytes[1];
        self.pos += 2;
        Ok(())
    }

    pub fn write_u32(&mut self, value: u32) -> Result<(), ()> {
        if self.pos + 4 >= PACKET_MAXSIZE { return Err(()); }
        if self.pos + 4 >= self.len { self.len += 4; }
        let bytes = value.to_le_bytes();
        self.buf[self.pos..self.pos + 4].copy_from_slice(&bytes);
        self.pos += 4;
        Ok(())
    }

    pub fn write_f32(&mut self, value: f32) -> Result<(), ()> {
        if self.pos + 4 >= PACKET_MAXSIZE { return Err(()); }
        if self.pos + 4 >= self.len { self.len += 4; }
        let bytes = value.to_le_bytes();
        self.buf[self.pos..self.pos + 4].copy_from_slice(&bytes);
        self.pos += 4;
        Ok(())
    }

    pub fn write_f64(&mut self, value: f64) -> Result<(), ()> {
        if self.pos + 8 >= PACKET_MAXSIZE { return Err(()); }
        if self.pos + 8 >= self.len { self.len += 8; }
        let bytes = value.to_le_bytes();
        self.buf[self.pos..self.pos + 8].copy_from_slice(&bytes);
        self.pos += 8;
        Ok(())
    }

    /// A version character: its Unicode code point in a u16, so the field keeps the
    /// size the original's number had. Characters beyond U+FFFF do not fit.
    pub fn write_char(&mut self, value: char) -> Result<(), ()> {
        let code = u16::try_from(value as u32).map_err(|_| ())?;
        self.write_u16(code)
    }

    pub fn write_str(&mut self, text: &str) -> Result<(), ()> {
        for byte in text.bytes() {
            self.write_u8(byte)?;
        }
        self.write_u8(0)?;
        Ok(())
    }
}

pub fn str_unicode_len(text: &str) -> usize {
    text.chars().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_version_character_travels_as_its_code_point() {
        let mut packet = Packet::new(PacketType::IDENTITY);
        assert_eq!(packet.write_char(VERSION), Ok(()));
        packet.pos = 2;
        assert_eq!(packet.read_char(), Some(VERSION));
        assert_eq!(VERSION as u32, 0x3b1);
    }
}
