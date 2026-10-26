//! Behavior checks of player physics on real converted maps. Not parity tests:
//! parity needs traces recorded from the original game.

use super::common::started_room;
use crate::core::resources::names::sound;
use crate::core::config::GameplayConfig;
use crate::core::events::SimEvent;
use crate::core::objects::ids::ObjectId;
use crate::core::player::{Buttons, Character, Player};
use crate::core::game::Game;
use crate::core::rooms::ids::RoomId;
use crate::core::world::World;

fn run(player: &mut Player, world: &World, buttons: u16, ticks: usize) -> Vec<SimEvent> {
    let mut events = Vec::new();
    for _ in 0..ticks {
        player.tick(world, Buttons(buttons), &mut events);
    }
    events
}

/// Drops `player` onto the spring; true once it launched.
fn falls_onto_spring(player: &mut Player, world: &World) -> bool {
    (0..60).any(|_| {
        let events = run(player, world, 0, 1);
        events.contains(&SimEvent::Sound { sound: sound::SND_SPRING, x: player.x, y: player.y })
    })
}

#[test]
fn player_lands_on_greenhill_and_runs_right() {
    let world = started_room(RoomId::Greenhill);
    let spawn = world.ids_of(ObjectId::Spawnpoint).next().expect("greenhill has a spawn point");
    let mut player = Player::new(Character::Tails, world.instances[spawn].x, world.instances[spawn].y - 18.0, &GameplayConfig::default());

    run(&mut player, &world, 0, 120);
    assert!(player.is_grounded, "player should stand on the floor after 2 seconds: {player:?}");

    let start_x = player.x;
    run(&mut player, &world, Buttons::RIGHT, 120);
    assert!(player.is_grounded, "player fell through the floor while running: {player:?}");
    assert!(player.x > start_x + 100.0, "player should run right: {} -> {}", start_x, player.x);
}

#[test]
fn spring_launches_player_up_and_recharges() {
    let world = started_room(RoomId::Angelisland);
    // Exactly obj_spring_up: ids_of also returns children such as wall springs.
    let spring = world
        .ids_of(ObjectId::SpringUp)
        .find(|&id| world.instances[id].object == ObjectId::SpringUp)
        .expect("angel island has an up spring");
    let bbox = world.bbox(spring).unwrap();
    let above_spring = ((bbox.left + bbox.right) / 2.0, bbox.top - 100.0);
    let mut player = Player::new(Character::Tails, above_spring.0, above_spring.1, &GameplayConfig::default());

    assert!(falls_onto_spring(&mut player, &world), "spring should fire when falling onto it: {player:?}");
    assert_eq!(player.yspd, -12.0, "obj_spring_up throws with ydir = -12");
    assert_eq!(player.spring_look(spring), (true, true), "spring should show pressed and recharge");

    let mut teammate = Player::new(Character::Tails, above_spring.0, above_spring.1, &GameplayConfig::default());
    assert!(falls_onto_spring(&mut teammate, &world), "each player has its own copy of the spring, as in the original");

    run(&mut player, &world, 0, 240);
    assert_eq!(player.spring_look(spring), (false, false), "spring recharges after 4 seconds");
}

/// The rings are the simulation's: a game that owns its level (the server's round, and
/// Sandbox, which runs the same simulation without any networking) puts them out on
/// the map's ring spawners by itself, and a survivor walking into one takes it.
#[test]
fn a_game_with_its_own_level_puts_out_rings_and_lets_a_player_take_one() {
    use crate::core::config::ticks;
    let world = started_room(RoomId::Greenhill);
    let spawner = world.ids_of(ObjectId::RingSpawner).next().expect("green hill has ring spawners");
    let (x, y) = (world.instances[spawner].x, world.instances[spawner].y);
    let mut game = Game::new(world);
    game.level = Some(crate::core::level::Level::new(&game.world, 12345));
    let rules = game.world.config.rings.clone();

    // Nobody in the level yet: the rings still appear, one every spawn_every_seconds.
    super::common::hold(&mut game, 0, ticks(rules.spawn_every_seconds) as usize + 1);
    assert_eq!(game.world.rings.len(), 1, "one ring after the first interval");
    super::common::hold(&mut game, 0, ticks(rules.spawn_every_seconds) as usize);
    assert_eq!(game.world.rings.len(), 2, "and another one after the next");
    assert!(game.world.rings.iter().all(|ring| ring.spawner.is_some()), "each stands on a ring spawner");

    // A client in a round has no level of its own and must not put out rings.
    let mut client = Game::new(started_room(RoomId::Greenhill));
    super::common::hold(&mut client, 0, ticks(rules.spawn_every_seconds) as usize * 3);
    assert!(client.world.rings.is_empty(), "a game without a level shows only what it is sent");

    // Tails walks into a ring standing on the first spawner.
    game.world.rings.clear();
    game.world.rings.push(crate::core::rings::MapRing::on_spawner(x, y, false, 0));
    game.players.push(Player::new(Character::Tails, x, y, &GameplayConfig::default()));
    let events = super::common::hold(&mut game, 0, ticks(1.0 / rules.fade_in_per_step / 60.0) as usize + 2);
    assert!(game.world.rings.is_empty(), "the ring is taken once it has faded in");
    assert_eq!(game.players[0].rings, 1, "and it counts for the player");
    assert!(events.iter().any(|event| matches!(event, SimEvent::RingTaken { red: false, .. })), "{events:?}");
}

