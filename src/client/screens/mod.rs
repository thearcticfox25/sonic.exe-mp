//! One screen per GameMaker room the client shows. Every tick a screen runs one
//! step and draws one picture; asking for another room (room_goto) switches the
//! screen after that tick, as GameMaker does at the end of a step.

mod character_art;
mod chat;
mod connecting;
mod fades;
mod host_actions;
mod level;
mod lobby;
mod logo;
mod map_preview;
mod menu;
mod message;
mod minigame;
mod results;
mod round_clock;
mod vote_kick;
mod waiting;

use crate::client::options::{self, FULLSCREEN_MODE};
use crate::client::Context;
use anyhow::{bail, Result};
use macroquad::input::{mouse_position, KeyCode};
use crate::core::rooms::ids::RoomId;

pub use results::ResultsRow;

pub enum Screen {
    Logo(logo::Logo),
    Menu(menu::Menu),
    Connecting(connecting::Connecting),
    Message(message::Message),
    Waiting(waiting::Waiting),
    Lobby(lobby::Lobby),
    Level(level::Level),
    Results(results::Results),
}

/// alarm[n] == -1
pub const ALARM_OFF: i32 = -1;
/// The back button of the connecting, waiting and message rooms: the word in the menu
/// lettering, red as a selected menu button, where the picture button_back was. It
/// answers the mouse around its word as the menu's buttons do (the original's
/// rectangle, 223, 153, 32x16, sat half below the picture).
const BACK_LABEL: &str = "back";
const BACK_LABEL_CORNER: (f64, f64) = (224.0, 148.0);

impl Screen {
    /// room_goto
    pub fn open(room: RoomId, context: &mut Context) -> Result<Screen> {
        Ok(match room {
            RoomId::Logo => Screen::Logo(logo::Logo::open(context)?),
            RoomId::Menu => Screen::Menu(menu::Menu::open(context)?),
            RoomId::Connecting => Screen::Connecting(connecting::Connecting::open(context)?),
            RoomId::Message => Screen::Message(message::Message::open(context)?),
            RoomId::Waiting => Screen::Waiting(waiting::Waiting::open(context)?),
            RoomId::Lobby => Screen::Lobby(lobby::Lobby::open(context)?),
            RoomId::Results => Screen::Results(results::Results::open(context)?),
            level if crate::client::levels::LEVELS.iter().any(|known| known.room == level) => Screen::Level(level::Level::open(context, level)?),
            other => bail!("room {other:?} is not ported yet"),
        })
    }

    /// What the touch controls do here (obj_controls Room Start).
    fn touch_room(&self) -> crate::client::touch::TouchRoom {
        use crate::client::touch::TouchRoom;
        match self {
            Screen::Logo(_) | Screen::Menu(_) | Screen::Connecting(_) | Screen::Results(_) => TouchRoom::Off,
            Screen::Lobby(lobby) => TouchRoom::Lobby { voting: lobby.is_voting() },
            Screen::Waiting(_) => TouchRoom::Waiting,
            Screen::Level(level) if level.is_sandbox() => TouchRoom::Sandbox,
            Screen::Level(_) | Screen::Message(_) => TouchRoom::Play,
        }
    }

    /// One step and one picture. Returns the room to go to, if the screen asked for one.
    pub fn tick(&mut self, context: &mut Context) -> Option<RoomId> {
        let (mouse_x, mouse_y) = mouse_position();
        let mouse = context.canvas.window_to_view(mouse_x, mouse_y);
        context.input.begin_tick(mouse);
        let mut control_keys = context.gamepad.held_keys(&context.options.keys);
        if context.options.touch_controls {
            let points = crate::client::touch::points(&context.canvas);
            control_keys.extend(context.touch.update(&points, self.touch_room(), &context.options.touch_layout, &context.options.keys));
            context.touch.hud_shown = !matches!(self, Screen::Level(level) if !level.hud_shown());
        }
        context.input.hold_control_keys(control_keys);
        toggle_fullscreen_on_f4(context);
        context.canvas.begin();
        match self {
            Screen::Logo(logo) => logo.tick(context),
            Screen::Menu(menu) => menu.tick(context),
            Screen::Connecting(connecting) => connecting.tick(context),
            Screen::Message(message) => message.tick(context),
            Screen::Waiting(waiting) => waiting.tick(context),
            Screen::Lobby(lobby) => lobby.tick(context),
            Screen::Level(level) => level.tick(context),
            Screen::Results(results) => results.tick(context),
        }
    }
}

