//! obj_camera: centered on this player (trailing a little in the air, moved by
//! looking up or down), still for a moment after dying or escaping, then watching
//! the other survivors; shaken by hits, kept inside the room.

use crate::client::canvas::{VIEW_HEIGHT, VIEW_WIDTH};
use macroquad::rand;
use crate::core::collision::{sprite_bbox, Bbox};
use crate::core::resources::names::sprite;
use crate::core::resources::sprites::Sprites;
use crate::core::player::{gm_sign, Buttons, Character, Player};
use crate::core::config::{step, HurtRules};
use crate::core::config::ticks;

/// In the air the camera lets the player move this far vertically before following.
const AIR_SLACK: f64 = 32.0;
/// Looking up or down for longer than this moves the camera, a few pixels a step.
const LOOK_DELAY_SECONDS: f64 = 1.5;
const LOOK_DISTANCE: f64 = 100.0;
const LOOK_STEP: f64 = 2.0;
/// Pixels a 60 Hz step the camera returns towards the player when nobody looks up or down.
const RETURN_SPEED: f64 = 2.0;
/// Mode 3 waits this long before watching someone else (alarm[1] = 60 * 3).
const FADE_SECONDS: f64 = 3.0;
/// A watched player further than this from the camera is caught up with half the way a step.
const CATCH_UP_DISTANCE: f64 = 10.0;
/// Pixels a step the free camera flies with the movement keys.
const FREE_SPEED: f64 = 6.0;
const CATCH_UP_SHARE: f64 = 0.5;

/// global.cameraMode
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CameraMode {
    /// 0: this player.
    Follow,
    /// 2: someone else, changed with any key unless locked.
    Spectate,
    /// 3: standing still after dying or escaping.
    Fade,
    /// 4: on a place (Limp City's watched eye), set from outside.
    Watch,
    /// The free camera of the Sandbox page and of anyone watching the round
    /// (see `watch_everyone`): it follows nobody and flies with the movement keys.
    Free,
}

pub struct Camera {
    pub x: f64,
    pub y: f64,
    dist: f64,
    look_timer: i32,
    shake: Option<Shake>,
    pub mode: CameraMode,
    previous_mode: Option<CameraMode>,
    fade_alarm: i32,
    /// spectatingInd and spectatingObj
    spectating_index: usize,
    spectating: Option<u16>,
    /// The big ring is the stop being watched instead of a player.
    watching_ring: bool,
    /// After the round's end the camera stays on EXE.
    pub locked: bool,
    /// The one way of watching a round: a spectator, a player who chose the free
    /// camera card, and a player who died while hurt.dead_watch_everyone is on all
    /// watch everyone -- survivors, demons and killers alike, and the big ring once it
    /// is out -- skip whoever is away in their pause menu, and may fly the free camera
    /// instead. Otherwise a player who died watches only the survivors who are still
    /// up, as the original let them, and the server sends them no more than those see.
    watch_everyone: bool,
    /// hurt.watch_while_revivable: whether a player who is down while teammates may
    /// still revive them goes watching, or stays on their body until that is settled.
    watch_while_revivable: bool,
}

/// What the camera looks at this step.
pub struct CameraView<'a> {
    pub own: Option<&'a Player>,
    /// Everyone else, by id in the order the client knows them.
    pub others: &'a [(u16, Player)],
    pub exe_ids: &'a [u16],
    pub any_key_pressed: bool,
    /// Where the big ring stands once it is out (its silhouette or open).
    pub exit_ring: Option<(f64, f64)>,
    /// What this client holds down, which the free camera flies with.
    pub buttons: Buttons,
    pub room_width: f64,
    pub room_height: f64,
}

struct Shake {
    time: i32,
    magnitude: f64,
    fade: f64,
}

