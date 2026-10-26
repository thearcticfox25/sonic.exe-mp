//! Echidna Ruins' crystals (the server's mj_ass) and judgers (mj_judger). The lava
//! platform is lava.rs.

use super::Random;
use crate::core::config::{original_ticks, step, ticks, EchidnaRuinsRules};
use crate::core::objects::ids::ObjectId;
use crate::core::world::{ObjectVars, World};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EchidnaRuins {
    crystal_timer: i32,
    crystal_wait: i32,
    lit: bool,
    glow: f64,
    judger_timer: i32,
    judger_wait: i32,
    judger: Judger,
    flames: Vec<JudgerFlame>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
enum Judger {
    Waiting = 0,
    Ready = 1,
    Firing = 2,
}

/// obj_marijuna_judger Step: while firing, a judger breathes a flame every 6 steps
/// from in front of its mouth.
const FLAME_EVERY_STEPS: i32 = 6;
const FLAME_MOUTH_RIGHT: f64 = 30.0;
const FLAME_MOUTH_LEFT: f64 = -40.0;
const FLAME_MOUTH_UP: f64 = 4.0;
/// obj_marijuna_judgerpor: a flame flies 4 pixels a step the way its judger faces,
/// drifts up or down by up to 0.3 a step and is gone after 30 to 50 steps.
const FLAME_SPEED: f64 = 4.0;
const FLAME_DRIFT_THOUSANDTHS: u32 = 300;
const FLAME_LIFE_STEPS: i32 = 30;
const FLAME_LIFE_EXTRA_STEPS: u32 = 20;

/// obj_marijuna_judgerpor: one flame, which burns whoever it touches (player/echidna_ruins.rs).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct JudgerFlame {
    pub x: f64,
    pub y: f64,
    /// The judger's image_xscale: the way the flame flies and faces.
    pub dir: f64,
    drift: f64,
    ticks_left: i32,
}

/// What clients are told: whether the crystals are lit and how much they glow, the
/// judgers' state (0 waiting, 1 ready, 2 firing) and their flames.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EchidnaRuinsView {
    pub lit: bool,
    pub glow: f32,
    pub judger: u8,
    pub flames: Vec<JudgerFlame>,
}

impl EchidnaRuins {
    pub(super) fn new(rules: &EchidnaRuinsRules, random: &mut Random) -> EchidnaRuins {
        EchidnaRuins {
            crystal_timer: 0,
            crystal_wait: ticks(rules.crystal_seconds),
            lit: false,
            glow: 0.0,
            judger_timer: 0,
            judger_wait: ticks(rules.judger_wait_seconds + random.below(rules.judger_wait_extra_seconds) as f64),
            judger: Judger::Waiting,
            flames: Vec::new(),
        }
    }

    pub(super) fn tick(&mut self, rules: &EchidnaRuinsRules, world: &World, random: &mut Random) {
        // mj_ass
        if self.crystal_timer >= self.crystal_wait {
            self.lit = !self.lit;
            self.crystal_timer = 0;
            self.crystal_wait = ticks(rules.crystal_seconds + random.below(rules.crystal_extra_seconds) as f64);
        }
        self.crystal_timer += 1;
        // obj_marijuna_crystal Step
        let glow_step = rules.crystal_glow_per_step * step();
        self.glow = if self.lit { (self.glow + glow_step).min(1.0) } else { (self.glow - glow_step).max(0.0) };
        // mj_judger
        if self.judger_timer >= self.judger_wait {
            (self.judger, self.judger_wait) = match self.judger {
                Judger::Waiting => (Judger::Ready, ticks(rules.judger_ready_seconds)),
                Judger::Ready => (Judger::Firing, ticks(rules.judger_fire_seconds + random.below(rules.judger_fire_extra_seconds) as f64)),
                Judger::Firing => (Judger::Waiting, ticks(rules.judger_wait_seconds + random.below(rules.judger_wait_extra_seconds) as f64)),
            };
            self.judger_timer = 0;
        }
        if self.judger == Judger::Firing && self.judger_timer % original_ticks(FLAME_EVERY_STEPS) == 0 {
            self.breathe_flames(world, random);
        }
        self.judger_timer += 1;
        self.move_flames();
    }

