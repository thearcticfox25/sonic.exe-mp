//! Dropping through jump-through platforms: down held, then up.

use super::common::{flat_floor_spot, hold_each, started_room};
use crate::core::config::GameplayConfig;
use crate::core::game::Game;
use crate::core::objects::ids::ObjectId;
use crate::core::player::{Buttons, Character, Player};
use crate::core::rooms::ids::RoomId;

/// Tails standing on a jump-through platform of one of the maps with nothing solid
/// under his feet, and the height of its top.
fn tails_on_a_platform() -> (Game, f64) {
    for room in [RoomId::Greenhill, RoomId::Angelisland, RoomId::Weedzone, RoomId::Volcanovalley, RoomId::Youcantrun] {
        let world = started_room(room);
        let platforms: Vec<_> = world.ids_of(ObjectId::PlatformJumptrough).collect();
        for platform in platforms {
            let Some(bbox) = world.bbox(platform) else { continue };
            if bbox.right - bbox.left < 48.0 {
                continue;
            }
            let mut game = Game::new(started_room(room));
            let x = ((bbox.left + bbox.right) / 2.0).floor();
            game.players.push(Player::new(Character::Tails, x, bbox.top - 24.0, &GameplayConfig::default()));
            hold_each(&mut game, &[0], 60);
            let tails = &game.players[0];
            if tails.is_grounded && tails.stands_on_platforms_only(&game.world) {
                let top = tails.y;
                return (game, top);
            }
        }
    }
    panic!("no map has a jump-through platform to stand on");
}

#[test]
fn down_then_up_drops_through_a_platform() {
    let (mut game, top) = tails_on_a_platform();
    hold_each(&mut game, &[Buttons::DOWN], 1);
    hold_each(&mut game, &[Buttons::DOWN | Buttons::UP], 1);
    let tails = &game.players[0];
    assert!(!tails.is_grounded && !tails.is_jumping && tails.yspd >= 0.0, "dropped, not jumped: {tails:?}");
    hold_each(&mut game, &[0], 30);
    let tails = &game.players[0];
    assert!(tails.y > top + 16.0, "fell through, from {top} to {}", tails.y);
}

#[test]
fn down_then_jump_on_a_platform_is_still_a_jump() {
    let (mut game, top) = tails_on_a_platform();
    hold_each(&mut game, &[Buttons::DOWN], 1);
    hold_each(&mut game, &[Buttons::DOWN | Buttons::A], 1);
    let tails = &game.players[0];
    assert!(tails.is_jumping && tails.yspd < 0.0 && tails.y <= top, "jumped: {tails:?}");
}

#[test]
fn solid_floor_cannot_be_dropped_through() {
    let world = started_room(RoomId::Ravinemist);
    let (x, y) = flat_floor_spot(&world);
    let mut game = Game::new(world);
    game.players.push(Player::new(Character::Tails, x, y - 24.0, &GameplayConfig::default()));
    hold_each(&mut game, &[0], 60);
    let tails = &game.players[0];
    assert!(tails.is_grounded && !tails.stands_on_platforms_only(&game.world), "on solid floor at {x}, {y}: grounded {}, only platforms {}", tails.is_grounded, tails.stands_on_platforms_only(&game.world));
    hold_each(&mut game, &[Buttons::DOWN], 1);
    hold_each(&mut game, &[Buttons::DOWN | Buttons::UP], 1);
    assert!(game.players[0].is_grounded, "still standing: {:?}", game.players[0]);
}

