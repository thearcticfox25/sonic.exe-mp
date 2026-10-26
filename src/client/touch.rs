//! obj_controls' touch controls (scr_controls_init_touch, scr_controls_update_touch,
//! its Draw GUI and Room Start): a joystick and buttons drawn over the game that hold
//! the bound keys while a finger is on them, as keyboard_key_press did. They are on
//! when `touch_controls` in Options.toml says so, which is the default on Android only.

use crate::client::canvas::Canvas;
use crate::client::options::KeyBindings;
use crate::core::resources::names::sprite;
use crate::core::resources::SpriteId;
use crate::core::world::C_WHITE;
use macroquad::input::{self as mq, KeyCode, MouseButton, TouchPhase};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Where a control stands on the 480x270 view, how big and how see-through it is.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub struct Placement {
    pub x: f64,
    pub y: f64,
    pub scale: f64,
    pub opacity: f64,
}

/// The controls, numbered as obj_controls.buttonSetProp numbers them; Pause is new.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Control {
    Joystick,
    A,
    B,
    C,
    Emotion1,
    Emotion2,
    Emotion3,
    PlayerList,
    Idle,
    Pause,
}

pub const CONTROLS: [Control; 10] = [
    Control::Joystick,
    Control::A,
    Control::B,
    Control::C,
    Control::Emotion1,
    Control::Emotion2,
    Control::Emotion3,
    Control::PlayerList,
    Control::Idle,
    Control::Pause,
];

/// The layout the player arranges on the touch controls page (obj_config's *Touch).
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(default)]
pub struct TouchLayout {
    pub joystick: Placement,
    pub a: Placement,
    pub b: Placement,
    pub c: Placement,
    pub emotion1: Placement,
    pub emotion2: Placement,
    pub emotion3: Placement,
    pub player_list: Placement,
    pub idle: Placement,
    pub pause: Placement,
}

/// obj_config's first layout, which "reset" on the touch controls page brings back.
impl Default for TouchLayout {
    fn default() -> TouchLayout {
        let at = |x, y| Placement { x, y, scale: 1.0, opacity: 0.6 };
        TouchLayout {
            joystick: at(68.0, 187.0),
            a: at(378.0, 221.0),
            b: at(416.0, 177.0),
            c: at(452.0, 221.0),
            emotion1: at(466.0, 48.0),
            emotion2: at(466.0, 80.0),
            emotion3: at(466.0, 112.0),
            player_list: at(23.0, 81.0),
            idle: at(466.0, 144.0),
            // Above the emotion buttons, one step of theirs higher.
            pause: at(466.0, 16.0),
        }
    }
}

impl TouchLayout {
    pub fn get(&self, control: Control) -> Placement {
        *self.clone().get_mut(control)
    }

    pub fn get_mut(&mut self, control: Control) -> &mut Placement {
        match control {
            Control::Joystick => &mut self.joystick,
            Control::A => &mut self.a,
            Control::B => &mut self.b,
            Control::C => &mut self.c,
            Control::Emotion1 => &mut self.emotion1,
            Control::Emotion2 => &mut self.emotion2,
            Control::Emotion3 => &mut self.emotion3,
            Control::PlayerList => &mut self.player_list,
            Control::Idle => &mut self.idle,
            Control::Pause => &mut self.pause,
        }
    }
}

/// point_in_circle radii of scr_controls_update_touch. The joystick is taken within
/// a circle that does not grow with it, and leans once the finger is this far off.
const BIG_BUTTON_RADIUS: f64 = 40.0;
const SMALL_BUTTON_RADIUS: f64 = 22.0;
const JOYSTICK_GRAB_RADIUS: f64 = 160.0;
const JOYSTICK_DEAD_ZONE: f64 = 16.0;
/// A control a finger is on is drawn this much less see-through.
const HELD_OPACITY: f64 = 0.15;
/// A tap anywhere hides the on-screen keyboard once it has been up this long
/// (kbdTimer >= 60 * .1 in steps of the original).
const KEYBOARD_HIDE_SECONDS: f64 = 0.1;

