//! Ravine Mist's shards and slugs.

use super::common::{hold_each, started_room};
use crate::core::config::{ticks, GameplayConfig};
use crate::core::events::SimEvent;
use crate::core::game::Game;
use crate::core::level::Level;
use crate::core::player::{Character, ExeCharacter, Player};
use crate::core::rooms::ids::RoomId;

fn ravine_mist() -> Game {
    let mut game = Game::new(started_room(RoomId::Ravinemist));
    game.level = Some(Level::new(&game.world, 11));
    game
}

#[test]
fn shards_are_found_by_touch_and_dropped_on_death() {
    let mut game = ravine_mist();
    let rules = game.world.config.levels.ravine_mist.clone();
    hold_each(&mut game, &[], 1);
    let view = game.world.ravine_mist.clone().unwrap();
    assert_eq!((view.shards.len(), view.found), (rules.shard_amount, 0));
    let shard = view.shards[0];
    game.players.push(Player::new(Character::Cream, shard.x as f64, shard.y as f64, &GameplayConfig::default()));
    hold_each(&mut game, &[0], 2);
    assert_eq!(game.players[0].shards, 1, "taken");
    assert_eq!(game.world.ravine_mist.as_ref().unwrap().found, 1);
    assert!(!game.level.as_ref().unwrap().big_ring_may_open(), "not enough for the big ring");

    let cfg = game.world.config.clone();
    game.players[0].instakill(&game.world, &cfg, &mut Vec::new());
    hold_each(&mut game, &[0], 2);
    let view = game.world.ravine_mist.clone().unwrap();
    assert_eq!(view.found, 0, "dropped again");
    let dropped = view.shards.iter().find(|shard| shard.falling).copied().expect("a falling shard");
    hold_each(&mut game, &[0], 300);
    let landed = game.world.ravine_mist.as_ref().unwrap().shards.iter().find(|shard| shard.id == dropped.id).copied().unwrap();
    assert!(!landed.falling && landed.y > dropped.y, "fell to the floor: {dropped:?} to {landed:?}");
}

#[test]
fn slugs_crawl_out_bite_and_are_squashed_by_a_jump() {
    let mut game = ravine_mist();
    let rules = game.world.config.levels.ravine_mist.clone();
    hold_each(&mut game, &[], ticks(rules.slug_every_seconds + 3.0) as usize);
    let slug = game.world.ravine_mist.as_ref().unwrap().slugs[0];
    assert!(slug.awake(), "out and faded in: {slug:?}");

    let mut tails = Player::new(Character::Tails, slug.x as f64, slug.y as f64 - 12.0, &GameplayConfig::default());
    tails.rings = 3;
    game.players.push(tails);
    hold_each(&mut game, &[0], 30);
    assert!(game.players[0].rings < 3, "bitten: {}", game.players[0].rings);

    let slug = game.world.ravine_mist.as_ref().unwrap().slugs[0];
    game.players[0].x = slug.x as f64;
    game.players[0].y = slug.y as f64 - 12.0;
    game.players[0].is_jumping = true;
    let events = hold_each(&mut game, &[0], 1);
    assert!(events.iter().any(|event| matches!(event, SimEvent::SlugSquashed { .. })), "{events:?}");
    assert!(game.players[0].yspd < 0.0, "thrown up");
}

#[test]
fn killers_hold_shards_until_stunned_and_only_survivors_open_the_ring() {
    let mut game = ravine_mist();
    let cfg = (*game.world.config).clone();
    assert!(cfg.levels.ravine_mist.killers_carry_shards, "on by default");
    hold_each(&mut game, &[], 1);
    let shards = game.world.ravine_mist.clone().unwrap().shards;
    let found = |game: &Game| game.world.ravine_mist.as_ref().unwrap().found;
    game.players.push(Player::new_exe(ExeCharacter::Original, shards[0].x as f64, shards[0].y as f64, &cfg));
    game.players.push(Player::new(Character::Cream, shards[1].x as f64, shards[1].y as f64, &cfg));
    hold_each(&mut game, &[0, 0], 2);
    assert_eq!((game.players[0].shards, game.players[1].shards), (1, 1), "both took one");
    assert_eq!(found(&game), 1, "only the survivor's shard counts");

    game.players[0].shocked_timer = ticks(1.0);
    hold_each(&mut game, &[0, 0], 1);
    assert_eq!(game.players[0].shards, 0, "stunned, the killer drops it");
    assert!(game.world.ravine_mist.as_ref().unwrap().shards.iter().any(|shard| shard.falling), "it falls where they stood");

    game.players[1].removed = true;
    hold_each(&mut game, &[0, 0], 1);
    assert_eq!(found(&game), 1, "an escaped survivor's shards still count");
    game.players[1].left_server = true;
    hold_each(&mut game, &[0, 0], 1);
    assert_eq!(found(&game), 0, "one who left the server drops theirs");
}

#[test]
fn with_the_original_rule_killers_leave_shards_alone() {
    let mut game = ravine_mist();
    let mut cfg = (*game.world.config).clone();
    cfg.levels.ravine_mist.killers_carry_shards = false;
    game.world.config = std::sync::Arc::new(cfg.clone());
    hold_each(&mut game, &[], 1);
    let shard = game.world.ravine_mist.clone().unwrap().shards[0];
    game.players.push(Player::new_exe(ExeCharacter::Original, shard.x as f64, shard.y as f64, &cfg));
    hold_each(&mut game, &[0], 2);
    assert_eq!(game.players[0].shards, 0, "EXE passes it by");
}
