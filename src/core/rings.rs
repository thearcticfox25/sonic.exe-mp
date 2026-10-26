//! Rings the round rules put on the map (obj_ring, obj_redring): each fades in,
//! then the first survivor touching it takes it, and EXE or a demon attacking
//! into a normal ring breaks it. Taking one is SERVER_RING_COLLECTED of the
//! taker's client in the original. Rings knocked out of a hurt survivor
//! (Rings.toml scatter_on_hit) are the same rings, bouncing about for a while.

use crate::core::resources::names::{sound, sprite};
use crate::core::collision::sprite_bbox;
use crate::core::config::{original_ticks, step, ticks, GameplayConfig};
use crate::core::objects::ids::ObjectId;
use crate::core::events::{SimEvent, EFFECT_SPEED};
use crate::core::player::{Character, Player};
use crate::core::world::World;

pub struct MapRing {
    pub x: f64,
    pub y: f64,
    pub red: bool,
    /// image_alpha: the ring can be touched once this reaches 1.
    pub alpha: f64,
    /// The map's ring spawner this one stands on, free again when it goes; Cream's
    /// rings stand where she laid them and have none.
    pub spawner: Option<u8>,
    /// Set for a ring knocked out of a hurt survivor.
    pub scatter: Option<Scatter>,
}

/// A ring knocked out of a hurt survivor, as in Sonic 1: thrown out in a fan, falling
/// and bouncing off the floor until its time is up.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scatter {
    pub xspd: f64,
    pub yspd: f64,
    /// Nobody can take or break it yet, so the hurt survivor does not catch it at once.
    untouchable_ticks: i32,
    ticks_left: i32,
}

/// Sonic 1's lost rings, in pixels a 60 Hz step: 16 rings to a circle starting at
/// 101.25 degrees, 22.5 degrees apart, left and right in turn; the second circle is
/// half as fast.
const SCATTER_SPEED: f64 = 4.0;
const SCATTER_FIRST_ANGLE: f64 = 101.25;
const SCATTER_ANGLE_STEP: f64 = 22.5;
const SCATTER_RINGS_A_CIRCLE: i32 = 16;
const SCATTER_GRAVITY: f64 = 0.09375;
const SCATTER_BOUNCE: f64 = 0.75;
/// The ring sprite is 16 by 16 with its origin at the top left corner.
const RING_HALF_SIZE: f64 = 8.0;
/// A ring blinks for its last second, a frame on and a frame off of the original's.
const SCATTER_BLINK_SECONDS: f64 = 1.0;
const SCATTER_BLINK_STEPS: i32 = 4;

/// What happened to a ring this tick, and to whom (an index into the players).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RingTouch {
    Taken(usize),
    Broken(usize),
}

impl MapRing {
    pub fn new(x: f64, y: f64, red: bool) -> MapRing {
        MapRing { x, y, red, alpha: 0.0, spawner: None, scatter: None }
    }

    /// `count` rings (no more than scatter_max) thrown out of a survivor hurt at (x, y).
    pub fn scattered(cfg: &GameplayConfig, x: f64, y: f64, count: i32) -> Vec<MapRing> {
        let rules = &cfg.rings;
        (0..count.min(rules.scatter_max))
            .map(|index| {
                let circle = index / SCATTER_RINGS_A_CIRCLE;
                let speed = SCATTER_SPEED / f64::from(1 << circle);
                let angle = (SCATTER_FIRST_ANGLE + SCATTER_ANGLE_STEP * f64::from((index % SCATTER_RINGS_A_CIRCLE) / 2)).to_radians();
                let side = if index % 2 == 0 { 1.0 } else { -1.0 };
                let scatter = Scatter {
                    xspd: side * angle.cos() * speed,
                    yspd: -angle.sin() * speed,
                    untouchable_ticks: ticks(rules.scatter_untouchable_seconds),
                    ticks_left: ticks(rules.scatter_seconds),
                };
                MapRing { x: x - RING_HALF_SIZE, y: y - RING_HALF_SIZE, red: false, alpha: 1.0, spawner: None, scatter: Some(scatter) }
            })
            .collect()
    }

    /// A scattered ring whose time is up, or that fell out of the room.
    pub fn is_gone(&self, world: &World) -> bool {
        self.scatter.is_some_and(|scatter| scatter.ticks_left <= 0) || self.y > world.room_height
    }

