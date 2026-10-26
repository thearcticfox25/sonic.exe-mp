//! Echidna Ruins as clients show it (obj_marijuna_crystalcontroller and friends): the
//! crystals' glow and chime, the judgers' growl, the screen turning inside out around a
//! player whose controls a crystal reversed, and the statues and static once the big ring
//! is out. The server lights the crystals and wakes the judgers (sim level/echidna_ruins.rs).

use crate::client::canvas::Canvas;
use crate::core::resources::names::{sound, sprite};
use crate::core::resources::SoundId;
use crate::core::objects::ids::ObjectId;
use crate::core::player::{Character, Player};
use crate::core::world::{InstanceId, ObjectVars, World, C_WHITE};
use macroquad::rand::gen_range;
use crate::core::config::step;
use crate::core::config::{original_ticks, ticks};

/// swapControls: the circle grows from the crystal, speeding up, and shrinks around
/// the player for the last 1.5 seconds.
const CIRCLE_START_SPEED: f64 = 0.016 * 400.0;
const CIRCLE_ACCELERATION: f64 = 0.016 * 4.0;
const SHRINK_FROM_SECONDS: f64 = 1.5;
/// Draw GUI: the sad face over the player, higher for the taller characters.
const FACE_ABOVE: f64 = 20.0;
const FACE_ABOVE_TALL: f64 = 25.0;
const FACE_ABOVE_EGGMAN: f64 = 36.0;
/// obj_marijuna_statue and obj_marjiuna_static fade by this a step; the static shows at a tenth.
const FADE: f64 = 0.016;
const STATIC_ALPHA: f64 = 0.1;
/// The frame one random statue changes to when the big ring comes.
const CHOSEN_STATUE_FRAME: f64 = 2.0;
/// obj_marijuna_judger: a flame is made at depth + 1, just behind its judger (depth 150).
pub const FLAME_DEPTH: i32 = 151;
/// obj_marijuna_judger Step: the fire roars every 15 steps while the judgers breathe it.
const ROAR_EVERY_STEPS: i32 = 15;
const JUDGER_FIRING: u8 = 2;

pub struct EchidnaRuins {
    seen: Vec<(InstanceId, bool, u8)>,
    circle: (f64, f64, f64),
    speed: f64,
    /// Since the big ring came: the static's fade.
    static_fade: Option<f64>,
    /// Steps since the judgers began to breathe fire.
    firing_ticks: i32,
}

impl EchidnaRuins {
    pub fn open(world: &World) -> EchidnaRuins {
        EchidnaRuins { seen: states(world), circle: (0.0, 0.0, 0.0), speed: CIRCLE_START_SPEED, static_fade: None, firing_ticks: 0 }
    }

    /// swapControls, on this player's client.
    pub fn reversed(&mut self, x: f64, y: f64) {
        self.circle = (x, y, 0.0);
        self.speed = CIRCLE_START_SPEED;
    }

