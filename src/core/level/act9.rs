//! Act 9's walls (the server's act9wall): as the round's time runs out the ceiling
//! comes down and the side walls close in, until nothing is left of the arena.

use crate::core::objects::ids::ObjectId;
use crate::core::world::{ObjectVars, World};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Act9Walls {
    /// The round's time when the walls started, in ticks.
    start: Option<i64>,
    /// The round's clock counts whole seconds; the walls move between its changes too,
    /// by the ticks since it last changed (at most a second's worth, so a stopped clock
    /// stops them).
    clock_seen: i64,
    ticks_since_clock_changed: i64,
}

impl Act9Walls {
    /// How far the walls have closed, 0 to 1. They start with the round's clock: the
    /// level ticks while the players are still loading, when the clock is not set yet
    /// (0), and starting from that left the walls open the whole round.
    pub(super) fn tick(&mut self, world: &World) -> f32 {
        // Without a clock the walls never start.
        let Some(ticks_left) = world.timer_ticks else { return 0.0 };
        if self.start.is_none() && world.round_started && ticks_left > 0 {
            self.start = Some(ticks_left);
        }
        if ticks_left == self.clock_seen {
            self.ticks_since_clock_changed += 1;
        } else {
            (self.clock_seen, self.ticks_since_clock_changed) = (ticks_left, 0);
        }
        let Some(start) = self.start else { return 0.0 };
        let within_second = self.ticks_since_clock_changed.min(crate::core::config::ticks(1.0) as i64);
        ((start - ticks_left + within_second) as f64 / start as f64).clamp(0.0, 1.0) as f32
    }
}

/// SERVER_ACT9WALL_STATE: each wall's edge at its share of the way.
pub(super) fn apply(world: &mut World, closed: f32) {
    let rules = world.config.levels.act9.clone();
    let room_width = world.room_width;
    let walls: Vec<_> = world.ids_of(ObjectId::Act9Wall).collect();
    for wall in walls {
        let ObjectVars::Act9Wall { nid } = world.instances[wall].vars else { continue };
        let ceiling = rules.ceiling_travel * closed as f64;
        let side = rules.side_travel * closed as f64;
        let instance = &mut world.instances[wall];
        let (x, y) = match nid {
            0 => (-WALL_THICKNESS, ceiling - CEILING_THICKNESS),
            1 => (side - WALL_THICKNESS, 0.0),
            _ => (room_width - side, 0.0),
        };
        (instance.x, instance.y) = (x, y);
    }
}

/// The walls are this thick (their scaled sprite in the map).
const WALL_THICKNESS: f64 = 2240.0;
const CEILING_THICKNESS: f64 = 768.0;
