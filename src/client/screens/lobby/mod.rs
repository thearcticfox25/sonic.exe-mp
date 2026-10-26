//! room_lobby: players getting ready, the map vote and the start of a round
//! (obj_lobby with obj_lobby_icon, obj_lobby_prolet, obj_lobby_waiting and
//! obj_lobby_votekick).

mod cards;
mod icons;

use super::character_art::{art_frame, art_index, ART_POSITION, CHARACTER_ART, DESCRIPTION_POSITION};
use super::chat::Chat;
use super::fades::{BlackFadeOut, WhiteFlash};
use super::go_to_error;
use crate::client::canvas::{make_color_rgb, Canvas, C_DKGRAY, C_RED, VIEW_HEIGHT, VIEW_WIDTH};
use crate::client::net::{self, Notice};
use crate::client::room::Room;
use crate::client::text::{draw_counter_centered, draw_text};
use crate::client::Context;
use anyhow::{Context as _, Result};
use cards::{CardsInput, PreferenceCards, EXE_NAMES, SURVIVOR_NAMES};
use icons::{LobbyView, PlayerIcon, Splash};
use macroquad::input::KeyCode;
use super::map_preview::MapPreview;
use super::minigame::Minigame;
use crate::packet::{Packet, PacketType, EXE_COUNT, NO_CHARACTER, SURVIVOR_COUNT};
use crate::core::resources::names::{sound, sprite};
use crate::core::objects::ids::ObjectId;
use crate::core::rooms::ids::RoomId;
use crate::core::world::C_WHITE;
use super::host_actions::HostActions;
use super::vote_kick::{PickerOutcome, PlayerPicker};
use crate::core::config::step;

/// obj_lobby.state
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LobbyState {
    Lobby,
    Vote,
    /// obj_lobby state 2: everyone picks their character on their icon.
    CharacterSelect,
    /// After the vote, before the level, when the server hands out the characters
    /// without the character select stage: the map, this player's character and
    /// every character of the round.
    RoundIntro,
}

/// How often the server's ?gui window asks who is connected.
const HOST_WINDOW_LIST_SECONDS: f64 = 2.0;
const CHAT_MAX_LENGTH: usize = 48;
const CHAT_LINES_TOP: f64 = 142.0;
const CHAT_BOX_TOP: f64 = 211.0;
const EXE_CHANCE_TEXT_POSITION: (f64, f64) = (170.0, 256.0);
/// The dots of "waiting for players" step every 300 ms, from none to three.
const SIGN_FRAME_MILLISECONDS: f64 = 300.0;
const SIGN_DOT_STEPS: usize = 4;
const COUNTER_POSITION: (f64, f64) = (240.0, 13.0);
/// The key hints of the bottom bar, where spr_lobby_bars had them written: the first from
/// the left, the second around the middle, the last up to the right.
const HINTS_TOP: f64 = 242.0;
const HINT_LEFT: f64 = 28.0;
const HINT_RIGHT_END: f64 = 459.0;
/// Two hints on top of each other (ready, and choose ahead) are this far apart.
const HINT_LINE_HEIGHT: f64 = 12.0;
/// A countdown number pops in bigger and shrinks back.
const COUNTER_POP_SCALE: f64 = 1.5;
const COUNTER_SHRINK_PER_STEP: f64 = 0.02;
/// Vote and character select timer values that tick audibly, in red on the stage.
const WARNING_SECONDS: u8 = 3;

/// The three map cards of the vote.
const MAP_CARD_X: [f64; 3] = [16.0, 179.0, 341.0];
const MAP_CARD_TOP: f64 = 58.0;
/// The cards slide down this far while fading in.
const MAP_CARD_SLIDE: f64 = 20.0;
const MAP_CARD_FADE_PER_STEP: f64 = 0.016;
/// The selected card jumps up this far and settles.
const MAP_CARD_JUMP: f64 = 4.0;
const MAP_CARD_JUMP_FALL_PER_STEP: f64 = 0.32;
const VOTE_COUNT_OFFSET: (f64, f64) = (61.0, 53.0);
const MAP_DESCRIPTION_POSITION: (f64, f64) = (4.0, 233.0);
const VOTED_MAP_BLEND: u32 = make_color_rgb(0x27, 0xFF, 0x23);

/// The round intro, laid out as the character select stage of obj_lobby.
/// The chosen map's card, at the right under the top bar.
const INTRO_MAP_CARD_POSITION: (f64, f64) = (341.0, 32.0);
/// The other players' characters stand small and dimmed under the card, right
/// of this player's character, spread between these x.
const INTRO_CROWD_LEFT: f64 = 230.0;
const INTRO_CROWD_RIGHT: f64 = 480.0;
const INTRO_CROWD_SCALE: f64 = 0.5;
const INTRO_CROWD_BLEND: u32 = C_DKGRAY;
/// spr_char_bars has the title "select the character" in its top bar; there is
/// no selection here, so the top bar is its title-free left end stretched.
const CHAR_BARS_TOP_HEIGHT: f64 = 27.0;
const CHAR_BARS_UNTITLED_WIDTH: f64 = 64.0;
/// The intro stays this long once the round starts, then fades (obj_lobby state 3).
/// The original faded at once after a selection stage; here the round starts in
/// the same server tick as the vote result, so the intro would only be seen fading.
const INTRO_HOLD_SECONDS: f64 = 2.0;
const INTRO_FADE_PER_STEP: f64 = 0.01;