    /// Returns the sounds heard from places (measured from the left ear), and whether the
    /// judgers just began to fire, which cuts their growl short: it is longer than the 2
    /// seconds they get ready, and only the roar of the fire should be heard then.
    pub fn step(&mut self, canvas: &mut Canvas, world: &mut World, own: Option<&Player>, view: (f64, f64), big_ring_out: bool) -> (Vec<(SoundId, f64, f64)>, bool) {
        let mut heard = Vec::new();
        let mut fire_started = false;
        let now = states(world);
        for (&(id, lit, state), &(_, was_lit, was_state)) in now.iter().zip(&self.seen) {
            let (x, y) = (world.instances[id].x, world.instances[id].y);
            // SERVER_MJCRYSTAL_STATE and SERVER_MJJUDGER_STATE.
            if lit && !was_lit {
                heard.push((sound::SND_DESTINY, x, y));
            }
            if state == 1 && state != was_state {
                heard.push((sound::SND_JUDGER, x, y));
            }
            fire_started |= state == JUDGER_FIRING && was_state != JUDGER_FIRING;
            if state == JUDGER_FIRING && self.firing_ticks % original_ticks(ROAR_EVERY_STEPS) == 0 {
                heard.push((sound::SND_LAVAHIT, x, y));
            }
        }
        let firing = now.iter().any(|&(_, _, state)| state == JUDGER_FIRING);
        self.firing_ticks = if firing { self.firing_ticks + 1 } else { 0 };
        self.seen = now;

        // obj_marijuna_crystalcontroller Step
        canvas.inverse_circle = None;
        if let Some(own) = own.filter(|_| !world.game_ends) {
            let (x, y, size) = &mut self.circle;
            if own.controls_reversed > ticks(SHRINK_FROM_SECONDS) {
                if *size < world.room_height {
                    *size += self.speed * step();
                } else {
                    (*x, *y, *size) = (own.x, own.y, world.room_height);
                }
            } else {
                (*x, *y) = (own.x, own.y);
                *size = if *size > 0.0 { *size - self.speed } else { 0.0 };
            }
            self.speed += CIRCLE_ACCELERATION * step();
            canvas.inverse_circle = Some((*x - view.0, *y - view.1, *size));
        }

        // Once the big ring is out a statue changes, all fade into their other look and static comes.
        if big_ring_out && self.static_fade.is_none() {
            self.static_fade = Some(0.0);
            let statues: Vec<InstanceId> = world.ids_of(ObjectId::MarijunaStatue).collect();
            if !statues.is_empty() {
                world.instances[statues[gen_range(0, statues.len())]].image_index = CHOSEN_STATUE_FRAME;
            }
        }
        if let Some(fade) = self.static_fade.as_mut() {
            *fade = (*fade + FADE).min(1.0);
            let statues: Vec<InstanceId> = world.ids_of(ObjectId::MarijunaStatue).collect();
            for statue in statues {
                let instance = &mut world.instances[statue];
                instance.image_alpha = (instance.image_alpha - FADE).max(0.0);
            }
        }
        (heard, fire_started)
    }

    /// obj_marjiuna_static Draw
    pub fn draw_static(&self, canvas: &mut Canvas, view: (f64, f64), current_time_ms: f64) {
        if let Some(fade) = self.static_fade {
            let frame = current_time_ms * canvas.sprites.get(sprite::SPR_STATIC).fps as f64 / 1000.0;
            canvas.draw_sprite_ext(sprite::SPR_STATIC, frame, view.0, view.1, 1.0, 1.0, 0.0, C_WHITE, fade * STATIC_ALPHA);
        }
    }

    /// obj_marijuna_crystalcontroller Draw GUI: the sad face while the controls are reversed.
    pub fn draw_gui(&self, canvas: &mut Canvas, own: Option<&Player>, view: (f64, f64)) {
        let Some(own) = own.filter(|own| own.controls_reversed > 0) else { return };
        let above = match own.character {
            Character::Sally | Character::Knux | Character::Amy => FACE_ABOVE_TALL,
            Character::Eggman => FACE_ABOVE_EGGMAN,
            _ => FACE_ABOVE,
        };
        canvas.draw_sprite(sprite::SPR_MARIJUNA_BOOHOO, 0.0, (own.x - view.0).ceil(), (own.y - view.1).ceil() - above);
    }
}

/// obj_marijuna_judgerpor Draw: the judgers' flames, each facing the way it flies.
pub fn draw_flames(canvas: &mut Canvas, world: &World) {
    for flame in &world.judger_flames {
        canvas.draw_sprite_ext(sprite::SPR_MARIJUNA_FIREBALL, 0.0, flame.x, flame.y, flame.dir.signum(), 1.0, 0.0, C_WHITE, 1.0);
    }
}

/// Each crystal's and judger's state as the world shows it now.
fn states(world: &World) -> Vec<(InstanceId, bool, u8)> {
    world
        .ids_of(ObjectId::MarijunaCrystal)
        .chain(world.ids_of(ObjectId::MarijunaJudger))
        .map(|id| match world.instances[id].vars {
            ObjectVars::MjCrystal { lit, .. } => (id, lit, 0),
            ObjectVars::Judger { state } => (id, false, state),
            _ => (id, false, 0),
        })
        .collect()
}