    /// A scattered ring flies, falls and bounces.
    fn fly(&mut self, world: &World) {
        let Some(scatter) = self.scatter.as_mut() else { return };
        scatter.untouchable_ticks -= 1;
        scatter.ticks_left -= 1;
        self.x += scatter.xspd * step();
        self.y += scatter.yspd * step();
        scatter.yspd += SCATTER_GRAVITY * step();
        let (middle_x, middle_y) = (self.x + RING_HALF_SIZE, self.y + RING_HALF_SIZE);
        let meets = |x: f64, y: f64, object: ObjectId| crate::core::collision::sensor_meeting(world, x, y, object);
        // Floors, jump-through platforms among them, from above.
        if scatter.yspd > 0.0 && meets(middle_x, self.y + RING_HALF_SIZE * 2.0, ObjectId::FloorParent) {
            scatter.yspd = -scatter.yspd * SCATTER_BOUNCE;
        }
        // Majin Forest's ends are the same place: a ring goes on through, as players do.
        if let Some(wrapped) = crate::core::player::majin_forest_wrap(world, middle_x, scatter.xspd) {
            self.x = wrapped - RING_HALF_SIZE;
            return self.blink();
        }
        // Solid ceilings and walls, and the room's sides, which Sonic 1's rings flew through.
        if scatter.yspd < 0.0 && meets(middle_x, self.y, ObjectId::SolidParent) {
            scatter.yspd = -scatter.yspd * SCATTER_BOUNCE;
        }
        let ahead_x = middle_x + RING_HALF_SIZE * scatter.xspd.signum();
        let out_of_room = ahead_x < 0.0 || ahead_x > world.room_width;
        if scatter.xspd != 0.0 && (out_of_room || meets(ahead_x, middle_y, ObjectId::SolidParent)) {
            scatter.xspd = -scatter.xspd * SCATTER_BOUNCE;
        }
        self.blink();
    }

    /// A scattered ring blinks out over its last second.
    fn blink(&mut self) {
        let Some(scatter) = self.scatter else { return };
        let blinking = scatter.ticks_left < ticks(SCATTER_BLINK_SECONDS);
        let shown = (scatter.ticks_left / original_ticks(SCATTER_BLINK_STEPS)) % 2 == 0;
        self.alpha = if blinking && !shown { 0.0 } else { 1.0 };
    }

    /// A ring on the map's ring spawner number `spawner`.
    pub fn on_spawner(x: f64, y: f64, red: bool, spawner: u8) -> MapRing {
        MapRing { x, y, red, alpha: 0.0, spawner: Some(spawner), scatter: None }
    }

    /// Step of obj_ring / obj_redring, checked against every player in turn as each
    /// client checked its own player.
    pub fn tick(&mut self, world: &World, cfg: &GameplayConfig, players: &[Player]) -> Option<RingTouch> {
        if let Some(scatter) = self.scatter {
            self.fly(world);
            if scatter.untouchable_ticks > 0 {
                return None;
            }
        } else if self.alpha < 1.0 {
            self.alpha += if self.red { cfg.rings.red_fade_in_per_step } else { cfg.rings.fade_in_per_step } * crate::core::config::step();
        }
        if self.alpha < 1.0 && self.scatter.is_none() {
            return None;
        }
        let ring_sprite = if self.red { sprite::SPR_REDRING } else { sprite::SPR_RING };
        let ring = sprite_bbox(world.sprites.get(ring_sprite), self.x, self.y, 1.0, 1.0, 0.0);
        players.iter().enumerate().filter(|(_, player)| !player.removed).find_map(|(index, player)| {
            let body = sprite_bbox(world.sprites.get(player.sprite_index), player.x, player.y, player.image_xscale, 1.0, 0.0);
            if !ring.overlaps(&body) {
                return None;
            }
            let demon = player.character == Character::Exe || player.revival_times >= 2;
            if self.red {
                let can_take = player.character != Character::Exe && player.red_ring_timer <= 0 && player.hp > 0 && player.revival_times < 2;
                return can_take.then_some(RingTouch::Taken(index));
            }
            if demon {
                let breaks = !matches!(player.character, Character::Tails | Character::Cream) && player.is_attacking;
                return breaks.then_some(RingTouch::Broken(index));
            }
            (player.hp > 0).then_some(RingTouch::Taken(index))
        })
    }
}