pub struct Lobby {
    room: Room,
    /// The layer of obj_lobby, which also holds obj_lobby_waiting.
    lobby_layer: usize,
    preview_layer: usize,
    state: LobbyState,
    countdown: u8,
    is_counting: bool,
    /// isReady: the local player pressed ready.
    is_ready: bool,
    count_scale: f64,
    chat: Chat,
    /// The vote: the three maps, the one under the cursor, and the votes so far.
    maps: [u8; 3],
    chosen_map: usize,
    chosen_map_jump: f64,
    votes: [u8; 3],
    /// This player may still vote: not after voting, and never while watching.
    can_vote: bool,
    map_fade: f64,
    vote_timer: u8,
    /// The character select stage: its seconds left, and the character under the
    /// cursor (selected), 1-based of this player's side.
    character_timer: u8,
    cursor: u8,
    /// canUse: the server confirmed this player, so the lobby takes input.
    can_use: bool,
    icons: Vec<PlayerIcon>,
    /// The page of the row of other players on the screen (icons::row_pages).
    row_page: usize,
    splashes: Vec<Splash>,
    preview: MapPreview,
    /// obj_lobby_votekick
    picker: Option<PlayerPicker>,
    /// The server's ?gui window: what to do to the player it picked (host_actions.rs).
    host_actions: Option<HostActions>,
    /// The ?gui window asks again who is connected when this runs out.
    list_refresh_ticks: i32,
    /// The preference cards that replace the character select stage.
    cards: PreferenceCards,
    flash: Option<WhiteFlash>,
    fade_in: Option<BlackFadeOut>,
    /// The round intro: its map, ticks shown, and the fade into the level once the round starts.
    round_map: Option<u16>,
    intro_ticks: f64,
    round_starting_ticks: Option<i32>,
    fade_out: f64,
    /// The waiting room's minigame, for whoever watches the round (states::spectate).
    minigame: Minigame,
}

impl Lobby {
    pub fn open(context: &mut Context) -> Result<Lobby> {
        let room = Room::load(&context.resources_folder, RoomId::Lobby, &context.canvas.sprites)?;
        let (lobby_layer, _) = room.placed(ObjectId::Lobby).next().context("room_lobby has no obj_lobby")?;
        let (preview_layer, _) = room.placed(ObjectId::LobbyProlet).next().context("room_lobby has no obj_lobby_prolet")?;
        // obj_lobby_waiting, also placed in room_lobby, is not drawn: it was a leftover
        // copy of the sign obj_lobby draws over the bars, lying under them and peeking out.
        let net = &context.net;
        let fade_in = (net.is_connected && net.is_ready && net.id.is_some()).then(BlackFadeOut::new);
        let mut lobby = Lobby {
            room,
            lobby_layer,
            preview_layer,
            state: LobbyState::Lobby,
            countdown: 0,
            is_counting: false,
            is_ready: false,
            count_scale: 1.0,
            chat: Chat::new(true),
            maps: [0; 3],
            chosen_map: 0,
            chosen_map_jump: MAP_CARD_JUMP,
            votes: [0; 3],
            can_vote: false,
            map_fade: 0.0,
            vote_timer: 20,
            character_timer: 0,
            cursor: 1,
            can_use: context.net.observer_key.is_some(),
            icons: Vec::new(),
            row_page: 0,
            splashes: Vec::new(),
            preview: MapPreview::new(),
            // The server's ?gui window does nothing here but pick players.
            picker: context.net.observer_key.is_some().then(PlayerPicker::for_host_window),
            host_actions: None,
            list_refresh_ticks: 0,
            cards: PreferenceCards::new(&context.options),
            flash: None,
            fade_in,
            round_map: None,
            intro_ticks: 0.0,
            round_starting_ticks: None,
            fade_out: 0.0,
            minigame: Minigame::new(),
        };
        let past_lobby = matches!(context.net.state, net::NetState::Vote | net::NetState::CharSelect);
        if past_lobby && context.net.observer_key.is_none() {
            lobby.enter_running_stage(context);
        } else {
            // RoomCreationCode
            context.audio.play_music(sound::MUS_LOBBY);
        }
        Ok(lobby)
    }