impl Control {
    /// The radius a finger has to be within, at the control's scale.
    pub fn radius(self, placement: Placement) -> f64 {
        match self {
            Control::Joystick => JOYSTICK_GRAB_RADIUS,
            Control::A | Control::B | Control::C => BIG_BUTTON_RADIUS * placement.scale,
            _ => SMALL_BUTTON_RADIUS * placement.scale,
        }
    }

    /// The button's picture: frame 0 free, frame 1 pressed (the joystick has a frame
    /// for each way it leans instead).
    fn button_sprite(self) -> SpriteId {
        match self {
            Control::Joystick => sprite::SPR_MOBILE_DPAD,
            Control::A | Control::B | Control::C => sprite::SPR_MOBILE_BUTTON,
            _ => sprite::SPR_MOBILE_SMALLBUTTON,
        }
    }

    /// What a small button shows on it, free or pressed. The big ones say it in words
    /// (`label`), since what they do depends on the room.
    pub fn icon(self, held: bool) -> Option<(SpriteId, f64)> {
        // mobile_smallicons holds each icon free, then pressed.
        let small_icon = |first_frame: f64| (sprite::SPR_MOBILE_SMALLICONS, first_frame + if held { 1.0 } else { 0.0 });
        match self {
            Control::Emotion1 => Some((sprite::SPR_GUI_EMOTION_ANGER, 0.0)),
            Control::Emotion2 => Some((sprite::SPR_GUI_EMOTION_MISERY, 0.0)),
            Control::Emotion3 => Some((sprite::SPR_GUI_EMOTION_HAPPINESS, 0.0)),
            Control::PlayerList => Some(small_icon(0.0)),
            Control::Pause => Some(small_icon(2.0)),
            Control::Idle => Some(small_icon(4.0)),
            Control::Joystick | Control::A | Control::B | Control::C => None,
        }
    }

    /// A, B and C.
    fn is_big_button(self) -> bool {
        matches!(self, Control::A | Control::B | Control::C)
    }

    /// Whether the control is shown and held in this room: the big buttons everywhere,
    /// the rest only with the joystick, outside the lobby (Draw GUI's bUse and jUse).
    fn is_used_in(self, room: TouchRoom) -> bool {
        match room {
            TouchRoom::Off => false,
            TouchRoom::Lobby { .. } => self.is_big_button(),
            TouchRoom::Sandbox => self != Control::PlayerList,
            // The pause menu is a level's (level/pause.rs).
            TouchRoom::Waiting => self != Control::Pause,
            TouchRoom::Play => true,
        }
    }
}

/// What the controls do in the current room (obj_controls Room Start, and the room
/// cases of APressed, BPressed and CPressed).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TouchRoom {
    /// Logo, menu, connecting, results: no controls.
    Off,
    /// The lobby: only A, B and C. While it votes B and C pick the map; before, C opens the chat.
    Lobby { voting: bool },
    /// The waiting room: everything, and C opens the chat.
    Waiting,
    /// A level (and the rest): everything, each control holding its key.
    Play,
    /// A level played alone: as Play, without the player list, which has nobody on it.
    Sandbox,
}

/// A finger on the view, or the left mouse button while no finger is down
/// (device_mouse 0 is the mouse where there is one).
#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub id: u64,
    pub x: f64,
    pub y: f64,
}

const MOUSE_POINT: u64 = u64::MAX;

/// The fingers down this tick, in view coordinates.
pub fn points(canvas: &Canvas) -> Vec<Point> {
    let mut points: Vec<Point> = mq::touches()
        .into_iter()
        .filter(|touch| !matches!(touch.phase, TouchPhase::Ended | TouchPhase::Cancelled))
        .map(|touch| {
            let (x, y) = canvas.window_to_view(touch.position.x, touch.position.y);
            Point { id: touch.id, x, y }
        })
        .collect();
    if points.is_empty() && mq::is_mouse_button_down(MouseButton::Left) {
        let (x, y) = mq::mouse_position();
        let (x, y) = canvas.window_to_view(x, y);
        points.push(Point { id: MOUSE_POINT, x, y });
    }
    points
}

/// A HUD sign put on a button instead of in its corner (obj_playerui's gauges, where
/// a thumb would hide them): drawn over the button, centered on it.
#[derive(Clone, Copy, Debug)]
pub struct ButtonSign {
    pub button: Control,
    pub sprite: SpriteId,
    pub frame: f64,
    pub color: u32,
    pub alpha: f64,
}

