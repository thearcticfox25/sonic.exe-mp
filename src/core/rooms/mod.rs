pub mod ids;

use ids::RoomId;

/// The playable levels by the number the server and the protocol use (global.levels).
pub const LEVEL_ROOMS: [RoomId; 22] = [
    // Map 0, the cut level of the 12.2022 alpha: the maps count from 1 everywhere a
    // player or a server owner sees them, and only "0" written on purpose reaches it.
    RoomId::Level1,
    RoomId::Hideandseek2,
    RoomId::Ravinemist,
    RoomId::Dotdotdot,
    RoomId::Deserttown,
    RoomId::Youcantrun,
    RoomId::Limpcity,
    RoomId::Notperfect,
    RoomId::Kindandfair,
    RoomId::Act9,
    RoomId::Nastyparadise,
    RoomId::Pricelessfreedom,
    RoomId::Volcanovalley,
    RoomId::Greenhill,
    RoomId::MajinForest,
    RoomId::Angelisland,
    RoomId::Torturecave,
    RoomId::Dartower,
    RoomId::Haundream,
    RoomId::Fartzone,
    RoomId::Weedzone,
    RoomId::Marijuna,
];

/// Fart Zone, the practice map that .practice votes for. It stands before Mystic Wood
/// and Echidna Ruins, where the versions that had it before them put it.
pub const PRACTICE_LEVEL: u8 = 19;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_practice_level_is_fart_zone() {
        assert_eq!(LEVEL_ROOMS[PRACTICE_LEVEL as usize], RoomId::Fartzone);
    }
}
