pub mod ravine_mist;
pub mod desert_town;
pub mod hide_and_seek;

use crate::server::{BigRingState, OutboxMsg, Server};

/// The round most maps have: 180 s plus 20 s for every player after the first, ring
/// coefficient 5 (the original's map_init).
const DEFAULT_TIME: (usize, usize) = (180, 20);
const DEFAULT_RINGS: u8 = 5;

pub struct MapDef {
    pub name: &'static str,
    /// Round time: base seconds, plus seconds for every in-game player after the first.
    pub time: (usize, usize),
    /// Ring coefficient (map_ring).
    pub rings: u8,
    pub tick: fn(&mut Server, &mut Vec<OutboxMsg>),
    /// What the map vote writes under the cards (global.levelDescriptions), "\n" between
    /// its lines. Fart Zone had none in the original.
    pub description: &'static str,
}

pub static MAP_LIST: [MapDef; 22] = [
    MapDef { name: "Level 1",           time: DEFAULT_TIME, rings: DEFAULT_RINGS, tick: map_tick,
        description: "unreleased map ported from closed beta testing version of original TD2D" },
    MapDef { name: "Hide and Seek 2",   time: DEFAULT_TIME, rings: DEFAULT_RINGS, tick: map_tick,
        description: "long and spacious map with no less spacious boxes to hide behind, using       \nsprings echoes all over the cave, revealing your location to exe |+25 sec." },
    MapDef { name: "Ravine Mist",       time: (180, 20),    rings: 5,             tick: ravine_mist::rmz_tick,
        description: "6 shards have to be collected to access the exit on the last minute, collected\nshards will lead you to the exit. avoid slugs that feed on rings, kill them  \nto obtain the ring if they devour one" },
    MapDef { name: "...",               time: (205, 20),    rings: 5,             tick: map_tick,
        description: "long stairway with three floors, you can't run on stairs and the lower you go \n- the darker it gets |+25 sec." },
    MapDef { name: "Desert Town",       time: DEFAULT_TIME, rings: DEFAULT_RINGS, tick: map_tick,
        description: "high buildings will serve you a hideout, it's hard to break away here, but you\ncan hide almost everywhere" },
    MapDef { name: "You Can't Run",     time: (180, 20),    rings: 5,             tick: map_tick,
        description: "constant gas leaks will fill the areas poisoning you and causing you to       \nhallucinate, revealing your location to exe " },
    MapDef { name: "Limp City",         time: (155, 20),    rings: 5,             tick: map_tick,
        description: "the city is limp but not blind, you can be watched through eye cameras or use \nthem yourself, there are the circuits that periodically activate blocking the \npassages |-25 sec." },
    MapDef { name: "Not Perfect",       time: DEFAULT_TIME, rings: DEFAULT_RINGS, tick: map_tick,
        description: "map rotates every 30 seconds, the floor becomes walls and the walls become the\nfloor, the exit is available on only one turn |-25 sec." },
    MapDef { name: "Kind and Fair",     time: (180, 20),    rings: 5,             tick: map_tick,
        description: "avoid breaking speed monitors, as they will launch you right in the spikes    \nwithout letting you stop running until you meet a wall or get hurt" },
    MapDef { name: "Act 9",             time: (130, 10),    rings: 3,             tick: map_tick,
        description: "you can see only the silhouettes, the walls are slowly shrickening, avoid     \ngetting cornered if you don't want to be squished |-50 sec. +10 sec. for each\nsurvivor instead of 25" },
    MapDef { name: "Nasty Paradise",    time: (155, 20),    rings: 5,             tick: map_tick,
        description: "giant snowballs roll down the slopes, damaging and slowing you down.\nsometimes springs and rings cover with ice |-25 sec." },
    MapDef { name: "Priceless Freedom", time: (155, 10),    rings: 5,             tick: map_tick,
        description: "escape with elevators and avoid black rings that will drain your health upon  \ngetting them, damage from them can be avoided by having 5 rings |-25 sec. +10 \nsec. for each survivor instead of 25" },
    MapDef { name: "Volcano Valley",    time: (180, 20),    rings: 5,             tick: map_tick,
        description: "long map filled with lava pits, some of which burst with lava pillars. break  \nthe pots to get more rings" },
    MapDef { name: "Hill",              time: DEFAULT_TIME, rings: DEFAULT_RINGS, tick: map_tick,
        description: "long linear map filled with water that slows you down. when the lightning     \nstrikes, don't touch the water" },
    MapDef { name: "Majin Forest",      time: (155, 10),    rings: 3,             tick: map_tick,
        description: "wherever you run - you will never reach the end, there are no bounds, fun is   \ninfinite |-25 sec. +10 sec. for each survivor instead of 25" },
    MapDef { name: "Hide and Seek",     time: DEFAULT_TIME, rings: DEFAULT_RINGS, tick: map_tick,
        description: "hide behind palm trees and bushes, moving along the ziplines" },
    MapDef { name: "Torture Cave",      time: DEFAULT_TIME, rings: DEFAULT_RINGS, tick: map_tick,
        description: "an abandoned mine illuminated only by searchlights, sometimes acid evaporates \nincreasing its area, stay away from it unless you're a fan of sulfuric acid  \nbaths |-25 sec." },
    MapDef { name: "Dark Tower",        time: (205, 20),    rings: 5,             tick: map_tick,
        description: "don't stand under stalactites, they will fall if you walk under them. every 10 \nseconds it gets dark for a while, at this time try not to run into the owner  \nof this place |+25 sec." },
    MapDef { name: "Haunting Dream",    time: (205, 20),    rings: 5,             tick: map_tick,
        description: "you will be able to open the passages by activating the crystals. hold the jump  \nor up on special springs so that they throw you higher, and most importantly, \ndo not wake up |+25 sec." },
    MapDef { name: "Fart Zone",         time: (256, 20),    rings: 1,             tick: map_tick,
        description: "map to practice, have fun and test 3rd party textures" },
    MapDef { name: "Mystic Wood",       time: (155, 20),    rings: 5,             tick: map_tick,
        description: "be careful on conveyor belts and get to ignited lanterns to banish spirit that \nhaunts the forest, before things get even darker |-25 sec" },
    MapDef { name: "Echidna Ruins",     time: (205, 20),    rings: 5,             tick: map_tick,
        description: "you better not touch the crystals releasing the phantom energy, otherwise it \nwill be difficult for you to move and avoid traps of the ancient temple \n|+25 sec" },
];