#[derive(Default)]
pub struct TouchControls {
    /// The finger holding each control, in CONTROLS order.
    holders: [Option<u64>; CONTROLS.len()],
    /// jDir: the joystick picture's frame, the way it leans.
    joystick_frame: f64,
    room: Option<TouchRoom>,
    /// global.showHud: the controls hide with the HUD.
    pub hud_shown: bool,
    keyboard_shown: bool,
    keyboard_seconds: f64,
    /// The signs the level puts on the buttons this tick.
    pub signs: Vec<ButtonSign>,
}

impl TouchControls {
    /// scr_controls_update_touch: which controls the fingers hold, and the keys they
    /// hold for this room.
    pub fn update(&mut self, points: &[Point], room: TouchRoom, layout: &TouchLayout, keys: &KeyBindings) -> HashSet<KeyCode> {
        if self.room != Some(room) {
            // Room Start: nothing stays held from the room before.
            self.holders = Default::default();
            self.room = Some(room);
        }
        self.signs.clear();
        self.hide_keyboard_on_a_tap(points);
        let mut held = HashSet::new();
        if room == TouchRoom::Off {
            self.holders = Default::default();
            return held;
        }
        for (index, &control) in CONTROLS.iter().enumerate() {
            let placement = layout.get(control);
            let inside = |point: &Point| (point.x - placement.x).hypot(point.y - placement.y) <= control.radius(placement);
            let used = control.is_used_in(room);
            let holder = &mut self.holders[index];
            // A button lets go when its finger leaves it; the joystick keeps its finger.
            let keeps = |id: u64| points.iter().any(|point| point.id == id && (control == Control::Joystick || inside(point)));
            if !used || holder.is_some_and(|id| !keeps(id)) {
                *holder = None;
            }
            if used && holder.is_none() {
                *holder = points.iter().find(|point| inside(point)).map(|point| point.id);
                if holder.is_some() && self.opens_chat(control, room) {
                    self.keyboard_shown = !self.keyboard_shown;
                    macroquad::miniquad::window::show_keyboard(self.keyboard_shown);
                }
            }
        }
        for (index, &control) in CONTROLS.iter().enumerate() {
            if self.holders[index].is_some() && control != Control::Joystick {
                held.insert(key_of(control, room, keys));
            }
        }
        self.joystick_frame = 0.0;
        // The player list covers the screen: the joystick lets go while it is held.
        if self.holder(Control::PlayerList).is_none() {
            if let Some(point) = self.holder(Control::Joystick).and_then(|id| points.iter().find(|point| point.id == id)) {
                self.lean_joystick(point, layout.joystick, keys, &mut held);
            }
        }
        held
    }

    fn holder(&self, control: Control) -> Option<u64> {
        self.holders[CONTROLS.iter().position(|&known| known == control).expect("every control is listed")]
    }

    /// Up or down, then left or right, past the dead zone; the frame shows the last.
    fn lean_joystick(&mut self, point: &Point, joystick: Placement, keys: &KeyBindings, held: &mut HashSet<KeyCode>) {
        let (dx, dy) = (point.x - joystick.x, point.y - joystick.y);
        let dead_zone = JOYSTICK_DEAD_ZONE * joystick.scale;
        if dy > dead_zone {
            held.insert(keys.down.0);
            self.joystick_frame = 4.0;
        } else if dy < -dead_zone {
            held.insert(keys.up.0);
            self.joystick_frame = 3.0;
        }
        if dx > dead_zone {
            held.insert(keys.right.0);
            self.joystick_frame = 2.0;
        } else if dx < -dead_zone {
            held.insert(keys.left.0);
            self.joystick_frame = 1.0;
        }
    }

    /// C opens and closes the chat where there is one, with the on-screen keyboard.
    fn opens_chat(&self, control: Control, room: TouchRoom) -> bool {
        control == Control::C && matches!(room, TouchRoom::Lobby { voting: false } | TouchRoom::Waiting)
    }

