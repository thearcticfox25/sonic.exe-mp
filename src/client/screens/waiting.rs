//! room_waiting (obj_menu_waiting): shown right after connecting, and, without
//! states.gameplay.allow_spectators, while a round is running that this player
//! waits for. Lists the players of the round, shows the round clock and the chat.
//! With spectators allowed the round's own screens are shown instead, from the
//! stage the round is at (client/net.rs).

use super::chat::Chat;
use super::vote_kick::{PickerOutcome, PlayerPicker};
use super::round_clock::RoundClock;
use super::{draw_back_button, go_to_error, is_cancel_button_clicked};
use crate::client::net::{self, Notice};
use crate::client::room::Room;
use crate::client::text::{draw_text, text_width};
use crate::client::Context;
use anyhow::{Context as _, Result};
use macroquad::input::KeyCode;
use std::collections::BTreeMap;
use crate::core::resources::names::{sound, sprite};
use crate::core::resources::SpriteId;
use crate::core::objects::ids::ObjectId;
use crate::core::rooms::ids::RoomId;
use crate::core::world::C_WHITE;

const CHAT_MAX_LENGTH: usize = 36;
const CHAT_LINES_TOP: f64 = 182.0;
const CHAT_BOX_TOP: f64 = 251.0;
/// obj_menu_waiting stands where spr_menu_waiting's origin (95, 4) is.
const WAITING_ORIGIN: (f64, f64) = (95.0, 4.0);
/// Players of the round in a row, like the lobby.
const PLAYER_ROW_LEFT: f64 = 40.0;
const PLAYER_ROW_STEP: f64 = 80.0;
const PLAYER_ROW_Y: f64 = 58.0;
const NICKNAME_BELOW_ICON: f64 = 16.0;
const ICON_GROWTH_PER_STEP: f64 = 0.2;

pub struct Waiting {
    room: Room,
    waiting_layer: usize,
    waiting_position: (f64, f64),
    /// spr_menu_waiting while a round runs; until then the words of spr_menu_connection.
    waits_for_round: bool,
    draw_clock: bool,
    clock: RoundClock,
    players: BTreeMap<u16, WaitingPlayer>,
    chat: Chat,
    /// obj_menu_waitkick: picking who to vote kick (or, for an operator, who to kick,
    /// ban or make an operator), which also puts the vote kick sign up.
    picker: Option<PlayerPicker>,
    /// obj_minigame: offered while a round runs, played in the room's Minigame layers.
    minigame: super::minigame::Minigame,
}

struct WaitingPlayer {
    nickname: String,
    /// In the running round: shown by their character, else by their lobby icon.
    in_round: bool,
    icon: u8,
    exe: bool,
    character: u8,
    /// Grows to 1 when the icon appears.
    scale: f64,
}

impl Waiting {
    pub fn open(context: &mut Context) -> Result<Waiting> {
        let room = Room::load(&context.resources_folder, RoomId::Waiting, &context.canvas.sprites)?;
        let (waiting_layer, placed) = room.placed(ObjectId::MenuWaiting).next().context("room_waiting has no obj_menu_waiting")?;
        let waiting_position = (placed.x, placed.y);
        Ok(Waiting {
            room,
            waiting_layer,
            waiting_position,
            waits_for_round: false,
            draw_clock: false,
            clock: RoundClock::default(),
            players: BTreeMap::new(),
            chat: Chat::new(true),
            picker: None,
            minigame: super::minigame::Minigame::new(),
        })
    }

    pub fn tick(&mut self, context: &mut Context) -> Option<RoomId> {
        let next_room = self.step(context);
        self.draw(context);
        next_room
    }

    fn step(&mut self, context: &mut Context) -> Option<RoomId> {
        self.room.step(&context.canvas.sprites);
        // obj_menu_waiting KeyPress_13 and KeyPress_84: no chat while the game is on.
        if self.picker.is_none() && !self.minigame.ingame {
            if context.input.pressed(KeyCode::Enter) {
                self.chat.toggle(context);
            }
            if context.input.pressed(KeyCode::T) {
                self.chat.open(&mut context.input);
            }
        }
        let back_pressed = self.picker.is_none() && !self.chat.is_open && context.input.pressed(context.options.keys.special1.0);
        if back_pressed || is_cancel_button_clicked(context) {
            return Some(net::reset(context));
        }
        if self.minigame.shown {
            self.chat.type_message(&mut context.input, CHAT_MAX_LENGTH);
        }
        let paused = self.chat.is_open || self.picker.is_some();
        self.minigame.step_with_audio(context, paused);
        self.show_minigame_layers();
        for notice in net::update(context) {
            if let Some(room) = self.on_notice(context, notice) {
                return Some(room);
            }
        }
        None
    }