impl MapRing {
    /// obj_ring Step: a shot of demonized Tails breaks an ordinary ring it touches, for
    /// the one who fired it. Red rings have a Step of their own without this.
    pub fn broken_by_shot(&self, world: &World, shots: &[crate::core::tails_projectile::TailsProjectile]) -> Option<usize> {
        let still_untouchable = self.scatter.is_some_and(|scatter| scatter.untouchable_ticks > 0);
        if self.red || (self.alpha < 1.0 && self.scatter.is_none()) || still_untouchable {
            return None;
        }
        let ring = sprite_bbox(world.sprites.get(sprite::SPR_RING), self.x, self.y, 1.0, 1.0, 0.0);
        shots.iter().find(|shot| shot.hurts_survivors && shot.bbox(world).overlaps(&ring)).map(|shot| shot.owner)
    }
}

impl Player {
    /// SERVER_RING_COLLECTED: a ring (or a red ring) reached this player.
    /// `can_heal`: the server allows healing with rings.
    pub fn take_ring(&mut self, world: &World, cfg: &GameplayConfig, red: bool, can_heal: bool, events: &mut Vec<SimEvent>) {
        if self.character == Character::Exe || self.revival_times >= 2 {
            return;
        }
        let rules = &cfg.rings;
        if red {
            if self.red_ring_timer > 0 {
                return;
            }
            events.push(SimEvent::Sound { sound: sound::SND_REDRING, x: self.x, y: self.y });
            events.push(SimEvent::RedRingStarted);
            self.red_ring_timer = crate::core::config::ticks(rules.red_ring_seconds);
            return;
        }
        self.rings += 1;
        self.shorten_red_ring(world, cfg);
        if self.rings >= rules.rings_to_heal && self.hp < cfg.hurt.max_hp && can_heal {
            self.rings = 0;
            self.hp += cfg.hurt.heal_hp;
            events.push(SimEvent::Sound { sound: sound::SND_HEAL, x: self.x, y: self.y });
            events.push(SimEvent::RingsHealed);
        } else {
            events.push(SimEvent::Sound { sound: sound::SND_RING, x: self.x, y: self.y });
        }
    }

    /// SERVER_VVVASE_STATE: the rings of a vase this player broke. Unlike a ring, healing
    /// spends only the rings it needs and the ring sound plays too.
    pub fn take_vase_rings(&mut self, world: &World, cfg: &GameplayConfig, rings: i32, events: &mut Vec<SimEvent>) {
        if self.is_dead {
            return;
        }
        events.push(SimEvent::Sound { sound: sound::SND_RING, x: self.x, y: self.y });
        events.push(SimEvent::Effect { sprite: sprite::SPR_RING_SPARKLE, x: self.x, y: self.y, xscale: 1.0, image_speed: EFFECT_SPEED, yspd: 0.0 });
        self.rings += rings;
        self.shorten_red_ring(world, cfg);
        // The server's canHeal is whether the breaker has rings, which it always has by now.
        if self.rings >= cfg.rings.rings_to_heal && self.hp < cfg.hurt.max_hp {
            self.rings -= cfg.rings.rings_to_heal;
            self.hp += cfg.hurt.heal_hp;
            events.push(SimEvent::Sound { sound: sound::SND_HEAL, x: self.x, y: self.y });
            events.push(SimEvent::RingsHealed);
        }
    }

    /// Every ring taken under a red ring shortens it.
    fn shorten_red_ring(&mut self, world: &World, cfg: &GameplayConfig) {
        let rules = &cfg.rings;
        let min_ticks = crate::core::config::ticks(rules.red_ring_min_seconds);
        if self.red_ring_timer >= min_ticks {
            let last_minute = world.exit_ring_out;
            let shortening = if last_minute { rules.last_minute_red_ring_shortening_seconds } else { rules.red_ring_shortening_seconds };
            let shortening = shortening * crate::core::config::ticks_per_second();
            self.red_ring_timer = ((self.red_ring_timer as f64 - shortening) as i32).max(min_ticks);
        }
    }