impl Camera {
    /// Create event: the view centered on the player. `hurt` says how they watch the
    /// round once they fall (dead_watch_everyone, watch_while_revivable).
    pub fn new(player: &Player, hurt: &HurtRules) -> Camera {
        let mut camera = Camera::at(player.x, player.y, CameraMode::Follow);
        camera.watch_everyone = hurt.dead_watch_everyone;
        camera.watch_while_revivable = hurt.watch_while_revivable;
        camera
    }

    /// The free camera, which starts where the player it stands in for would have.
    pub fn free(x: f64, y: f64) -> Camera {
        Camera::at(x, y, CameraMode::Free)
    }

    /// A spectator's camera: it has no player of its own, so it starts out looking for
    /// one of the round's to follow (states::spectate).
    pub fn watching() -> Camera {
        let mut camera = Camera::at(0.0, 0.0, CameraMode::Spectate);
        camera.watch_everyone = true;
        camera
    }

    /// The player this camera watches, if it watches one (not the ring, not flying free).
    pub fn watched(&self) -> Option<u16> {
        self.spectating.filter(|_| self.mode == CameraMode::Spectate && !self.watching_ring)
    }

    /// Watching someone and the free camera, one key apart, for whoever watches
    /// everyone. Watching needs someone to watch: with nobody else in the level (the
    /// Sandbox free camera) the camera stays free. Returns whether it switched.
    pub fn toggle_free(&mut self, others: &[(u16, Player)]) -> bool {
        if self.locked {
            return false;
        }
        let next = match self.mode {
            CameraMode::Free if !others.is_empty() => CameraMode::Spectate,
            CameraMode::Spectate if self.watch_everyone => CameraMode::Free,
            _ => return false,
        };
        self.mode = next;
        true
    }

    fn at(x: f64, y: f64, mode: CameraMode) -> Camera {
        Camera {
            x: x.floor() - VIEW_WIDTH / 2.0,
            y: y.floor() - VIEW_HEIGHT / 2.0,
            dist: AIR_SLACK,
            look_timer: 0,
            shake: None,
            mode,
            previous_mode: None,
            fade_alarm: super::super::ALARM_OFF,
            spectating_index: 0,
            spectating: None,
            watching_ring: false,
            locked: false,
            watch_everyone: false,
            watch_while_revivable: true,
        }
    }

    /// obj_aiz_zipline: the camera jumps to center the player, without look-ahead.
    pub fn snap_to(&mut self, player: &Player) {
        self.x = player.x.floor() - VIEW_WIDTH / 2.0;
        self.y = player.y.floor() - VIEW_HEIGHT / 2.0;
        self.dist = 0.0;
    }

    /// Alarm_1 and Draw_76 (Pre-Draw) of obj_camera.
    pub fn step(&mut self, view: &CameraView, sprites: &Sprites) {
        if super::super::count_down_alarm(&mut self.fade_alarm) {
            self.fade_over(view);
        }
        match self.mode {
            CameraMode::Follow => {
                if let Some(own) = view.own {
                    self.follow(own);
                }
            }
            CameraMode::Spectate => self.spectate(view, sprites),
            CameraMode::Free => self.fly(view),
            CameraMode::Watch => {}
            CameraMode::Fade => {
                if self.previous_mode != Some(CameraMode::Fade) {
                    self.fade_alarm = ticks(FADE_SECONDS);
                }
            }
        }
        self.previous_mode = Some(self.mode);
        if self.look_timer == 0 {
            let back = (RETURN_SPEED * step()).min(self.dist.abs());
            self.dist -= back * gm_sign(self.dist);
        }
        self.apply_shake();
        self.x = gm_clamp(self.x, 0.0, view.room_width - VIEW_WIDTH);
        self.y = gm_clamp(self.y, 0.0, view.room_height - VIEW_HEIGHT);
    }

