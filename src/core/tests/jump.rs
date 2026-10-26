//! The variable jump (Physics.toml variable_jump): a short press jumps lower.

use super::common::{greenhill_with, hold};
use crate::core::player::{Buttons, Character};

/// How far above its start the player rises when jump is held for `held_ticks`.
fn jump_height(character: Character, held_ticks: usize) -> f64 {
    let mut game = greenhill_with(&[character]);
    hold(&mut game, 0, 120);
    let start = game.players[0].y;
    let mut top = start;
    for tick in 0..120 {
        hold(&mut game, if tick < held_ticks { Buttons::A } else { 0 }, 1);
        top = top.min(game.players[0].y);
    }
    start - top
}

#[test]
fn a_tap_jumps_lower_than_a_held_jump() {
    // Amy too, whose jump pose ends a few steps into the jump.
    for character in [Character::Sally, Character::Amy] {
        let (tap, held) = (jump_height(character, 8), jump_height(character, 60));
        assert!(tap < held * 0.75, "{character:?}: tap {tap} against held {held}");
    }
}