    /// ringBreak of obj_ring: the ring shatters under an attack.
    pub fn break_ring(&self, ring: &MapRing, events: &mut Vec<SimEvent>) {
        events.push(SimEvent::Sound { sound: sound::SND_EXE_RINGSHUTTER, x: self.x, y: self.y });
        events.push(SimEvent::RingBroke { x: ring.x, y: ring.y, xspd: self.xspd });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::resources::sprites::Sprites;
    use std::path::Path;
    use std::sync::Arc;

    fn world() -> World {
        let textures = Path::new(env!("CARGO_MANIFEST_DIR")).join("Resources/Textures");
        World::empty(Arc::new(Sprites::index(&textures).unwrap()), Arc::new(GameplayConfig::default()))
    }

    #[test]
    fn a_ring_fades_in_then_goes_to_the_survivor_and_breaks_under_an_attack() {
        let (world, cfg) = (world(), GameplayConfig::default());
        let tails = Player::new(Character::Tails, 100.0, 100.0, &cfg);
        let mut ring = MapRing::new(100.0, 100.0, false);
        let touches: Vec<Option<RingTouch>> = (0..31).map(|_| ring.tick(&world, &cfg, std::slice::from_ref(&tails))).collect();
        assert!(touches[..30].iter().all(Option::is_none), "still fading in for 30 ticks (the float sum stays below 1)");
        assert_eq!(touches[30], Some(RingTouch::Taken(0)));

        let mut exe = Player::new_exe(crate::core::player::ExeCharacter::Original, 100.0, 100.0, &cfg);
        assert_eq!(ring.tick(&world, &cfg, std::slice::from_ref(&exe)), None, "EXE does not take rings");
        exe.is_attacking = true;
        assert_eq!(ring.tick(&world, &cfg, &[exe]), Some(RingTouch::Broken(0)));
    }

    #[test]
    fn a_scattered_ring_bounces_back_off_the_side_of_the_room() {
        let (mut world, cfg) = (world(), GameplayConfig::default());
        (world.room_width, world.room_height) = (1024.0, 1024.0);
        let mut ring = MapRing::scattered(&cfg, 12.0, 512.0, 2).remove(0);
        assert!(ring.scatter.unwrap().xspd < 0.0, "the first ring flies left");
        for _ in 0..8 {
            ring.fly(&world);
        }
        assert!(ring.scatter.unwrap().xspd > 0.0 && ring.x >= -RING_HALF_SIZE, "and comes back: x {}", ring.x);
    }

    #[test]
    fn a_shot_of_demonized_tails_breaks_an_ordinary_ring_only() {
        let (world, cfg) = (world(), GameplayConfig::default());
        let mut ring = MapRing::new(100.0, 100.0, false);
        ring.alpha = 1.0;
        let demon_shot = crate::core::tails_projectile::TailsProjectile::new(&cfg, 3, 100.0, 100.0, 1.0, 1, true, 0);
        let survivor_shot = crate::core::tails_projectile::TailsProjectile::new(&cfg, 4, 100.0, 100.0, 1.0, 1, false, 0);
        assert_eq!(ring.broken_by_shot(&world, std::slice::from_ref(&survivor_shot)), None, "a survivor's shot leaves rings alone");
        assert_eq!(ring.broken_by_shot(&world, &[survivor_shot, demon_shot.clone()]), Some(3), "broken for the shooter");
        let mut red = MapRing::new(100.0, 100.0, true);
        red.alpha = 1.0;
        assert_eq!(red.broken_by_shot(&world, &[demon_shot]), None, "red rings have no such Step");
    }

    #[test]
    fn ten_rings_heal_a_hurt_survivor() {
        let (mut world, cfg) = (world(), GameplayConfig::default());
        let mut tails = Player::new(Character::Tails, 0.0, 0.0, &cfg);
        tails.hp = cfg.hurt.max_hp - cfg.hurt.heal_hp;
        tails.rings = 9;
        let mut events = Vec::new();
        tails.take_ring(&world, &cfg, false, true, &mut events);
        assert_eq!((tails.rings, tails.hp), (0, cfg.hurt.max_hp));
        assert!(events.contains(&SimEvent::RingsHealed));

        tails.red_ring_timer = 100;
        tails.take_ring(&world, &cfg, false, true, &mut Vec::new());
        assert_eq!(tails.red_ring_timer, 70, "a ring shortens the red ring by half a second");
    }
}