    /// Alarm_1: nobody revived this player in the moment after dying, so the camera goes watching.
    fn fade_over(&mut self, view: &CameraView) {
        if view.own.is_some_and(|own| own.hp > 0) {
            return;
        }
        // Teammates may still revive them: the camera stays on the body, asking again
        // every step until they are revived or the countdown runs out.
        if !self.watch_while_revivable && view.own.is_some_and(Player::may_be_revived) {
            self.fade_alarm = 1;
            return;
        }
        let watching_exe = self.spectating.and_then(|id| view.others.iter().find(|(other, _)| *other == id)).is_some_and(|(_, player)| player.character == Character::Exe);
        if watching_exe {
            return;
        }
        (self.spectating, self.watching_ring) = (None, false);
        self.spectating_index = 0;
        self.mode = CameraMode::Spectate;
    }

    /// The free camera: the keys that walk a player move the view instead.
    fn fly(&mut self, view: &CameraView) {
        let axis = |less: u16, more: u16| f64::from(view.buttons.held(more)) - f64::from(view.buttons.held(less));
        self.x += axis(Buttons::LEFT, Buttons::RIGHT) * FREE_SPEED * step();
        self.y += axis(Buttons::UP, Buttons::DOWN) * FREE_SPEED * step();
    }

    /// Mode 2: the next player worth watching -- for someone who watches everyone,
    /// anyone at all who is not away in their pause menu; otherwise the next survivor
    /// who is neither EXE, a demon nor down. Once the big ring is out, it is one more
    /// stop after the last player, before the first one again, for whoever watches
    /// everyone: a dead survivor who could tell the living where it is waits for them
    /// to find it, as the original made them.
    fn spectate(&mut self, view: &CameraView, sprites: &Sprites) {
        let exit_ring = view.exit_ring.filter(|_| self.watch_everyone);
        let stops = view.others.len() + usize::from(exit_ring.is_some());
        if self.spectating.is_none() && !self.watching_ring && stops > 0 {
            match view.others.get(self.spectating_index.min(stops - 1)) {
                Some((id, player)) if self.worth_watching(*id, player, view.exe_ids) => self.spectating = Some(*id),
                Some(_) => self.next_spectated(stops),
                None => self.watching_ring = true,
            }
        }
        let target = match self.watching_ring {
            true => exit_ring.map(|(x, y)| {
                let body = sprite_bbox(sprites.get(sprite::SPR_BIGRING), x, y, 1.0, 1.0, 0.0);
                ((body.left + body.right) / 2.0, (body.top + body.bottom) / 2.0, body)
            }),
            false => self.spectating.and_then(|id| view.others.iter().find(|(other, _)| *other == id)).map(|(_, player)| {
                (player.x, player.y, sprite_bbox(sprites.get(player.sprite_index), player.x, player.y, player.image_xscale, 1.0, 0.0))
            }),
        };
        match target {
            Some((x, y, body)) => self.catch_up(x, y, &body),
            None => (self.spectating, self.watching_ring) = (None, false),
        }
        if !self.locked && view.any_key_pressed {
            // After the ring comes the first player again.
            if self.watching_ring {
                self.spectating_index = 0;
            } else {
                self.next_spectated(stops);
            }
            (self.spectating, self.watching_ring) = (None, false);
        }
    }

    fn worth_watching(&self, id: u16, player: &Player, exe_ids: &[u16]) -> bool {
        match self.watch_everyone {
            true => !player.paused,
            false => !exe_ids.contains(&id) && player.revival_times < 2 && !player.is_dead,
        }
    }

    /// Centers the view on (`x`, `y`), half the way a step while `body` is far from it.
    fn catch_up(&mut self, x: f64, y: f64, body: &Bbox) {
        let (to_x, to_y) = (x.floor() - VIEW_WIDTH / 2.0, y.floor() - VIEW_HEIGHT / 2.0);
        // distance_to_object from the camera's own position to the watched box.
        let dx = (body.left - self.x).max(self.x - body.right).max(0.0);
        let dy = (body.top - self.y).max(self.y - body.bottom).max(0.0);
        if dx.hypot(dy) > CATCH_UP_DISTANCE {
            self.x += (to_x - self.x) * crate::core::config::eased_share(CATCH_UP_SHARE);
            self.y += (to_y - self.y) * crate::core::config::eased_share(CATCH_UP_SHARE);
        } else {
            (self.x, self.y) = (to_x, to_y);
        }
    }

