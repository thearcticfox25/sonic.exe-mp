//! scr_controls_init_gamepad and scr_controls_update_gamepad: the first connected
//! gamepad presses the keys the player bound, as the original's keyboard_key_press did,
//! so everything that reads the keyboard reads the gamepad too.

use crate::client::options::KeyBindings;
use gilrs::{Axis, Button, Gilrs};
use macroquad::input::KeyCode;
use std::collections::HashSet;

/// gamepad_set_axis_deadzone(gamepadId, 0.5): the stick counts past half way.
const STICK_DEADZONE: f32 = 0.5;

pub struct Gamepad {
    /// None where gamepads cannot be read (no backend for the system).
    gilrs: Option<Gilrs>,
}

impl Gamepad {
    pub fn new() -> Gamepad {
        let gilrs = Gilrs::new().inspect_err(|error| eprintln!("gamepads are not available: {error}")).ok();
        Gamepad { gilrs }
    }

    /// The keys the first connected gamepad holds this tick. The original turned the
    /// stick into the arrow keys; here it is the bound directions, as for the buttons.
    pub fn held_keys(&mut self, keys: &KeyBindings) -> HashSet<KeyCode> {
        let mut held = HashSet::new();
        let Some(gilrs) = self.gilrs.as_mut() else { return held };
        // The events are only drained: gilrs keeps each gamepad's state from them.
        while gilrs.next_event().is_some() {}
        let Some((_, pad)) = gilrs.gamepads().next() else { return held };

        let (x, y) = (pad.value(Axis::LeftStickX), pad.value(Axis::LeftStickY));
        if x > STICK_DEADZONE {
            held.insert(keys.right.0);
        } else if x < -STICK_DEADZONE {
            held.insert(keys.left.0);
        }
        // gilrs counts up as positive, GameMaker down.
        if y > STICK_DEADZONE {
            held.insert(keys.up.0);
        } else if y < -STICK_DEADZONE {
            held.insert(keys.down.0);
        }

        let buttons = [
            (Button::South, keys.jump.0),
            (Button::East, keys.special1.0),
            (Button::West, keys.special2.0),
            (Button::Start, KeyCode::Enter),
            (Button::LeftTrigger, keys.player_list.0),
            (Button::RightTrigger, keys.hide_gui.0),
            (Button::DPadUp, keys.emotion1.0),
            (Button::DPadRight, keys.emotion2.0),
            (Button::DPadLeft, keys.emotion2.0),
            (Button::DPadDown, keys.emotion3.0),
        ];
        held.extend(buttons.into_iter().filter(|&(button, _)| pad.is_pressed(button)).map(|(_, key)| key));
        held
    }
}