    /// What the icons need to know about the lobby.
    fn view<'a>(&self, net: &'a net::NetClient) -> LobbyView<'a> {
        let is_picking = self.is_picking(net);
        LobbyView {
            state: self.state,
            is_ready: self.is_ready,
            net,
            row_page: self.row_page,
            is_picking,
            cursor: self.cursor,
            cursor_taken: is_picking && is_taken(net, self.cursor),
        }
    }

    /// On the character select stage and still choosing.
    fn is_picking(&self, net: &net::NetClient) -> bool {
        self.state == LobbyState::CharacterSelect && self.can_use && !net.is_spectating && net.character == NO_CHARACTER
    }

    /// Opened in the middle of a round's way to its level: by a player who came in
    /// while it is voted or picked for (states.gameplay.allow_spectators), or by
    /// everyone between the rounds of a tournament, who come from the results. The
    /// players are the ones the client already knows, the server confirmed this one
    /// long ago, and the stage is the one the client is at.
    fn enter_running_stage(&mut self, context: &mut Context) {
        self.can_use = true;
        self.sync_icons_with_players(context);
        if let (Some(id), false) = (context.net.id, context.net.is_spectating) {
            let nickname = context.options.nickname.clone();
            self.player_joined(context, id, nickname, false, false);
        }
        match context.net.state {
            net::NetState::Vote => self.start_vote(context),
            net::NetState::CharSelect => self.exe_chosen(context.net.level.unwrap_or(0), context),
            _ => {}
        }
    }

    /// The music of the stage the lobby shows; whoever watches the round hears the
    /// waiting room's instead, unless their minigame plays its own.
    fn play_stage_music(&self, context: &mut Context) {
        let music = match self.state {
            _ if context.net.is_spectating => sound::MUS_WAITING,
            LobbyState::Lobby | LobbyState::Vote => sound::MUS_LOBBY,
            LobbyState::CharacterSelect | LobbyState::RoundIntro => sound::MUS_LOBBY_EPIC,
        };
        if !self.minigame.ingame && !context.audio.is_playing(music) {
            context.audio.play_music(music);
        }
    }

    /// SERVER_VOTE_MAPS: the three cards. Whoever watches the round sees them as
    /// they are after voting, without a vote of their own, and may play the minigame.
    fn start_vote(&mut self, context: &mut Context) {
        self.state = LobbyState::Vote;
        self.maps = context.net.vote_maps;
        self.can_vote = !context.net.is_spectating;
        for icon in &mut self.icons {
            icon.hidden = true;
        }
        self.room.set_layer_visible("Background", false);
        if context.net.is_spectating {
            self.minigame.show();
        }
        self.play_stage_music(context);
    }

    /// obj_lobby.state != 0: past the lobby itself, where the touch controls pick the map.
    pub fn is_voting(&self) -> bool {
        self.state != LobbyState::Lobby
    }

    pub fn tick(&mut self, context: &mut Context) -> Option<RoomId> {
        let current_time_ms = (macroquad::time::get_time() * 1000.0).floor();
        let next_room = self.step(context);
        self.draw(context, current_time_ms);
        next_room
    }

    fn step(&mut self, context: &mut Context) -> Option<RoomId> {
        self.room.step(&context.canvas.sprites);
        if context.net.observer_key.is_some() {
            return self.step_as_host_window(context);
        }
        // Whoever watches the round keeps the chat past the lobby, as the waiting room does.
        let chat_stage = self.state == LobbyState::Lobby || context.net.is_spectating;
        let chat_allowed = chat_stage && self.can_use && self.picker.is_none() && !self.minigame.ingame;
        if chat_allowed && context.input.pressed(KeyCode::Enter) {
            self.chat.toggle(context);
        }
        if chat_allowed && context.input.pressed(KeyCode::T) {
            self.chat.open(&mut context.input);
        }
        if let Some(room) = self.step_input(context) {
            return Some(room);
        }
        self.minigame.step_with_audio(context, self.chat.is_open);
        if let Some(level_room) = self.step_round_start() {
            return Some(level_room);
        }

        // A player leaving may take the last page with them.
        let last_page = icons::row_pages(context.net.players.len()).saturating_sub(1);
        self.row_page = self.row_page.min(last_page);
        let view = self.view(&context.net);
        for icon in &mut self.icons {
            icon.step(&view, &context.canvas.sprites);
        }
        self.splashes.retain_mut(|splash| splash.step(&context.canvas.sprites));
        self.flash = self.flash.take().and_then(|mut flash| flash.step().then_some(flash));
        self.fade_in = self.fade_in.take().and_then(|mut fade| fade.step().then_some(fade));

        for notice in net::update(context) {
            if let Some(room) = self.on_notice(context, notice) {
                return Some(room);
            }
        }
        None
    }

    /// The server's ?gui window: everyone connected stands in the row to pick from, kept
    /// up to date by asking the server every few seconds; the rounds are watched in the
    /// level, and nothing else of the lobby (the vote, the intro) is the window's.
    fn step_as_host_window(&mut self, context: &mut Context) -> Option<RoomId> {
        self.list_refresh_ticks -= 1;
        if self.list_refresh_ticks <= 0 {
            self.list_refresh_ticks = crate::core::config::ticks(HOST_WINDOW_LIST_SECONDS);
            context.net.request_lobby_players();
        }
        self.sync_icons_with_players(context);
        let last_page = icons::row_pages(context.net.players.len()).saturating_sub(1);
        self.row_page = self.row_page.min(last_page);
        let view = self.view(&context.net);
        for icon in &mut self.icons {
            icon.step(&view, &context.canvas.sprites);
        }
        self.splashes.retain_mut(|splash| splash.step(&context.canvas.sprites));
        for notice in net::update(context) {
            match notice {
                Notice::GoTo(room) if room != RoomId::Lobby => return Some(room),
                Notice::ShowError(code) => return Some(go_to_error(context, code)),
                _ => {}
            }
        }
        None
    }

    /// An icon for everyone the client knows of, and none for anyone gone.
    fn sync_icons_with_players(&mut self, context: &mut Context) {
        let players: Vec<(u16, String, bool)> = context.net.players.iter().map(|(&id, player)| (id, player.nickname.clone(), player.is_ready)).collect();
        for (id, nickname, is_ready) in &players {
            self.player_joined(context, *id, nickname.clone(), *is_ready, false);
        }
        let gone: Vec<u16> = self.icons.iter().map(|icon| icon.player).filter(|id| !context.net.players.contains_key(id)).collect();
        for id in gone {
            self.player_left(context, id);
        }
    }

    /// Step_0: typing, moving the vote cursor, paging the row of players, voting,
    /// getting ready, leaving.
    fn step_input(&mut self, context: &mut Context) -> Option<RoomId> {
        if self.chat.is_open {
            self.chat.type_message(&mut context.input, CHAT_MAX_LENGTH);
        }
        if self.cards.is_open {
            if matches!(self.cards.step(context), CardsInput::Confirm) {
                self.get_ready_with_cards(context);
            }
            return None;
        }
        if self.state == LobbyState::CharacterSelect {
            self.step_character_select(context);
            return None;
        }
        let keys = context.options.keys.clone();
        let input = &context.input;
        let can_move_vote_cursor = self.state == LobbyState::Vote && self.can_vote && self.map_fade >= 1.0;
        let can_turn_row_page = self.state == LobbyState::Lobby && !self.chat.is_open && self.picker.is_none();
        let sideways = sideways(context);

        if sideways != 0 {
            if self.can_use && can_move_vote_cursor {
                self.chosen_map = (self.chosen_map as i32 + sideways).rem_euclid(3) as usize;
                self.chosen_map_jump = MAP_CARD_JUMP;
                context.audio.play(sound::SND_MENU_SELECT, false);
            } else if can_turn_row_page {
                self.turn_row_page(context, sideways);
            }
        } else if input.pressed(keys.jump.0) {
            if !self.can_use {
                return None;
            }
            if can_move_vote_cursor {
                let mut packet = Packet::new(PacketType::VOTE);
                let _ = packet.write_u8(self.chosen_map as u8);
                context.net.send_reliable(&packet);
                context.audio.play(sound::SND_MENU_PRESS, false);
                self.can_vote = false;
                return None;
            }
            if self.chat.is_open || self.picker.is_some() || !context.net.is_ready {
                return None;
            }
            if context.net.state == net::NetState::Lobby {
                // Ready with the preference cards as they were last set (special 2 opens
                // them); getting unready, or ready where nobody may pick, only toggles.
                if self.is_ready || !context.net.preselects {
                    self.is_ready = !self.is_ready;
                    self.send_ready_state(context);
                    context.audio.play(sound::SND_READY, false);
                } else {
                    self.get_ready_with_cards(context);
                }
            }
        } else if input.pressed(keys.special2.0) {
            let may_choose = self.can_use && self.picker.is_none() && !self.chat.is_open && context.net.is_ready;
            if may_choose && context.net.preselects && context.net.state == net::NetState::Lobby {
                self.cards.is_open = true;
                context.audio.play(sound::SND_MENU_SELECT, false);
            }
        } else if input.pressed(keys.special1.0) {
            let may_leave = self.can_use && self.picker.is_none() && !self.chat.is_open && context.net.is_ready;
            if may_leave && self.state == LobbyState::Lobby {
                return Some(net::reset(context));
            }
        }
        None
    }

    /// obj_lobby_icon state 2: left and right go through the characters of this
    /// player's side, jump asks the server for the one under the cursor.
    fn step_character_select(&mut self, context: &mut Context) {
        if !self.is_picking(&context.net) {
            return;
        }
        let count = if context.net.is_exe { EXE_COUNT } else { SURVIVOR_COUNT };
        let sideways = sideways(context);
        if sideways != 0 {
            self.cursor = ((self.cursor as i32 - 1 + sideways).rem_euclid(count as i32) + 1) as u8;
            if let Some(own_icon) = self.icons.iter_mut().find(|icon| icon.is_own(&context.net)) {
                own_icon.jump();
            }
            context.audio.play(sound::SND_MENU_SELECT, false);
            return;
        }
        if !context.input.pressed(context.options.keys.jump.0) {
            return;
        }
        if is_taken(&context.net, self.cursor) {
            context.audio.play(sound::SND_NONO, false);
            return;
        }
        let kind = if context.net.is_exe { PacketType::LOBBY_EXECHARACTER } else { PacketType::LOBBY_CHARACTER };
        let mut packet = Packet::new(kind);
        let _ = packet.write_u8(self.cursor);
        context.net.send_reliable(&packet);
    }

    /// The next or the previous page of the row of other players, if there is one.
    fn turn_row_page(&mut self, context: &mut Context, step: i32) {
        let pages = icons::row_pages(context.net.players.len());
        let Some(page) = self.row_page.checked_add_signed(step as isize).filter(|&page| page < pages) else { return };
        self.row_page = page;
        context.audio.play(sound::SND_MENU_SELECT, false);
    }

    /// Z on the open cards: ready with them, unless a card's character is taken.
    fn get_ready_with_cards(&mut self, context: &mut Context) {
        if let Some(taken) = self.cards.taken_character(&context.net) {
            self.refuse_card(context, Some(&taken));
            return;
        }
        self.cards.is_open = false;
        self.is_ready = true;
        self.send_ready_state(context);
        context.audio.play(sound::SND_READY, false);
        context.options.preferred_exe = self.cards.exe;
        context.options.preferred_survivor = self.cards.survivor;
        context.options.save();
    }

    /// LOBBY_READY_STATE with the preference cards, 1-based.
    fn send_ready_state(&self, context: &mut Context) {
        let mut packet = Packet::new(PacketType::LOBBY_READY_STATE);
        let _ = packet.write_u8(u8::from(self.is_ready));
        let _ = packet.write_u8(self.cards.exe);
        let _ = packet.write_u8(self.cards.survivor);
        context.net.send_reliable(&packet);
    }

    /// A card's character is someone else's: say so and let the player choose again.
    fn refuse_card(&mut self, context: &mut Context, character_name: Option<&&str>) {
        context.audio.play(sound::SND_NONO, false);
        let name = character_name.copied().unwrap_or("that character");
        self.chat.add_message(context, "\\(lobby)~", &format!("{name} is already taken"));
        self.is_ready = false;
        self.cards.is_open = true;
    }

    fn on_notice(&mut self, context: &mut Context, notice: Notice) -> Option<RoomId> {
        match notice {
            Notice::GoTo(room) => return Some(room),
            Notice::ShowError(code) => return Some(go_to_error(context, code)),
            Notice::PlayerJoined { id, play_sound } => {
                if let Some(player) = context.net.players.get(&id) {
                    let (nickname, is_ready) = (player.nickname.clone(), player.is_ready);
                    self.player_joined(context, id, nickname, is_ready, play_sound);
                }
            }
            Notice::JoinedLobby => {
                if let Some(id) = context.net.id {
                    let nickname = context.options.nickname.clone();
                    self.player_joined(context, id, nickname, false, true);
                }
                self.can_use = true;
            }
            Notice::PlayerLeft(id) => self.player_left(context, id),
            Notice::ChatMessage { sender, text } => self.chat.add_message(context, &sender, &text),
            Notice::Countdown { counting, seconds } => {
                self.count_scale = COUNTER_POP_SCALE;
                self.is_counting = counting;
                self.countdown = seconds;
            }
            Notice::VoteMaps => {
                // Not ready, or ready with the freecam card, when the lobby started the
                // round: this client watches it (states::spectate::seat_lobby_watchers).
                if !self.is_ready || (context.net.preselects && self.cards.chooses_freecam()) {
                    context.net.is_spectating = true;
                }
                self.start_vote(context);
            }
            Notice::VoteTime(seconds) => {
                self.count_scale = 1.0;
                self.vote_timer = seconds;
                if seconds <= WARNING_SECONDS {
                    self.count_scale = COUNTER_POP_SCALE;
                    context.audio.play(sound::SND_CLOCK, false);
                }
            }
            Notice::CharacterTime(seconds) => {
                self.count_scale = 1.0;
                self.character_timer = seconds;
                if seconds <= WARNING_SECONDS {
                    self.count_scale = COUNTER_POP_SCALE;
                    context.audio.play(sound::SND_CLOCK, false);
                }
            }
            Notice::VoteCounts(votes) => self.votes = votes,
            Notice::ExeChosen { map } => self.exe_chosen(map, context),
            Notice::SurvivorTaken(survivor) => self.refuse_card(context, SURVIVOR_NAMES.get(survivor.wrapping_sub(1) as usize)),
            Notice::ExeTaken(exe) => self.refuse_card(context, EXE_NAMES.get(exe.wrapping_sub(1) as usize)),
            Notice::RoundStarts => {
                // Without the stage the intro shows what everyone got.
                if !context.net.character_stage {
                    self.state = LobbyState::RoundIntro;
                    for icon in &mut self.icons {
                        icon.hidden = true;
                    }
                }
                self.round_starting_ticks = Some(0);
                context.audio.stop_music();
            }
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

    /// lobby_player_joined
    fn player_joined(&mut self, context: &mut Context, id: u16, nickname: String, is_ready: bool, play_sound: bool) {
        if self.icons.iter().any(|icon| icon.player == id) {
            return;
        }
        self.icons.push(PlayerIcon::new(id, nickname, is_ready));
        if play_sound {
            context.audio.play(sound::SND_RING, false);
        }
    }

    /// lobby_player_left: the icon bursts into a splash.
    fn player_left(&mut self, context: &mut Context, id: u16) {
        let Some(index) = self.icons.iter().position(|icon| icon.player == id) else { return };
        let icon = self.icons.remove(index);
        self.splashes.push(Splash::new(icon.x, icon.y));
        context.audio.play(sound::SND_HURT, false);
    }

    /// SERVER_LOBBY_EXE: the vote is over and the character select stage begins, its
    /// cursor on this player's preference card. Without the stage the round starts
    /// at once and the round intro replaces it (Notice::RoundStarts).
    fn exe_chosen(&mut self, map: u16, context: &mut Context) {
        self.flash = Some(WhiteFlash::new());
        self.state = LobbyState::CharacterSelect;
        self.play_stage_music(context);
        let net = &context.net;
        let (preferred, count) = if net.is_exe { (self.cards.exe, EXE_COUNT) } else { (self.cards.survivor, SURVIVOR_COUNT) };
        self.cursor = if (1..=count).contains(&preferred) { preferred } else { 1 };
        for icon in &mut self.icons {
            icon.hidden = false;
        }
        self.round_map = Some(map);
        self.preview.blend = C_WHITE;
        self.preview.set_zone(map as usize);
    }

    /// After the round starts the intro holds a moment, fades out, and the level opens.
    fn step_round_start(&mut self) -> Option<RoomId> {
        let ticks = self.round_starting_ticks.as_mut()?;
        *ticks += 1;
        // The stage fades at once (obj_lobby state 3), the intro is held to be seen.
        if self.state == LobbyState::RoundIntro && *ticks <= crate::core::config::ticks(INTRO_HOLD_SECONDS) {
            return None;
        }
        self.fade_out += INTRO_FADE_PER_STEP * step();
        if self.fade_out <= 1.0 {
            return None;
        }
        let level = crate::client::levels::LEVELS.get(self.round_map? as usize)?;
        Some(level.room)
    }

    fn draw(&mut self, context: &mut Context, current_time_ms: f64) {
        for layer_index in 0..self.room.layers.len() {
            if !self.room.layers[layer_index].visible {
                continue;
            }
            let haze = context.canvas.start_heat_haze(self.room.layers[layer_index].heat_haze);
            self.room.draw_layer(&mut context.canvas, &self.room.layers[layer_index]);
            if layer_index == self.preview_layer {
                self.preview.draw(&mut context.canvas, current_time_ms);
            }
            if layer_index == self.lobby_layer {
                self.draw_lobby(context, current_time_ms);
                self.draw_picker(context, current_time_ms);
            }
            context.canvas.finish_heat_haze(haze);
        }
        let view = self.view(&context.net);
        for icon in &mut self.icons {
            icon.draw(&mut context.canvas, &view, &context.unlockables, current_time_ms);
        }
        if self.state == LobbyState::Lobby {
            let pages = icons::row_pages(context.net.players.len());
            icons::draw_page_arrows(&mut context.canvas, self.row_page, pages, current_time_ms);
            if let Some(own_icon) = self.icons.iter().find(|icon| icon.is_own(&context.net)) {
                self.cards.draw(&mut context.canvas, own_icon.x, own_icon.y, &context.net, self.is_ready, current_time_ms);
            }
        }
        for splash in &self.splashes {
            splash.draw(&mut context.canvas);
        }
        if let Some(flash) = &self.flash {
            flash.draw(&mut context.canvas);
        }
        if let Some(fade) = &self.fade_in {
            fade.draw_gui(&mut context.canvas);
        }
        self.minigame.draw_over(&mut context.canvas, &context.options.keys, current_time_ms);
        if self.fade_out > 0.0 {
            context.canvas.draw_sprite_ext(sprite::SPR_BLACK, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, C_WHITE, self.fade_out);
        }
    }

    /// The other players' characters dimmed behind, the map's card, and this
    /// player's character in front with its bars and description strip.
    fn draw_round_intro(&mut self, context: &mut Context) {
        let net = &context.net;
        let canvas = &mut context.canvas;
        let ticks = self.intro_ticks;
        let frame = |canvas: &Canvas, art: usize| art_frame(canvas, art, ticks);
        let (own_x, floor_y) = ART_POSITION;

        let crowd = others_characters(net);
        let crowd_spacing = (INTRO_CROWD_RIGHT - INTRO_CROWD_LEFT) / (crowd.len() + 1) as f64;
        for (place, &art) in crowd.iter().enumerate() {
            let x = INTRO_CROWD_LEFT + crowd_spacing * (place + 1) as f64;
            let scale = INTRO_CROWD_SCALE;
            canvas.draw_sprite_ext(CHARACTER_ART[art], frame(canvas, art), x, floor_y, scale, scale, 0.0, INTRO_CROWD_BLEND, 1.0);
        }
        if let Some(map) = self.round_map {
            let (card_x, card_y) = INTRO_MAP_CARD_POSITION;
            canvas.draw_sprite(sprite::SPR_MAPVOTE, map as f64, card_x, card_y);
        }
        let own_art = art_index(net.character, net.is_exe);
        if let Some(art) = own_art {
            canvas.draw_sprite(CHARACTER_ART[art], frame(canvas, art), own_x, floor_y);
        }
        let bars_frame = if net.is_exe { 1.0 } else { 0.0 };
        let top_bar = [0.0, 0.0, CHAR_BARS_UNTITLED_WIDTH, CHAR_BARS_TOP_HEIGHT];
        canvas.draw_sprite_part_ext(sprite::SPR_CHAR_BARS, bars_frame, top_bar, 0.0, 0.0, VIEW_WIDTH / CHAR_BARS_UNTITLED_WIDTH, 1.0, C_WHITE, 1.0);
        let rest = [0.0, CHAR_BARS_TOP_HEIGHT, VIEW_WIDTH, VIEW_HEIGHT - CHAR_BARS_TOP_HEIGHT];
        canvas.draw_sprite_part_ext(sprite::SPR_CHAR_BARS, bars_frame, rest, 0.0, CHAR_BARS_TOP_HEIGHT, 1.0, 1.0, C_WHITE, 1.0);
        if let Some(art) = own_art {
            let (strip_x, strip_y) = DESCRIPTION_POSITION;
            canvas.draw_sprite(sprite::SPR_CHAR_INFO, art as f64, strip_x, strip_y);
        }
        // Counted in 60 Hz steps, which the character art animates by.
        self.intro_ticks += crate::core::config::step();
    }

    /// obj_lobby Draw state 2: the character under the cursor (or the one picked)
    /// with its bars and description strip, and the clock while choosing. A taken
    /// character's picture is left out; a player watching the round sees only the bars.
    fn draw_character_select(&mut self, context: &mut Context) {
        let net = &context.net;
        let is_picking = self.is_picking(net);
        let canvas = &mut context.canvas;
        let character = if is_picking { self.cursor } else { net.character };
        let art = art_index(character, net.is_exe).filter(|_| !(is_picking && is_taken(net, self.cursor)));
        if let Some(art) = art {
            let (x, y) = ART_POSITION;
            canvas.draw_sprite(CHARACTER_ART[art], art_frame(canvas, art, self.intro_ticks), x, y);
        }
        canvas.draw_sprite(sprite::SPR_CHAR_BARS, if net.is_exe { 1.0 } else { 0.0 }, 0.0, 0.0);
        if let Some(art) = art_index(character, net.is_exe) {
            let (strip_x, strip_y) = DESCRIPTION_POSITION;
            canvas.draw_sprite(sprite::SPR_CHAR_INFO, art as f64, strip_x, strip_y);
        }
        if is_picking {
            let (counter_x, counter_y) = COUNTER_POSITION;
            let scale = self.count_scale;
            let color = if self.character_timer <= WARNING_SECONDS { C_RED } else { C_WHITE };
            draw_counter_centered(canvas, self.character_timer as i32, counter_x, counter_y, scale, color);
        }
        // Counted in 60 Hz steps, which the character art animates by.
        self.intro_ticks += step();
    }

    fn draw_picker(&mut self, context: &mut Context, current_time_ms: f64) {
        if let Some(actions) = self.host_actions.as_mut() {
            if !actions.step(context) {
                self.host_actions = None;
            }
            if let Some(actions) = &self.host_actions {
                actions.draw(&mut context.canvas);
            }
            return;
        }
        let Some(picker) = self.picker.as_mut() else { return };
        let others: Vec<u16> = context.net.players.keys().copied().filter(|id| Some(*id) != context.net.id).collect();
        let outcome = picker.draw(context, &others, current_time_ms);
        match outcome {
            PickerOutcome::Picked(player) => {
                let nickname = context.net.players.get(&player).map_or("", |player| player.nickname.as_str());
                self.host_actions = Some(HostActions::new(player, nickname));
            }
            PickerOutcome::Closed => self.picker = None,
            PickerOutcome::StillPicking if self.state != LobbyState::Lobby => self.picker = None,
            PickerOutcome::StillPicking => {}
        }
    }

    /// Draw_0 of obj_lobby.
    fn draw_lobby(&mut self, context: &mut Context, current_time_ms: f64) {
        if self.count_scale > 1.0 {
            self.count_scale -= COUNTER_SHRINK_PER_STEP * step();
        }
        match self.state {
            LobbyState::Lobby => self.draw_waiting_for_players(context, current_time_ms),
            LobbyState::Vote => self.draw_vote(context),
            // Whoever watches has no character to pick: the round intro, with
            // everyone's icons choosing above it.
            LobbyState::CharacterSelect if context.net.is_spectating => self.draw_round_intro(context),
            LobbyState::CharacterSelect => self.draw_character_select(context),
            LobbyState::RoundIntro => self.draw_round_intro(context),
        }
        // The lobby draws its own chat; past it only whoever watches the round has one.
        if self.state != LobbyState::Lobby && context.net.is_spectating {
            self.chat.draw(&mut context.canvas, CHAT_LINES_TOP, CHAT_BOX_TOP);
        }
    }

    fn draw_waiting_for_players(&mut self, context: &mut Context, current_time_ms: f64) {
        // The server's ?gui window neither gets ready, chats nor leaves: of the lobby it
        // keeps the players and the sign that says to choose one.
        let host_window = context.net.observer_key.is_some();
        let canvas = &mut context.canvas;
        if !host_window {
            draw_lobby_bars_and_hints(canvas, &context.options.keys, context.net.preselects);
        }
        let (counter_x, counter_y) = COUNTER_POSITION;
        if self.is_counting && self.picker.is_none() {
            let scale = self.count_scale;
            draw_counter_centered(canvas, self.countdown as i32, counter_x, counter_y, scale, C_WHITE);
        } else {
            if self.picker.is_some() {
                super::draw_choose_player_sign(canvas);
            } else {
                super::draw_waiting_sign(canvas, (current_time_ms / SIGN_FRAME_MILLISECONDS) as usize % SIGN_DOT_STEPS);
            }
        }
        if host_window {
            return;
        }
        self.chat.draw(canvas, CHAT_LINES_TOP, CHAT_BOX_TOP);
        let (text_x, text_y) = EXE_CHANCE_TEXT_POSITION;
        draw_text(canvas, text_x, text_y, &format!("\\your chance to be exe: {}%", context.net.exe_chance), C_WHITE, 1.0);
    }

    fn draw_vote(&mut self, context: &mut Context) {
        let canvas = &mut context.canvas;
        canvas.draw_sprite(sprite::SPR_VOTE_BARS, 0.0, 0.0, 0.0);
        let (counter_x, counter_y) = COUNTER_POSITION;
        let scale = self.count_scale;
        draw_counter_centered(canvas, self.vote_timer as i32, counter_x, counter_y, scale, C_WHITE);

        for (card, &map) in self.maps.iter().enumerate() {
            let is_cursor = self.chosen_map == card && self.can_vote;
            let blend = if is_cursor { C_WHITE } else { C_DKGRAY };
            canvas.draw_sprite_ext(sprite::SPR_MAPVOTE, map as f64, MAP_CARD_X[card], self.map_card_y(card), 1.0, 1.0, 0.0, blend, self.map_fade);
        }
        if self.map_fade < 1.0 {
            self.map_fade += MAP_CARD_FADE_PER_STEP * step();
        }
        if self.chosen_map_jump > 0.0 {
            self.chosen_map_jump -= MAP_CARD_JUMP_FALL_PER_STEP * step();
        }

        // The preview shows the map under the cursor from the next frame on.
        let map_under_cursor = self.maps[self.chosen_map] as usize;
        self.preview.visible = true;
        self.preview.blend = C_DKGRAY;
        if self.preview.map() != map_under_cursor {
            self.preview.set_zone(map_under_cursor);
        }
        draw_map_description(canvas, map_under_cursor);
        if self.can_vote {
            return;
        }
        // The green count is this player's vote; whoever watches has none.
        let voted_card = (!context.net.is_spectating).then_some(self.chosen_map);
        for (card, &count) in self.votes.iter().enumerate() {
            let blend = if voted_card == Some(card) { VOTED_MAP_BLEND } else { C_WHITE };
            let (offset_x, offset_y) = VOTE_COUNT_OFFSET;
            canvas.draw_sprite_ext(sprite::SPR_COUNTER2, count as f64, MAP_CARD_X[card] + offset_x, self.map_card_y(card) + offset_y, 1.0, 1.0, 0.0, blend, 1.0);
        }
    }

    /// Cards slide down while fading in; the one under the cursor jumps.
    fn map_card_y(&self, card: usize) -> f64 {
        let jump = if self.chosen_map == card { self.chosen_map_jump } else { 0.0 };
        MAP_CARD_TOP + MAP_CARD_SLIDE * self.map_fade - jump
    }
}

/// Whether another player of this player's side holds `character`, while characters
/// are not shared (charselect_mod_unlocked); with one EXE nobody else is EXE.
fn is_taken(net: &net::NetClient, character: u8) -> bool {
    if net.duplicate_characters_allowed {
        return false;
    }
    net.players.iter().any(|(id, player)| Some(*id) != net.id && net.exe_ids.contains(id) == net.is_exe && player.character == character)
}

/// obj_lobby_icon state 2 and the vote: -1 for left, 1 for right, 0 for neither.
fn sideways(context: &Context) -> i32 {
    let keys = &context.options.keys;
    if context.input.pressed(keys.left.0) {
        -1
    } else if context.input.pressed(keys.right.0) {
        1
    } else {
        0
    }
}

/// obj_lobby Draw: the description of `map` in the vote's bottom bar. The Sandbox map
/// page, which looks as the vote does, writes it there too.
pub fn draw_map_description(canvas: &mut Canvas, map: usize) {
    let (text_x, text_y) = MAP_DESCRIPTION_POSITION;
    if let Some(map) = crate::maps::MAP_LIST.get(map) {
        draw_text(canvas, text_x, text_y, map.description, C_WHITE, 1.0);
    }
}

/// The characters the other players got this round, each once, in CHARACTER_ART order.
fn others_characters(net: &net::NetClient) -> Vec<usize> {
    let mut arts: Vec<usize> = net
        .players
        .iter()
        .filter_map(|(id, player)| art_index(player.character, net.exe_ids.contains(id)))
        .collect();
    arts.sort_unstable();
    arts.dedup();
    arts
}

/// spr_lobby_bars and the hints written on them, with the keys this player bound
/// rather than the defaults the picture showed. Where the server lets players choose
/// ahead, the key that opens the preference cards is written under ready.
fn draw_lobby_bars_and_hints(canvas: &mut Canvas, keys: &crate::client::options::KeyBindings, preselects: bool) {
    canvas.draw_sprite(sprite::SPR_LOBBY_BARS, 0.0, 0.0, 0.0);
    let ready = format!("ready <press {}>", crate::client::input::key_name(keys.jump.0));
    let chat = "open chat <press enter>";
    let exit = format!("exit <press {}>", crate::client::input::key_name(keys.special1.0));
    if preselects {
        let choose = format!("choose <press {}>", crate::client::input::key_name(keys.special2.0));
        canvas.draw_menu_text(HINT_LEFT, HINTS_TOP - HINT_LINE_HEIGHT / 2.0, &ready, C_WHITE, 1.0);
        canvas.draw_menu_text(HINT_LEFT, HINTS_TOP + HINT_LINE_HEIGHT / 2.0, &choose, C_WHITE, 1.0);
    } else {
        canvas.draw_menu_text(HINT_LEFT, HINTS_TOP, &ready, C_WHITE, 1.0);
    }
    canvas.draw_menu_text((VIEW_WIDTH / 2.0 - canvas.menu_text_width(chat) / 2.0).floor(), HINTS_TOP, chat, C_WHITE, 1.0);
    canvas.draw_menu_text(HINT_RIGHT_END - canvas.menu_text_width(&exit), HINTS_TOP, &exit, C_WHITE, 1.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intro_shows_each_other_players_character_once() {
        let mut net = net::NetClient::new();
        net.character = 3;
        // Player 3 is the EXE, playing Chaos (EXE 2); player 4 has no character yet.
        net.exe_ids = vec![3];
        for (id, character) in [(1, 2), (2, 2), (3, 2), (4, crate::packet::NO_CHARACTER)] {
            let mut player = net::Player::new(format!("fox{id}"), 0, false);
            player.character = character;
            net.players.insert(id, player);
        }
        assert_eq!(others_characters(&net), vec![1, 7], "knuckles once, chaos; not this player's eggman");
    }
}
