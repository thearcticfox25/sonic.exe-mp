//! Crouching and getting up, for every character: the crouch starts on its first frame
//! whatever frame idle had reached, and getting up plays it back to that frame.

use super::common::{greenhill_with, greenhill_with_exe, hold};
use crate::core::config::ticks;
use crate::core::game::Game;
use crate::core::player::animation::{IDLE, LOOKDOWN};
use crate::core::player::{Buttons, Character, ExeCharacter};

/// Long enough on the ground for idle to reach its looping last frames.
const IDLE_SECONDS: f64 = 6.0;

fn every_character() -> Vec<(String, Game)> {
    let survivors = [Character::Tails, Character::Knux, Character::Eggman, Character::Amy, Character::Cream, Character::Sally];
    let exes = [ExeCharacter::Original, ExeCharacter::Chaos, ExeCharacter::Exetior, ExeCharacter::Exeller];
    let survivor_games = survivors.into_iter().map(|character| (format!("{character:?}"), greenhill_with(&[character])));
    survivor_games.chain(exes.into_iter().map(|exe| (format!("{exe:?}"), greenhill_with_exe(exe)))).collect()
}

#[test]
fn every_character_crouches_from_the_first_frame_and_gets_up_through_it() {
    for (name, mut game) in every_character() {
        hold(&mut game, 0, ticks(IDLE_SECONDS) as usize);
        assert!(game.players[0].is_grounded, "{name} stands");
        hold(&mut game, Buttons::DOWN, 1);
        let player = &game.players[0];
        assert_eq!((player.state, player.image_index), (LOOKDOWN, 0.0), "{name} begins the crouch on its first frame");
        hold(&mut game, Buttons::DOWN, ticks(1.0) as usize);
        let crouched = game.players[0].image_index;
        if game.world.sprites.get(game.players[0].sprite_index).frame_count == 1 {
            // One frame (Tails, Sonic.EXE, Chaos, Exetior): nothing to play, up at once.
            hold(&mut game, 0, 1);
            assert_eq!(game.players[0].state, IDLE, "{name} stood up");
            continue;
        }
        assert!(crouched > 0.0, "{name} crouched");

        // Let go: the frames count back down, and idle shows only once they are done.
        hold(&mut game, 0, 1);
        let player = &game.players[0];
        assert_eq!(player.state, LOOKDOWN, "{name} is getting up");
        let mut previous = player.image_index;
        let mut steps = 0;
        while game.players[0].state == LOOKDOWN && steps < ticks(2.0) {
            hold(&mut game, 0, 1);
            let now = &game.players[0];
            if now.state == LOOKDOWN {
                assert!(now.image_index <= previous, "{name} does not wrap to the last frame");
                previous = now.image_index;
            }
            steps += 1;
        }
        assert_eq!(game.players[0].state, IDLE, "{name} stood up");
        assert!(steps > 1, "{name} took its time");
    }
}