    fn breathe_flames(&mut self, world: &World, random: &mut Random) {
        for judger in world.ids_of(ObjectId::MarijunaJudger) {
            let instance = &world.instances[judger];
            let mouth = if instance.image_xscale > 0.0 { FLAME_MOUTH_RIGHT } else { FLAME_MOUTH_LEFT };
            // random_range(-0.3, 0.3), to a thousandth.
            let drift = (random.below(FLAME_DRIFT_THOUSANDTHS * 2 + 1) as f64 - FLAME_DRIFT_THOUSANDTHS as f64) / 1000.0;
            let ticks_left = original_ticks(FLAME_LIFE_STEPS + random.below(FLAME_LIFE_EXTRA_STEPS) as i32);
            self.flames.push(JudgerFlame { x: instance.x + mouth, y: instance.y - FLAME_MOUTH_UP, dir: instance.image_xscale, drift, ticks_left });
        }
    }

    fn move_flames(&mut self) {
        for flame in &mut self.flames {
            flame.x += flame.dir * FLAME_SPEED * step();
            flame.y += flame.drift * step();
            flame.ticks_left -= 1;
        }
        self.flames.retain(|flame| flame.ticks_left > 0);
    }

    pub(super) fn view(&self) -> EchidnaRuinsView {
        EchidnaRuinsView { lit: self.lit, glow: self.glow as f32, judger: self.judger as u8, flames: self.flames.clone() }
    }
}

pub(super) fn apply(world: &mut World, view: &EchidnaRuinsView) {
    let (lit, glow, state) = (view.lit, view.glow, view.judger);
    world.judger_flames = view.flames.clone();
    let ids: Vec<_> = world.ids_of(ObjectId::MarijunaCrystal).chain(world.ids_of(ObjectId::MarijunaJudger)).collect();
    for id in ids {
        let instance = &mut world.instances[id];
        match instance.vars {
            ObjectVars::MjCrystal { .. } => instance.vars = ObjectVars::MjCrystal { lit, glow: glow as f64 },
            // The original read SERVER_MJJUDGER_STATE as a bool, so firing looked like getting
            // ready and the judgers never breathed fire; the whole state is passed on here.
            ObjectVars::Judger { .. } => {
                instance.vars = ObjectVars::Judger { state };
                instance.image_index = if state > 0 { 1.0 } else { 0.0 };
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::rooms::ids::RoomId;

    #[test]
    fn firing_judgers_breathe_flames_that_fly_ahead_and_die_out() {
        let world = crate::core::tests::common::started_room(RoomId::Marijuna);
        let rules = EchidnaRuinsRules::default();
        let mut random = Random::new(7);
        let mut ruins = EchidnaRuins::new(&rules, &mut random);
        while ruins.judger != Judger::Firing {
            ruins.tick(&rules, &world, &mut random);
        }
        assert_eq!(ruins.flames.len(), world.ids_of(ObjectId::MarijunaJudger).count(), "a flame from every judger at once");
        let first = ruins.flames[0];
        ruins.tick(&rules, &world, &mut random);
        assert!((ruins.flames[0].x - first.x) * first.dir > 0.0, "a flame flies the way its judger faces");
        while ruins.judger == Judger::Firing {
            ruins.tick(&rules, &world, &mut random);
        }
        for _ in 0..original_ticks(FLAME_LIFE_STEPS + FLAME_LIFE_EXTRA_STEPS as i32) {
            ruins.tick(&rules, &world, &mut random);
        }
        assert!(ruins.flames.is_empty(), "the fire is out once the judgers stop");
    }
}
