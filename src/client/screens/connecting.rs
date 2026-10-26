//! room_connecting (obj_connecting): shown while ENet connects to the server.

use super::{count_down_alarm, draw_back_button, go_to_error, is_cancel_button_clicked};
use crate::client::net::{self, Notice};
use crate::client::canvas::Canvas;
use crate::client::room::Room;
use crate::client::Context;
use anyhow::Result;
use crate::core::rooms::ids::RoomId;
use crate::core::config::ticks;

/// alarm[0]: steps before the connection attempt starts, so the room shows first.
const CONNECT_DELAY_SECONDS: f64 = 10.0 / 60.0;
/// global.errorCode when the address cannot be connected to.
const CANNOT_CONNECT_ERROR: u8 = 0;
/// spr_menu_connection's words, written in the menu lettering. The dots come and go 10
/// times a second, as the picture's four frames did; the words start where its left
/// edge stood, so they stay put while the dots change.
const CONNECTING_WORDS: &str = "connecting to the server";
const MOST_DOTS: usize = 3;
const DOTS_PER_SECOND: f64 = 10.0;
const WORDS_CORNER: (f64, f64) = (153.0, 131.0);

/// The words of spr_menu_connection with `dots` dots after them, from (left, top).
pub fn draw_connecting_words(canvas: &mut Canvas, left: f64, top: f64, dots: usize) {
    super::draw_words_with_dots(canvas, left, top, CONNECTING_WORDS, dots);
}

pub struct Connecting {
    room: Room,
    connect_alarm: i32,
}

impl Connecting {
    pub fn open(context: &mut Context) -> Result<Connecting> {
        Ok(Connecting { room: Room::load(&context.resources_folder, RoomId::Connecting, &context.canvas.sprites)?, connect_alarm: ticks(CONNECT_DELAY_SECONDS) })
    }

    pub fn tick(&mut self, context: &mut Context) -> Option<RoomId> {
        let next_room = self.step(context);
        self.room.draw_layers_with(&mut context.canvas, |_, _| {});
        let dots = (macroquad::time::get_time() * DOTS_PER_SECOND) as usize % (MOST_DOTS + 1);
        let (left, top) = WORDS_CORNER;
        draw_connecting_words(&mut context.canvas, left, top, dots);
        draw_back_button(&mut context.canvas);
        next_room
    }

    fn step(&mut self, context: &mut Context) -> Option<RoomId> {
        self.room.step(&context.canvas.sprites);
        if count_down_alarm(&mut self.connect_alarm) && !context.net.connect() {
            return Some(go_to_error(context, CANNOT_CONNECT_ERROR));
        }
        if context.input.pressed(context.options.keys.jump.0) || is_cancel_button_clicked(context) {
            return Some(net::reset(context));
        }
        for notice in net::update(context) {
            match notice {
                Notice::GoTo(room) => return Some(room),
                Notice::ShowError(code) => return Some(go_to_error(context, code)),
                _ => {}
            }
        }
        None
    }
}