    fn next_spectated(&mut self, stops: usize) {
        self.spectating_index += 1;
        if self.spectating_index + 1 > stops {
            self.spectating_index = 0;
        }
    }

    /// SERVER_GAME_PLAYER_ESCAPED: the player this camera watched got out, so it
    /// watches the ring they got out through, if it may watch the ring at all.
    pub fn follow_into_ring(&mut self, escaped: u16) {
        if self.mode == CameraMode::Spectate && self.spectating == Some(escaped) && self.watch_everyone {
            (self.spectating, self.watching_ring) = (None, true);
        }
    }

    /// The round's end: the camera stays on `player` (EXE) until the level closes.
    pub fn lock_on(&mut self, player: u16) {
        self.mode = CameraMode::Spectate;
        (self.spectating, self.watching_ring) = (Some(player), false);
        self.locked = true;
    }

    /// scr_camera_shake
    pub fn shake(&mut self, time: i32, magnitude: f64, fade: f64) {
        // The time comes in the original's 60 Hz steps.
        self.shake = Some(Shake { time: crate::core::config::original_ticks(time), magnitude, fade });
    }

    /// Draw_76 with global.cameraMode 0.
    fn follow(&mut self, player: &Player) {
        let centered_x = player.x.floor() - VIEW_WIDTH / 2.0;
        let centered_y = player.y.floor() - VIEW_HEIGHT / 2.0;
        self.x = centered_x;
        if !player.is_grounded {
            self.dist = self.y - centered_y;
            if self.dist.abs() > AIR_SLACK {
                self.dist = AIR_SLACK * gm_sign(self.dist);
                self.y = centered_y + self.dist;
            }
        } else {
            self.look(player);
            self.y = centered_y + self.dist;
        }
    }

    fn look(&mut self, player: &Player) {
        // scr_collision_objects_after: a hiding player holds the timer at its start.
        if player.is_hiding {
            self.look_timer = 0;
        }
        let direction = if player.is_looking_down {
            1.0
        } else if player.is_looking_up {
            -1.0
        } else {
            self.look_timer = 0;
            return;
        };
        self.look_timer += 1;
        if self.look_timer > ticks(LOOK_DELAY_SECONDS) && self.dist * direction < LOOK_DISTANCE {
            self.dist += LOOK_STEP * direction * step();
        }
    }

    fn apply_shake(&mut self) {
        let Some(shake) = self.shake.as_mut() else { return };
        shake.time -= 1;
        self.x += choose_sign() * shake.magnitude;
        self.y += choose_sign() * shake.magnitude;
        if shake.time <= 0 {
            shake.magnitude -= shake.fade * step();
            if shake.magnitude <= 0.0 {
                self.shake = None;
            }
        }
    }
}

/// choose(-1, 1)
fn choose_sign() -> f64 {
    if rand::gen_range(0, 2) == 0 {
        -1.0
    } else {
        1.0
    }
}

