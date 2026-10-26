//! Level 1, the cut level of the 12.2022 alpha (map 0): a room that loads and holds
//! the players up, with springs that throw them.

use super::common::{hold, started_room};
use crate::core::config::{ticks, GameplayConfig};
use crate::core::game::Game;
use crate::core::objects::ids::ObjectId;
use crate::core::player::{Character, Player};
use crate::core::rooms::ids::RoomId;

#[test]
fn every_spawn_point_stands_on_floor() {
    let world = started_room(RoomId::Level1);
    let spawns: Vec<(f64, f64)> = world.ids_of(ObjectId::Spawnpoint).map(|id| (world.instances[id].x, world.instances[id].y - 18.0)).collect();
    assert_eq!(spawns.len(), 6, "the six spawn points of the alpha");
    let mut game = Game::new(world);
    for &(x, y) in &spawns {
        game.players.push(Player::new(Character::Tails, x, y, &GameplayConfig::default()));
    }
    // Some of the alpha's spawn points hang above their floor (1367, 200 is 160 px over it).
    let room_height = game.world.room_height as f64;
    hold(&mut game, 0, ticks(2.0) as usize);
    for (player, &(x, y)) in game.players.iter().zip(&spawns) {
        assert!(player.is_grounded && player.y < room_height, "from ({x}, {y}) to {}: stands on floor", player.y);
    }
}

#[test]
fn a_spring_throws_a_player_up() {
    let world = started_room(RoomId::Level1);
    let spring = world.ids_of(ObjectId::SpringUp).next().expect("level1 has springs");
    let (x, y) = (world.instances[spring].x, world.instances[spring].y - 40.0);
    let mut game = Game::new(world);
    game.players.push(Player::new(Character::Tails, x, y, &GameplayConfig::default()));
    let lowest = (0..ticks(1.0)).map(|_| { hold(&mut game, 0, 1); game.players[0].y }).fold(f64::MIN, f64::max);
    hold(&mut game, 0, ticks(0.25) as usize);
    assert!(game.players[0].y < lowest - 32.0, "thrown up: {} after landing at {lowest}", game.players[0].y);
}