/// The highest map number shown to players: the maps they know are 1 to this. Map 0,
/// the cut level of the 12.2022 alpha, is left out of everything a player reads, as if
/// it were not there: only "0" written on purpose (.map 0, :vm 0, --map 0) reaches it,
/// or a server whose map_list turns it on.
pub const LAST_MAP: usize = MAP_LIST.len() - 1;

/// The map a player's number names (the map ids themselves, 1 to LAST_MAP, and 0 unsaid).
pub fn map_by_number(number: i32) -> Option<usize> {
    usize::try_from(number).ok().filter(|&map| map < MAP_LIST.len())
}

/// Sets up the round of map `map_idx`, or the default round for an unknown index.
pub fn map_init(server: &mut Server, map_idx: usize) {
    let (time, rings) = MAP_LIST.get(map_idx).map_or((DEFAULT_TIME, DEFAULT_RINGS), |map| (map.time, map.rings));
    map_time_ex(server, time.0, time.1);
    map_ring(server, rings);
    server.game.bring_state = BigRingState::None;
}

pub fn map_time_ex(server: &mut Server, base_sec: usize, mul_sec: usize) {
    let cfg = crate::config::cfg();
    let time_sec = if cfg.states.gameplay.banana.disable_timer {
        0u16
    } else {
        let ingame = server.ingame_count();
        (base_sec + ingame.saturating_sub(1) * mul_sec).min(9999) as u16
    };
    server.game.time_sec = time_sec;
    server.game.time = crate::states::seconds(time_sec as f64);
}

pub fn map_tick(_server: &mut Server, _outbox: &mut Vec<OutboxMsg>) {
}

pub fn map_ring(server: &mut Server, ring_coff: u8) {
    let ingame = server.ingame_count();
    let adjusted = if ingame > 3 && ring_coff > 1 { ring_coff - 1 } else { ring_coff };
    server.game.ring_coff = adjusted.max(1);
}
