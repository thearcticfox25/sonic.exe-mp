//! Every room of the game: the screens, then the playable levels in the order of their
//! map numbers (rooms/mod.rs LEVEL_ROOMS). Each room's layout is a Tiled map: a level's
//! is Maps/<file>.tmj, a screen's Menus/<file>.tmj.

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum RoomId {
    Logo,
    Menu,
    Lobby,
    Waiting,
    Message,
    Connecting,
    Results,

    Level1,
    Hideandseek2,
    Ravinemist,
    Dotdotdot,
    Deserttown,
    Youcantrun,
    Limpcity,
    Notperfect,
    Kindandfair,
    Act9,
    Nastyparadise,
    Pricelessfreedom,
    Volcanovalley,
    Greenhill,
    MajinForest,
    Angelisland,
    Torturecave,
    Dartower,
    Haundream,
    Fartzone,
    Weedzone,
    Marijuna,
}

impl RoomId {
    /// The room's Tiled map inside the resources folder.
    pub fn map_path(self, resources: &std::path::Path) -> std::path::PathBuf {
        use crate::core::resources::{MAPS, MENUS};
        let folder = if super::LEVEL_ROOMS.contains(&self) { MAPS } else { MENUS };
        resources.join(folder).join(format!("{}.tmj", self.file_name()))
    }

    /// Map file name inside Maps/, without extension.
    pub fn file_name(self) -> &'static str {
        match self {
            RoomId::Logo => "logo",
            RoomId::Menu => "menu",
            RoomId::Lobby => "lobby",
            RoomId::Waiting => "waiting",
            RoomId::Message => "message",
            RoomId::Connecting => "connecting",
            RoomId::Results => "results",
            RoomId::Level1 => "level1",
            RoomId::Hideandseek2 => "hideandseek2",
            RoomId::Ravinemist => "ravinemist",
            RoomId::Dotdotdot => "dotdotdot",
            RoomId::Deserttown => "deserttown",
            RoomId::Youcantrun => "youcantrun",
            RoomId::Limpcity => "limpcity",
            RoomId::Notperfect => "notperfect",
            RoomId::Kindandfair => "kindandfair",
            RoomId::Act9 => "act9",
            RoomId::Nastyparadise => "nastyparadise",
            RoomId::Pricelessfreedom => "pricelessfreedom",
            RoomId::Volcanovalley => "volcanovalley",
            RoomId::Greenhill => "greenhill",
            RoomId::MajinForest => "majinforest",
            RoomId::Angelisland => "angelisland",
            RoomId::Torturecave => "torturecave",
            RoomId::Dartower => "dartower",
            RoomId::Haundream => "haundream",
            RoomId::Fartzone => "fartzone",
            RoomId::Weedzone => "weedzone",
            RoomId::Marijuna => "marijuna",
        }
    }
}