    fn on_notice(&mut self, context: &mut Context, notice: Notice) -> Option<RoomId> {
        match notice {
            Notice::GoTo(room) => return Some(room),
            Notice::ShowError(code) => return Some(go_to_error(context, code)),
            // A spectator who came in during the character select stage watches it.
            Notice::CharacterTime(_) => return Some(RoomId::Lobby),
            Notice::WaitForRound => {
                self.waits_for_round = true;
                self.minigame.show();
            }
            Notice::WaitingPlayer { id, nickname, in_round, exe, character, icon } => {
                self.players.insert(id, WaitingPlayer { nickname, in_round, icon, exe, character, scale: 0.0 });
            }
            Notice::WaitingPlayerLeft(id) => {
                if self.players.remove(&id).is_some() {
                    context.audio.play(sound::SND_HURT, false);
                }
            }
            Notice::RoundTime(timer) => self.sync_clock(context, timer),
            Notice::ChatMessage { sender, text } => self.chat.add_message(context, &sender, &text),
            Notice::ChoosePlayer(request) => {
                self.chat.add_message(context, "\\()~", "choose the player");
                self.chat.add_message(context, "\\()~", "(press \\x~ to cancel)");
                self.picker = Some(PlayerPicker::new(request));
                self.chat.typed.clear();
                context.input.keyboard_string.clear();
            }
            _ => {}
        }
        None
    }

    /// obj_menu_waitkick Draw_0: the arrow over the chosen player, and what the keys do.
    fn draw_picker(&mut self, context: &mut Context) {
        let Some(picker) = self.picker.as_mut() else { return };
        let others: Vec<u16> = self.players.keys().copied().filter(|id| Some(*id) != context.net.id).collect();
        let time_ms = macroquad::time::get_time() * 1000.0;
        if matches!(picker.draw(context, &others, time_ms), PickerOutcome::Closed) {
            self.picker = None;
        }
    }

    /// layer_set_visible("Minigame", ...) and ("Minigame2", ...): shown while the game is on.
    fn show_minigame_layers(&mut self) {
        for name in ["Minigame", "Minigame2"] {
            if let Some(layer) = self.room.layer_mut(name) {
                layer.visible = self.minigame.ingame;
            }
        }
    }

    fn sync_clock(&mut self, context: &mut Context, timer: u16) {
        self.draw_clock = true;
        self.clock.sync(&mut context.audio, timer);
    }

    fn draw(&mut self, context: &mut Context) {
        let (x, y) = self.waiting_position;
        let (waiting_layer, waits_for_round) = (self.waiting_layer, self.waits_for_round);
        let minigame_layer = self.room.layers.iter().position(|layer| layer.name == "Minigame");
        let minigame = &self.minigame;
        self.room.draw_layers_with(&mut context.canvas, |canvas, layer| {
            if layer == waiting_layer && waits_for_round {
                canvas.draw_sprite(sprite::SPR_MENU_WAITING, 0.0, x, y);
            } else if layer == waiting_layer {
                super::connecting::draw_connecting_words(canvas, x - WAITING_ORIGIN.0, y - WAITING_ORIGIN.1, 0);
            }
            if Some(layer) == minigame_layer {
                minigame.draw(canvas);
            }
        });
        draw_back_button(&mut context.canvas);
        self.draw_gui(context);
    }

    /// Draw_64
    fn draw_gui(&mut self, context: &mut Context) {
        let canvas = &mut context.canvas;
        if context.audio.is_playing(sound::MUS_WAITING) || context.audio.is_playing(sound::MUS_MINIGAME) {
            self.chat.draw(canvas, CHAT_LINES_TOP, CHAT_BOX_TOP);
        }
        self.minigame.draw_gui(canvas, &context.options.keys, macroquad::time::get_time() * 1000.0);
        // obj_menu_waiting Draw_64: the players are not listed over the game.
        for (index, player) in self.players.values_mut().enumerate().filter(|_| !self.minigame.ingame) {
            let x = PLAYER_ROW_LEFT + index as f64 * PLAYER_ROW_STEP;
            let (icon_sprite, frame) = player_icon(player);
            canvas.draw_sprite_ext(icon_sprite, frame, x, PLAYER_ROW_Y, player.scale, 1.0, 0.0, C_WHITE, 1.0);
            draw_text(canvas, x - text_width(canvas, &player.nickname) / 2.0, PLAYER_ROW_Y + NICKNAME_BELOW_ICON, &player.nickname, C_WHITE, 1.0);
            player.scale = if player.scale < 1.0 { player.scale + ICON_GROWTH_PER_STEP } else { 1.0 };
        }
        if self.picker.is_some() {
            super::draw_choose_player_sign(canvas);
            self.draw_picker(context);
            return;
        }
        if !self.draw_clock {
            return;
        }
        self.clock.draw(canvas);
    }
}

/// The icon of a player: the character they play in the round, or their lobby icon.
/// obj_menu_waitkick: the arrow walks the same row of icons the room draws.

fn player_icon(player: &WaitingPlayer) -> (SpriteId, f64) {
    if !player.in_round {
        return (sprite::SPR_LOBBY_DICON, player.icon as f64);
    }
    if !player.exe {
        // spr_lobby_icon: "?" and Sonic.EXE, then Tails (character 1) on.
        return (sprite::SPR_LOBBY_ICON, (player.character + 1) as f64);
    }
    let exe_icon = match player.character {
        1 => sprite::SPR_LOBBY_EXEICON,
        2 => sprite::SPR_LOBBY_EXEICON2,
        3 => sprite::SPR_LOBBY_EXEICON3,
        4 => sprite::SPR_LOBBY_EXEICON4,
        _ => sprite::SPR_LOBBY_DICON,
    };
    (exe_icon, 0.0)
}