/// scr_move_basic: idling in one screenful of the map for twenty seconds is warned
/// about and slowed down, and moving a screen away ends both.
#[test]
fn staying_in_one_screenful_warns_the_survivor_and_slows_them() {
    use crate::core::config::ticks;
    let mut game = super::common::greenhill_with(&[Character::Tails]);
    let rules = game.world.config.anti_idle.clone();
    let patient = ticks(rules.warning_after_seconds + rules.chunk_check_seconds + 1.0) as usize;
    let events = super::common::hold(&mut game, 0, patient);
    assert!(events.contains(&SimEvent::IdleWarning { shown: true }), "warned after {} s of standing still", rules.warning_after_seconds);
    let tails = &game.players[0];
    assert!(tails.idle_warning && tails.is_slow, "and slowed while it is up: {tails:?}");
    assert!(tails.max_h_speed < GameplayConfig::default().tails.movement.max_speed_per_step, "top speed cut: {}", tails.max_h_speed);

    // A screen to the right is another chunk, which the next check notices.
    game.players[0].x += rules.chunk_width + 50.0;
    let events = super::common::hold(&mut game, 0, ticks(rules.chunk_check_seconds) as usize + 1);
    assert!(events.contains(&SimEvent::IdleWarning { shown: false }), "moving away ends it");
    assert!(!game.players[0].idle_warning);
}

#[test]
fn the_room_sides_are_a_wall_on_every_map() {
    let world = started_room(RoomId::Greenhill);
    let physics = GameplayConfig::default().physics.side_sensor_spread;
    // As a map whose own wall objects a modder moved or removed would let happen.
    let mut player = Player::new(Character::Tails, -50.0, 200.0, &GameplayConfig::default());
    player.xspd = -9.0;
    run(&mut player, &world, Buttons::LEFT, 1);
    assert_eq!((player.x, player.xspd), (physics, 0.0), "held at the left edge: {player:?}");

    let mut player = Player::new(Character::Tails, world.room_width + 50.0, 200.0, &GameplayConfig::default());
    player.xspd = 9.0;
    run(&mut player, &world, Buttons::RIGHT, 1);
    assert_eq!((player.x, player.xspd), (world.room_width - physics, 0.0), "held at the right edge: {player:?}");
}

#[test]
fn a_map_with_no_return_points_sends_a_fallen_player_to_its_spawn() {
    let world = started_room(RoomId::Greenhill);
    assert!(
        world.ids_of(ObjectId::AbyssTarget).next().is_none() && world.ids_of(ObjectId::DeathtpPoint).next().is_none(),
        "green hill has neither kind of return point"
    );
    let x = 600.0;
    let spawn = world.instance_nearest(x, world.room_height + 100.0, ObjectId::Spawnpoint).expect("green hill has spawn points");
    let (spawn_x, spawn_y) = (world.instances[spawn].x, world.instances[spawn].y);
    let mut player = Player::new(Character::Tails, x, world.room_height + 100.0, &GameplayConfig::default());
    run(&mut player, &world, 0, 1);
    let cfg = GameplayConfig::default();
    assert_eq!(player.hp, cfg.hurt.max_hp - cfg.hazards.abyss_damage, "the fall costs the abyss damage");
    // The spawn points stand on the floor, which lifts a standing player a body's height in the same tick.
    assert!((player.x - spawn_x).abs() < 1.0 && (player.y - spawn_y).abs() <= 24.0, "back at the nearest spawn {:?}: {player:?}", (spawn_x, spawn_y));
}

#[test]
fn falling_out_of_the_room_hurts_and_returns() {
    let world = started_room(RoomId::Angelisland);
    let mut player = Player::new(Character::Tails, 100.0, world.room_height + 100.0, &GameplayConfig::default());
    let events = run(&mut player, &world, 0, 1);
    let cfg = GameplayConfig::default();
    assert_eq!(player.hp, cfg.hurt.max_hp - cfg.hazards.abyss_damage, "the abyss costs its damage");
    assert!(player.y < world.room_height, "player should be moved back into the room");
    assert!(events.contains(&SimEvent::CameraShake));
}

