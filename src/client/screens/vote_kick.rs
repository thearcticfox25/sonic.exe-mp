//! obj_lobby_votekick and obj_menu_waitkick: picking a player with left and right, for
//! a vote kick or, for operators, a kick, ban or operator rights. The lobby and the
//! waiting room keep their own list of who is there, so the list is passed in. The
//! server's ?gui window picks with it too, for its own list of actions (host_actions.rs).

use crate::client::canvas::Canvas;
use crate::client::Context;
use crate::packet::{Packet, PacketType};
use crate::core::resources::names::sprite;

const ARROW_ROW_LEFT: f64 = 40.0;
const ARROW_ROW_STEP: f64 = 80.0;
const ARROW_Y: f64 = 58.0;
const ARROW_FRAME_MILLISECONDS: f64 = 300.0;

pub struct PlayerPicker {
    /// What the pick is for (packet::CHOOSE_*), sent back with the picked player; None
    /// for the ?gui window, which is told the pick instead and never closes the picker.
    answer: Option<u8>,
    /// ind: position among the other players.
    choice: usize,
}

/// What the picker did this step.
pub enum PickerOutcome {
    StillPicking,
    Closed,
    /// The ?gui window picked this player.
    Picked(u16),
}

impl PlayerPicker {
    /// `action` is what the server's LOBBY_CHOOSE_PLAYER asked for.
    pub fn new(action: u8) -> PlayerPicker {
        PlayerPicker { answer: Some(action), choice: 0 }
    }

    /// The server's ?gui window's picker, which stays open.
    pub fn for_host_window() -> PlayerPicker {
        PlayerPicker { answer: None, choice: 0 }
    }

    /// Draw_0: Z sends the pick, X cancels; the arrow marks the choice. `others` is
    /// everyone who can be picked, in the order their icons stand on the screen.
    pub fn draw(&mut self, context: &mut Context, others: &[u16], canvas_time_ms: f64) -> PickerOutcome {
        let picked = others.get(self.choice).copied();
        let keys = &context.options.keys;
        if let Some(player) = picked {
            if context.input.pressed(keys.jump.0) {
                let Some(answer) = self.answer else { return PickerOutcome::Picked(player) };
                let mut packet = Packet::new(PacketType::LOBBY_CHOOSE_PLAYER);
                let _ = packet.write_u8(answer);
                let _ = packet.write_u16(player);
                context.net.send_reliable(&packet);
                return PickerOutcome::Closed;
            }
            if context.input.pressed(keys.special1.0) && self.answer.is_some() {
                return PickerOutcome::Closed;
            }
        }
        if others.is_empty() {
            return if self.answer.is_some() { PickerOutcome::Closed } else { PickerOutcome::StillPicking };
        }
        let mut choice = self.choice as i32;
        if context.input.pressed(keys.left.0) {
            choice -= 1;
        }
        if context.input.pressed(keys.right.0) {
            choice += 1;
        }
        let count = others.len() as i32;
        if choice < 0 {
            choice = count - 1;
        } else if choice >= count {
            choice = 0;
        }
        self.choice = choice as usize;
        draw_arrow(&mut context.canvas, self.choice, canvas_time_ms);
        PickerOutcome::StillPicking
    }
}

fn draw_arrow(canvas: &mut Canvas, choice: usize, current_time_ms: f64) {
    let x = ARROW_ROW_LEFT + choice as f64 * ARROW_ROW_STEP;
    canvas.draw_sprite(sprite::SPR_LOBBY_ICON_ARROW, current_time_ms / ARROW_FRAME_MILLISECONDS, x, ARROW_Y);
}