    fn hide_keyboard_on_a_tap(&mut self, points: &[Point]) {
        if !self.keyboard_shown {
            self.keyboard_seconds = 0.0;
            return;
        }
        self.keyboard_seconds += 1.0 / crate::core::config::ticks_per_second();
        let tapped_elsewhere = !points.is_empty() && self.holder(Control::C).is_none();
        if self.keyboard_seconds >= KEYBOARD_HIDE_SECONDS && tapped_elsewhere {
            self.keyboard_shown = false;
            macroquad::miniquad::window::show_keyboard(false);
        }
    }

    /// obj_controls Draw GUI: the controls this room uses, brighter while held.
    pub fn draw(&self, canvas: &mut Canvas, layout: &TouchLayout) {
        let Some(room) = self.room.filter(|&room| room != TouchRoom::Off) else { return };
        if !self.hud_shown {
            return;
        }
        for (index, &control) in CONTROLS.iter().enumerate() {
            if !control.is_used_in(room) {
                continue;
            }
            let held = self.holders[index].is_some();
            draw_control(canvas, control, layout.get(control), held, self.joystick_frame);
            // A HUD sign on the button says what it does better than a word.
            if !self.signs.iter().any(|sign| sign.button == control) {
                draw_label(canvas, control, room, layout.get(control));
            }
        }
        for sign in &self.signs {
            // Centerd by what the picture shows: the gauges are drawn at the left of theirs.
            let (meta, at) = (canvas.sprites.get(sign.sprite), layout.get(sign.button));
            let [left, top, right, bottom] = meta.bbox;
            let x = at.x + meta.origin_x - (left + right + 1) as f64 / 2.0;
            let y = at.y + meta.origin_y - (top + bottom + 1) as f64 / 2.0;
            canvas.draw_sprite_ext(sign.sprite, sign.frame, x, y, 1.0, 1.0, 0.0, sign.color, sign.alpha);
        }
    }
}

/// A control at its place, at its scale and opacity, with its icon on it.
pub fn draw_control(canvas: &mut Canvas, control: Control, placement: Placement, held: bool, joystick_frame: f64) {
    let opacity = if held { placement.opacity + HELD_OPACITY } else { placement.opacity };
    let frame = match control {
        Control::Joystick => joystick_frame,
        _ if held => 1.0,
        _ => 0.0,
    };
    let (x, y, scale) = (placement.x, placement.y, placement.scale);
    canvas.draw_sprite_ext(control.button_sprite(), frame, x, y, scale, scale, 0.0, C_WHITE, opacity);
    if let Some((icon, frame)) = control.icon(held) {
        canvas.draw_sprite_ext(icon, frame, x, y, scale, scale, 0.0, C_WHITE, opacity);
    }
}

/// The word on a big button, centered on it, in the chat lettering.
pub fn draw_label(canvas: &mut Canvas, control: Control, room: TouchRoom, placement: Placement) {
    let Some(label) = label_of(control, room) else { return };
    let width = crate::client::text::text_width(canvas, label);
    let top = placement.y - (LABEL_HEIGHT / 2.0).ceil();
    crate::client::text::draw_text(canvas, placement.x - (width / 2.0).floor(), top, label, C_WHITE, placement.opacity);
}

/// The height of the chat lettering's letters.
const LABEL_HEIGHT: f64 = 7.0;

/// The key a control holds in this room.
fn key_of(control: Control, room: TouchRoom, keys: &KeyBindings) -> KeyCode {
    match (control, room) {
        (Control::A, _) => keys.jump.0,
        (Control::B, TouchRoom::Lobby { voting: true }) => keys.left.0,
        (Control::B, _) => keys.special1.0,
        (Control::C, TouchRoom::Lobby { voting: true }) => keys.right.0,
        // The chat opens and sends with Enter.
        (Control::C, TouchRoom::Lobby { voting: false } | TouchRoom::Waiting) => KeyCode::Enter,
        (Control::C, _) => keys.special2.0,
        (Control::Emotion1, _) => keys.emotion1.0,
        (Control::Emotion2, _) => keys.emotion2.0,
        (Control::Emotion3, _) => keys.emotion3.0,
        (Control::PlayerList, _) => keys.player_list.0,
        (Control::Idle, _) => keys.emotion4.0,
        (Control::Pause, _) => KeyCode::Escape,
        (Control::Joystick, _) => unreachable!("the joystick holds directions"),
    }
}