#[test]
fn water_slows_splashes_and_shocks_after_lightning() {
    let mut world = started_room(RoomId::Greenhill);
    let water = world.ids_of(ObjectId::GhzWater).next().expect("green hill has water");
    let bbox = world.bbox(water).unwrap();
    // A column where the water is open: no floor from its surface to well below it.
    let x = (0..world.room_width as i64)
        .step_by(16)
        .map(|x| x as f64)
        .find(|&x| (bbox.top as i64..bbox.top as i64 + 48).all(|y| !world.position_meeting(x, y as f64, ObjectId::FloorParent)))
        .expect("green hill has open water");
    let mut player = Player::new(Character::Tails, x, bbox.top + 20.0, &GameplayConfig::default());
    let events = run(&mut player, &world, 0, 2);
    assert!(player.in_water && player.is_slow, "feet in the water slow the player: {player:?}");
    assert!(events.iter().any(|event| matches!(event, SimEvent::Sound { sound: sound::SND_WATERSPLASH, .. })), "{events:?}");
    let cfg = GameplayConfig::default();
    assert_eq!(player.hp, cfg.hurt.max_hp, "no lightning yet");

    crate::core::level::LevelView { water_shocks: Some(true), ..Default::default() }.apply(&mut world);
    run(&mut player, &world, 0, 1);
    assert_eq!(player.hp, cfg.hurt.max_hp - cfg.levels.green_hill.shock_damage, "the shocked water hurts");
}

/// Rings.toml scatter_on_hit: a hit throws the survivor's rings about; they bounce off
/// the floor, cannot be caught back at once, then anyone can take them, and the rest
/// are gone when their time is up.
#[test]
fn a_hit_scatters_the_rings_which_bounce_and_can_be_taken_back() {
    use crate::core::config::ticks;
    use crate::core::player::hurt::Hit;
    let world = started_room(RoomId::Greenhill);
    let (x, standing_y) = super::common::flat_floor_spot(&world);
    // flat_floor_spot gives a place 40 pixels above where a body stands on the floor.
    let floor = standing_y + 18.0 + 40.0;
    let mut game = Game::new(world);
    game.level = Some(crate::core::level::Level::new(&game.world, 1));
    game.world.config = std::sync::Arc::new(GameplayConfig { rings: crate::core::config::RingRules { enabled: false, scatter_on_hit: true, ..GameplayConfig::default().rings }, ..GameplayConfig::default() });
    let cfg = game.world.config.clone();
    let mut tails = Player::new(Character::Tails, x, standing_y, &cfg);
    tails.rings = 5;
    let mut events = Vec::new();
    tails.hurt(&game.world, &cfg, Hit::damage(&cfg, 1), &mut events);
    assert_eq!(tails.rings, 0);
    let Some(&SimEvent::RingsScattered { count, .. }) = events.iter().find(|event| matches!(event, SimEvent::RingsScattered { .. })) else { panic!("{events:?}") };
    assert_eq!(count, 5);

    game.world.rings = crate::core::rings::MapRing::scattered(&cfg, x, standing_y, count);
    game.players.push(tails);
    super::common::hold(&mut game, 0, ticks(cfg.rings.scatter_untouchable_seconds) as usize - 1);
    assert_eq!(game.world.rings.len(), 5, "nobody can take them yet");
    assert!(game.world.rings.iter().all(|ring| ring.y < floor), "they bounce off the floor instead of falling through");
    let room_width = game.world.room_width;
    assert!(game.world.rings.iter().all(|ring| ring.x >= -16.0 && ring.x <= room_width), "and stay in the room");
    super::common::hold(&mut game, 0, ticks(cfg.rings.scatter_seconds) as usize);
    assert!(game.world.rings.is_empty(), "taken or gone once their time is up");
}

/// While the round waits for its players the map's objects wait too: no ring comes out
/// before the round starts, and they do once it has.
#[test]
fn the_level_waits_for_the_round_to_start() {
    use crate::core::config::ticks;
    let mut game = Game::new(started_room(RoomId::Greenhill));
    game.level = Some(crate::core::level::Level::new(&game.world, 1));
    game.world.round_started = false;
    let spawn_every = ticks(game.world.config.rings.spawn_every_seconds) as usize;
    super::common::hold(&mut game, 0, spawn_every * 2);
    assert!(game.world.rings.is_empty(), "nothing comes out while waiting");
    game.world.round_started = true;
    super::common::hold(&mut game, 0, spawn_every + 1);
    assert!(!game.world.rings.is_empty(), "and the rings start once the round has");
}