/// alarm[n]: counts down one step; true on the step the alarm rings.
pub fn count_down_alarm(alarm: &mut i32) -> bool {
    if *alarm <= 0 {
        return false;
    }
    *alarm -= 1;
    if *alarm > 0 {
        return false;
    }
    *alarm = ALARM_OFF;
    true
}

/// global.errorCode = code; room_goto(room_message)
pub fn go_to_error(context: &mut Context, code: u8) -> RoomId {
    context.error_code = code;
    RoomId::Message
}

/// Any mouse button pressed on the back button.
pub fn is_cancel_button_clicked(context: &Context) -> bool {
    let (left, top) = BACK_LABEL_CORNER;
    let right = left + context.canvas.menu_text_width(BACK_LABEL);
    let (x, y) = (context.input.mouse_x, context.input.mouse_y);
    let horizontal = (left - menu::LABEL_CLICK_MARGIN_X..right + menu::LABEL_CLICK_MARGIN_X).contains(&x);
    let vertical = (top - menu::LABEL_CLICK_MARGIN_Y..top + menu::LABEL_INK_HEIGHT + menu::LABEL_CLICK_MARGIN_Y).contains(&y);
    context.input.mouse_any_pressed() && horizontal && vertical
}

/// Words in the menu lettering followed by `dots` dots, from (left, top): the signs that
/// were pictures of four frames, the words and 0 to 3 dots (spr_menu_connection,
/// spr_lobby_waiting). Written from the left edge the picture had, they stay put while
/// the dots come and go.
pub fn draw_words_with_dots(canvas: &mut crate::client::canvas::Canvas, left: f64, top: f64, words: &str, dots: usize) {
    let text = format!("{words}{}", ".".repeat(dots));
    canvas.draw_menu_text(left, top, &text, crate::core::world::C_WHITE, 1.0);
}

/// The sign at the top of the lobby and the waiting room, where spr_lobby_waiting and
/// spr_lobby_votekick stood: from (240, 8), their origin the middle of their top edge.
const SIGN_TOP: f64 = 8.0;
const SIGN_CENTER: f64 = 240.0;
/// spr_lobby_waiting's left edge: 76 px left of its origin.
const WAITING_SIGN_LEFT: f64 = SIGN_CENTER - 76.0;

/// spr_lobby_waiting: "waiting for players" and 0 to 3 dots.
pub fn draw_waiting_sign(canvas: &mut crate::client::canvas::Canvas, dots: usize) {
    draw_words_with_dots(canvas, WAITING_SIGN_LEFT, SIGN_TOP, "waiting for players", dots);
}

/// spr_lobby_votekick: "choose player", which never changes, centered.
pub fn draw_choose_player_sign(canvas: &mut crate::client::canvas::Canvas) {
    let words = "choose player";
    let left = (SIGN_CENTER - canvas.menu_text_width(words) / 2.0).floor();
    canvas.draw_menu_text(left, SIGN_TOP, words, crate::core::world::C_WHITE, 1.0);
}

pub fn draw_back_button(canvas: &mut crate::client::canvas::Canvas) {
    let (left, top) = BACK_LABEL_CORNER;
    canvas.draw_menu_text(left, top, BACK_LABEL, menu::SELECTED_BLEND, 1.0);
}

/// obj_controls Begin Step: F4 switches to fullscreen and back to the last window size.
fn toggle_fullscreen_on_f4(context: &mut Context) {
    if !context.input.pressed(KeyCode::F4) {
        return;
    }
    let options = &mut context.options;
    if options.screen_mode == FULLSCREEN_MODE {
        options.screen_mode = context.windowed_screen_mode;
    } else {
        context.windowed_screen_mode = options.screen_mode;
        options.screen_mode = FULLSCREEN_MODE;
    }
    options::apply_screen_mode(options.screen_mode);
    options.save();
}