/// What a big button does in this room, in at most 5 letters (the width of its black
/// middle), as `key_of` decides it.
fn label_of(control: Control, room: TouchRoom) -> Option<&'static str> {
    let label = match (control, room) {
        (Control::A, TouchRoom::Lobby { voting: true }) => "vote",
        (Control::A, TouchRoom::Lobby { voting: false }) => "ready",
        (Control::A, _) => "jump",
        (Control::B, TouchRoom::Lobby { voting: true }) => "<",
        (Control::B, TouchRoom::Lobby { voting: false }) => "exit",
        (Control::B, _) => "sp 1",
        (Control::C, TouchRoom::Lobby { voting: true }) => ">",
        (Control::C, TouchRoom::Lobby { voting: false } | TouchRoom::Waiting) => "chat",
        (Control::C, _) => "sp 2",
        _ => return None,
    };
    Some(label)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(id: u64, placement: Placement) -> Point {
        Point { id, x: placement.x, y: placement.y }
    }

    #[test]
    fn a_finger_on_a_button_holds_its_key_until_it_slides_off() {
        let (layout, keys) = (TouchLayout::default(), KeyBindings::default());
        let mut touch = TouchControls::default();
        let held = touch.update(&[point(1, layout.a)], TouchRoom::Play, &layout, &keys);
        assert_eq!(held, HashSet::from([keys.jump.0]));
        let away = Point { id: 1, x: layout.a.x - 100.0, y: layout.a.y };
        assert!(touch.update(&[away], TouchRoom::Play, &layout, &keys).is_empty(), "slid off");
    }

    #[test]
    fn the_joystick_keeps_its_finger_and_leans_both_ways() {
        let (layout, keys) = (TouchLayout::default(), KeyBindings::default());
        let mut touch = TouchControls::default();
        touch.update(&[point(7, layout.joystick)], TouchRoom::Play, &layout, &keys);
        let far = Point { id: 7, x: layout.joystick.x + 300.0, y: layout.joystick.y - 30.0 };
        let held = touch.update(&[far], TouchRoom::Play, &layout, &keys);
        assert_eq!(held, HashSet::from([keys.right.0, keys.up.0]), "dragged far out, still held");
    }

    #[test]
    fn the_lobby_has_only_the_big_buttons_and_they_pick_the_map_while_voting() {
        let (layout, keys) = (TouchLayout::default(), KeyBindings::default());
        let mut touch = TouchControls::default();
        let voting = TouchRoom::Lobby { voting: true };
        assert!(touch.update(&[point(1, layout.joystick)], voting, &layout, &keys).is_empty());
        assert_eq!(touch.update(&[point(1, layout.b)], voting, &layout, &keys), HashSet::from([keys.left.0]));
        assert!(touch.update(&[point(1, layout.a)], TouchRoom::Off, &layout, &keys).is_empty());
    }

    #[test]
    fn every_shown_button_says_what_it_does() {
        let rooms = [TouchRoom::Lobby { voting: false }, TouchRoom::Lobby { voting: true }, TouchRoom::Waiting, TouchRoom::Play, TouchRoom::Sandbox];
        for room in rooms {
            for control in CONTROLS.into_iter().filter(|control| *control != Control::Joystick && control.is_used_in(room)) {
                let said = if control.is_big_button() { label_of(control, room).is_some() } else { control.icon(false).is_some() };
                assert!(said, "{control:?} in {room:?}");
            }
        }
        let longest = rooms.iter().flat_map(|&room| [Control::A, Control::B, Control::C].map(|control| label_of(control, room))).flatten().map(str::len).max();
        assert!(longest <= Some(5), "a label wider than the button's black middle");
    }

    #[test]
    fn the_sandbox_has_no_player_list_button() {
        let (layout, keys) = (TouchLayout::default(), KeyBindings::default());
        let mut touch = TouchControls::default();
        // The joystick grabs this finger too (its circle reaches the player list), so only
        // the player list's own key is looked at.
        assert!(!touch.update(&[point(1, layout.player_list)], TouchRoom::Sandbox, &layout, &keys).contains(&keys.player_list.0));
        assert!(touch.update(&[point(2, layout.player_list)], TouchRoom::Play, &layout, &keys).contains(&keys.player_list.0));
    }
}