/// GameMaker clamp: the upper bound wins when the bounds cross (a room smaller than the view).
fn gm_clamp(value: f64, low: f64, high: f64) -> f64 {
    let raised = if value < low { low } else { value };
    if raised > high {
        high
    } else {
        raised
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// With nobody else in the level there is nobody to watch: the Sandbox free
    /// camera stays free, and a dead player who watches only survivors has no free camera.
    #[test]
    fn watching_and_the_free_camera_switch_only_when_both_make_sense() {
        let tails = || Player::new(Character::Tails, 0.0, 0.0, &crate::core::config::GameplayConfig::default());
        let mut alone = Camera::free(0.0, 0.0);
        assert!(!alone.toggle_free(&[]));
        assert_eq!(alone.mode, CameraMode::Free);

        let others = [(2, tails())];
        assert!(alone.toggle_free(&others));
        assert_eq!(alone.mode, CameraMode::Spectate);

        let only_survivors = HurtRules { dead_watch_everyone: false, ..HurtRules::default() };
        let everyone = HurtRules { dead_watch_everyone: true, ..HurtRules::default() };
        let mut dead = Camera::new(&tails(), &only_survivors);
        dead.mode = CameraMode::Spectate;
        assert!(!dead.toggle_free(&others));
        let mut dead = Camera::new(&tails(), &everyone);
        dead.mode = CameraMode::Spectate;
        assert!(dead.toggle_free(&others));
        assert_eq!(dead.mode, CameraMode::Free);
    }

    /// Once the big ring is out it is the stop after the last player, and a watched
    /// player who escapes leaves the camera on the ring.
    #[test]
    fn the_big_ring_is_the_last_stop_and_where_an_escape_leads() {
        let textures = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Resources/Textures");
        let sprites = Sprites::index(&textures).unwrap();
        let tails = Player::new(Character::Tails, 0.0, 0.0, &crate::core::config::GameplayConfig::default());
        let others = [(2, tails.clone())];
        let view = |any_key_pressed: bool| CameraView {
            own: None,
            others: &others,
            exe_ids: &[],
            any_key_pressed,
            buttons: Buttons::default(),
            room_width: 4096.0,
            room_height: 4096.0,
            exit_ring: Some((1024.0, 1024.0)),
        };
        let mut camera = Camera::watching();
        camera.step(&view(false), &sprites);
        assert_eq!((camera.spectating, camera.watching_ring), (Some(2), false));
        camera.step(&view(true), &sprites);
        camera.step(&view(false), &sprites);
        assert!(camera.watching_ring, "after the last player comes the ring");
        camera.step(&view(true), &sprites);
        camera.step(&view(false), &sprites);
        assert_eq!((camera.spectating, camera.watching_ring), (Some(2), false), "and after the ring the first player");

        camera.follow_into_ring(2);
        assert!(camera.watching_ring);

        // A dead survivor who watches only the living is shown no ring.
        let mut dead = Camera::new(&tails, &HurtRules::default());
        dead.mode = CameraMode::Spectate;
        for any_key_pressed in [false, true, false, true, false] {
            dead.step(&view(any_key_pressed), &sprites);
            assert!(!dead.watching_ring);
        }
        dead.follow_into_ring(2);
        assert!(!dead.watching_ring);
    }

    /// With hurt.watch_while_revivable off, a player who is down stays on their body
    /// while teammates may revive them, and goes watching once the countdown runs out.
    #[test]
    fn a_revivable_player_watches_nobody_when_the_server_says_so() {
        let textures = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Resources/Textures");
        let sprites = Sprites::index(&textures).unwrap();
        let mut down = Player::new(Character::Tails, 0.0, 0.0, &crate::core::config::GameplayConfig::default());
        (down.hp, down.is_dead) = (0, true);
        let others = [(2, down.clone())];
        let blind = HurtRules { watch_while_revivable: false, ..HurtRules::default() };
        let mut camera = Camera::new(&down, &blind);
        camera.mode = CameraMode::Fade;
        let step = |camera: &mut Camera, own: &Player| {
            let view = CameraView { own: Some(own), others: &others, exe_ids: &[], any_key_pressed: false, buttons: Buttons::default(), room_width: 4096.0, room_height: 4096.0, exit_ring: None };
            camera.step(&view, &sprites);
        };
        for _ in 0..ticks(FADE_SECONDS) * 2 {
            step(&mut camera, &down);
        }
        assert_eq!(camera.mode, CameraMode::Fade, "still on the body while revivable");
        down.revival_times = 1;
        step(&mut camera, &down);
        assert_eq!(camera.mode, CameraMode::Spectate, "watching once nobody can revive them");
    }
}
