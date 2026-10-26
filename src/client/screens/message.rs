//! room_message (obj_message): why the connection failed or ended.

use super::{draw_back_button, is_cancel_button_clicked};
use crate::client::net;
use crate::client::room::Room;
use crate::client::Context;
use anyhow::{Context as _, Result};
use crate::core::objects::ids::ObjectId;
use crate::core::rooms::ids::RoomId;
use crate::client::canvas::{make_color_rgb, Canvas};

/// The message's lettering and color: the menu's, red, as spr_menu_error drew it.
const MESSAGE_COLOR: u32 = make_color_rgb(0xFF, 0x00, 0x00);
/// spr_menu_error's origin was the middle of its words, 4 px below their top.
const MESSAGE_ABOVE_ORIGIN: f64 = 4.0;

/// global.errorCode: the server's reason for a disconnect (DisconnectReason in
/// server.rs), or the client's own (0 and 4). The words spr_menu_error had, by frame.
fn message_text(code: u8) -> &'static str {
    match code {
        0 => "failed to connect",
        1 => "kicked by the host",
        2 => "banned by the host",
        3 => "version outdated",
        // No word from the server for too long: the UDP timeout.
        4 => "server timeout",
        5 => "packets weren't received",
        6 => "game has already started",
        7 => "afk or timeout",
        8 => "lobby is full",
        9 => "you are rate limited",
        10 => "server shutdown",
        11 => "ip is already in use",
        // DisconnectReason::Other (255) and anything newer than this client.
        _ => "disconnected",
    }
}

/// The message centered on obj_message.
fn draw_message(canvas: &mut Canvas, code: u8, x: f64, y: f64) {
    let text = message_text(code);
    let left = (x - canvas.menu_text_width(text) / 2.0).floor();
    canvas.draw_menu_text(left, y - MESSAGE_ABOVE_ORIGIN, text, MESSAGE_COLOR, 1.0);
}

pub struct Message {
    room: Room,
    message_layer: usize,
    message_position: (f64, f64),
}

impl Message {
    pub fn open(context: &mut Context) -> Result<Message> {
        let room = Room::load(&context.resources_folder, RoomId::Message, &context.canvas.sprites)?;
        let (message_layer, placed) = room.placed(ObjectId::Message).next().context("room_message has no obj_message")?;
        let message_position = (placed.x, placed.y);
        // Create_0: the connection is gone for good.
        context.net = net::NetClient::new();
        context.audio.stop_music();
        Ok(Message { room, message_layer, message_position })
    }

    pub fn tick(&mut self, context: &mut Context) -> Option<RoomId> {
        self.room.step(&context.canvas.sprites);
        let next_room = (context.input.pressed(context.options.keys.jump.0) || is_cancel_button_clicked(context)).then(|| net::reset(context));
        let code = context.error_code;
        let (x, y) = self.message_position;
        let message_layer = self.message_layer;
        self.room.draw_layers_with(&mut context.canvas, |canvas, layer| {
            if layer == message_layer {
                draw_message(canvas, code, x, y);
            }
        });
        draw_back_button(&mut context.canvas);
        next_room
    }
}
