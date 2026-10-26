//! Draw GUI of a level, back to front: the health, name and rings above other
//! players (obj_player_puppet) and above this player (the character objects), then
//! obj_level (screen tint, hiding and hit flashes, clock, everyone's health icons,
//! title card, ping, the player list).

use crate::client::canvas::{make_color_rgb, Canvas, TextFont, C_DKGRAY, C_GREEN, C_RED, VIEW_HEIGHT, VIEW_WIDTH};
use crate::client::screens::round_clock::RoundClock;
use crate::client::text::{draw_text, text_width};
use crate::client::options::KeyBindings;
use crate::client::touch::{ButtonSign, Control};
use crate::core::resources::names::sprite;
use crate::core::resources::SpriteId;
use crate::core::config::GameplayConfig;
use crate::core::player::{Character, ExeCharacter, Player};
use crate::core::snapshot::HealthView;
use crate::core::rooms::ids::RoomId;
use crate::core::world::{C_GREY, C_WHITE};
use crate::local_link::HOST_PLAYER;
use crate::core::config::step;

/// spr_badconnection: its frames, shown one every 200 ms.
const BAD_CONNECTION_FRAMES: f64 = 6.0;
/// Rooms without the dark screen overlay, in the order of their map numbers.
const ROOMS_WITHOUT_OVERLAY: [RoomId; 10] = [
    RoomId::Ravinemist,
    RoomId::Youcantrun,
    RoomId::Notperfect,
    RoomId::Act9,
    RoomId::Nastyparadise,
    RoomId::Volcanovalley,
    RoomId::Greenhill,
    RoomId::Angelisland,
    RoomId::Fartzone,
    RoomId::Weedzone,
];
const OVERLAY_ALPHA: f64 = 0.7;
const FLASH_FADE_PER_STEP: f64 = 0.02;
/// The HUD waits for the title card to get this far.
const TITLE_CARD_HUD_FROM: f64 = 4.0;
const TITLE_CARD_STEP: f64 = 0.12;
const TITLE_CARD_SLIDE: f64 = 12.0;
/// The title card slides in until 8, the black screen fades out from 8 to 16,
/// the card slides out after 18 and is gone at 30.
const TITLE_CARD_IN_UNTIL: f64 = 8.0;
const TITLE_CARD_BLACK_FROM: f64 = 8.0;
const TITLE_CARD_BLACK_UNTIL: f64 = 16.0;
const TITLE_CARD_BLACK_SHOWN_UNTIL: f64 = 18.0;
const TITLE_CARD_OUT_FROM: f64 = 18.0;
const TITLE_CARD_GONE_AT: f64 = 30.0;
/// spr_playerhealth: 7 frames per survivor, health in 5 steps, then downed and dead.
use crate::client::screens::character_art::HEALTH_FRAMES_PER_CHARACTER;
const HEALTH_STEPS: f64 = 5.0;
const DOWNED_FRAME: f64 = 5.0;
const DEAD_FRAME: f64 = 6.0;
/// The death countdown shows once it is below its start value.
const DEATH_COUNTDOWN_START: i32 = 31;
/// Above-head bars: this far above someone else's player; above this player it depends on the character.
const OTHER_BAR_ABOVE: f64 = 45.0;
const OTHER_NAME_ABOVE: f64 = 43.0;
const RINGS_ABOVE_BAR: f64 = 12.0;
/// obj_exe Draw_64 draws the frozen mark 20 px above EXE.
const EXE_FROZEN_ABOVE: f64 = 20.0;
/// obj_majin_forest_controller's copy of a name plate at the other end of Majin Forest.
const WRAPPED_BAR_ABOVE: f64 = 50.0;
const WRAPPED_NAME_ABOVE: f64 = 40.0;
const PING_WARNING_MS: i32 = 80;
const PING_BAD_MS: i32 = 160;
/// The sign over a player whose pause menu is open, above their name plate.
const AFK_SIGN: &str = "afk";
const AFK_ABOVE: f64 = 54.0;
const PLAYER_LIST_TOP: f64 = 44.0;
const PLAYER_LIST_ROW: f64 = 30.0;
/// spr_tab was a whole screen of black at 0.4 with the heading drawn in: now the
/// dimming is a plain fill and the heading is text, where the picture had it.
const PLAYER_LIST_DIM: f64 = 0.4;
const PLAYER_LIST_TITLE: &str = "player list";
const PLAYER_LIST_TITLE_CENTER: f64 = 245.0;
const PLAYER_LIST_TITLE_TOP: f64 = 18.0;
/// The column names, grey in the chat lettering, from the left of a column of rows.
const PLAYER_LIST_LABELS_TOP: f64 = 37.0;
const PLAYER_LIST_NICKNAME_X: f64 = 37.0;
const PLAYER_LIST_PING_X: f64 = 186.0;
/// What a spectator's own row of the player list says where the ping would be.
const SPECTATOR_LABEL: &str = "spectator";
/// The emotion faces at the bottom right, top to bottom, each the one on its touch
/// button. Placed by their middle (origin 5, 5), where spr_gui_emotions had the
/// top-left corner of the first at (442, 222).
const EMOTION_BUTTONS: [Control; 3] = [Control::Emotion1, Control::Emotion2, Control::Emotion3];
const EMOTION_FACES_X: f64 = 447.0;
const EMOTION_FACES_Y: f64 = 227.0;
const EMOTION_FACE_STEP: f64 = 14.0;
/// The faces are opaque for the touch buttons; spr_gui_emotions had them at alpha 148.
const EMOTION_FACE_ALPHA: f64 = 148.0 / 255.0;
/// What the host's row says where the ping would be: a game that hosts (IServer) plays
/// its own server through memory, so its round trip is always 0.
const HOST_LABEL: &str = "host";
/// Where the name and the ping are written in a row, from its left.
const PLAYER_LIST_NAME_IN_ROW: f64 = 31.0;
const PLAYER_LIST_PING_IN_ROW: f64 = 183.0;
const PLAYER_LIST_TEXT_IN_ROW: f64 = 11.0;
/// The death countdown over a health icon, from its bottom left corner.
const COUNTDOWN_IN_BADGE: (f64, f64) = (12.0, 4.0);
/// A survivor's rings (the count inside a ring) and, in Ravine Mist, shards (the count,
/// green) at the right of the name's box. The 6 px digits stand in the middle of the
/// 16 px ring.
const PLAYER_LIST_RING_IN_ROW: (f64, f64) = (152.0, 6.0);
const PLAYER_LIST_RINGS_IN_ROW: (f64, f64) = (PLAYER_LIST_RING_IN_ROW.0 + 5.0, PLAYER_LIST_RING_IN_ROW.1 + 5.0);
/// Ravine Mist's shards: the count inside a shard without its glow, left of the ring;
/// the shard's body is around its origin.
const PLAYER_LIST_SHARD_IN_ROW: (f64, f64) = (140.0, 14.0);
const PLAYER_LIST_SHARDS_IN_ROW: (f64, f64) = (PLAYER_LIST_SHARD_IN_ROW.0 - 3.0, PLAYER_LIST_SHARD_IN_ROW.1 - 3.0);
/// The circle of a row holds a 26 px health icon 1 px in, or a 24 px EXE badge 2 px in.
const PLAYER_LIST_HEALTH_BADGE_IN_ROW: f64 = 1.0;
const PLAYER_LIST_EXE_BADGE_IN_ROW: f64 = 2.0;
/// spr_lobby_icon: drawn from its middle; its last four frames are the EXE characters
/// in order (this port's, added after the original eight), which the list shows.
const LOBBY_ICON_ORIGIN: f64 = 12.0;
const LOBBY_ICON_FIRST_EXE: f64 = 8.0;
/// spr_playerhealth and its kin: 26 px, drawn from their bottom left corner.
const HEALTH_BADGE_SIZE: f64 = 26.0;
/// spr_tab_entry: its width and origin (its middle).
const PLAYER_LIST_ENTRY_WIDTH: f64 = 226.0;
const PLAYER_LIST_ENTRY_ORIGIN: f64 = 113.0;
/// Two columns this far apart when the rows do not fit under each other.
const PLAYER_LIST_COLUMN_GAP: f64 = 8.0;
/// The row of whoever the camera watches stands where the bottom row of health icons
/// was, a little higher, clear of the screen's edge.
const WATCHED_ROW_TOP: f64 = VIEW_HEIGHT - 40.0;
/// Half of the 6 px a digit of the chat lettering takes.
const NUMBER_HALF_DIGIT: f64 = 3.0;

fn ping_good_color() -> u32 {
    make_color_rgb(0x0F, 0xFF, 0x39)
}

/// What the HUD shows of a player of the round.
pub struct HudPlayer<'a> {
    pub id: u16,
    pub nickname: &'a str,
    /// None once the player has escaped, and while this client may not see them
    /// (states.gameplay.limit_player_view): then only `health` is known about them.
    pub player: Option<&'a Player>,
    /// What the player list shows about them wherever they are.
    /// None once they have escaped, and for a spectator, who plays nobody.
    pub health: Option<HealthView>,
    /// The character they play, known even after escaping (1..6 survivors).
    pub character: Character,
    pub is_exe: bool,
    /// SERVER_GAME_DEATHTIMER_TICK: seconds left and whether EXE camps the body.
    pub death_timer: Option<(u8, bool)>,
    pub ping_ms: i32,
}

pub struct Hud {
    room: RoomId,
    /// A level played alone (Sandbox): no clock, no health icons, no ping and no
    /// player list, because there is neither a round nor anyone else.
    solo: bool,
    /// Watching a round nobody here plays (states::spectate): the row of health icons
    /// and the player list leave out the row this client would have had.
    spectating: bool,
    /// The server's ?gui window's hitbox view: no picture laid over the screen.
    pub without_overlay: bool,
    /// global.showHud
    pub shown: bool,
    pub clock: RoundClock,
    /// bloodFade: set by a hit, fades out.
    pub blood_fade: f64,
    /// This player fell and the camera still lies on them (level/mod.rs): the screen
    /// darkens as it does while hiding.
    pub lying_down: bool,
    hide_fade: f64,
    /// obj_level state 1: the big ring is out, survivors can escape.
    pub escape: bool,
    /// obj_netclient.udp_timeout < 60 * 3: the server has been silent for a while.
    pub bad_connection: bool,
    title_card: f64,
    card_x: f64,
    card_y: f64,
    level_number: f64,
    /// HurtRules.max_hp: how many hits a survivor has, which the bars and the icons
    /// of the bottom row draw one step per.
    max_hp: i32,
}

impl Hud {
    pub fn new(room: RoomId, level_number: usize, solo: bool, spectating: bool, max_hp: i32) -> Hud {
        Hud {
            room,
            solo,
            spectating,
            without_overlay: false,
            shown: true,
            clock: RoundClock::default(),
            blood_fade: 0.0,
            hide_fade: 0.0,
            lying_down: false,
            escape: false,
            bad_connection: false,
            title_card: 0.0,
            card_x: crate::client::canvas::VIEW_WIDTH,
            card_y: -crate::client::canvas::VIEW_HEIGHT,
            level_number: level_number as f64,
            max_hp,
        }
    }

    /// obj_level Draw_64. `own` is this player, `others` everyone else of the round.
    /// `watched`: whoever the camera watches, if it watches someone (camera.rs watched):
    /// their player list row is shown at the bottom instead of their plate above them.
    #[allow(clippy::too_many_arguments)]
    pub fn draw(&mut self, canvas: &mut Canvas, own: &HudPlayer, others: &[HudPlayer], watched: Option<&HudPlayer>, options: &crate::client::options::Options, current_time_ms: f64, player_list_held: bool) {
        if !ROOMS_WITHOUT_OVERLAY.contains(&self.room) && !self.without_overlay {
            canvas.draw_sprite_ext(sprite::SPR_SCREENOVERLAY, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, C_WHITE, OVERLAY_ALPHA);
        }
        canvas.draw_sprite_ext(sprite::SPR_HIDEGUI, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, C_WHITE, self.hide_fade);
        canvas.draw_sprite_ext(sprite::SPR_ATTACKGUI, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, C_WHITE, self.blood_fade);
        if self.blood_fade > 0.0 {
            self.blood_fade -= FLASH_FADE_PER_STEP * step();
        }
        let hiding = own.player.is_some_and(|player| player.is_hiding) || self.lying_down;
        if hiding && self.hide_fade < 1.0 {
            self.hide_fade += FLASH_FADE_PER_STEP * step();
        } else if !hiding && self.hide_fade > 0.0 {
            self.hide_fade -= FLASH_FADE_PER_STEP * step();
        }

        if self.shown && !self.solo && self.title_card > TITLE_CARD_HUD_FROM {
            if self.bad_connection {
                canvas.draw_sprite(sprite::SPR_BADCONNECTION, ((current_time_ms / 200.0).floor() % BAD_CONNECTION_FRAMES).floor(), 0.0, 0.0);
            }
            self.clock.draw(canvas);
            if self.escape && (current_time_ms / 30.0).floor() % 20.0 < 10.0 {
                let demon = own.is_exe || own.player.is_some_and(|player| player.revival_times >= 2);
                canvas.draw_sprite(sprite::SPR_STATUS, if demon { 1.0 } else { 0.0 }, 240.0, 32.0);
            }
        }
        self.draw_title_card(canvas);
        // A spectator has no round trip time of its own to show: it sends nothing to
        // be answered, and the round is not waiting for it either. Nor has the host.
        if self.shown && !self.solo && !self.spectating && own.id != HOST_PLAYER {
            if own.ping_ms <= 0 {
                draw_text(canvas, 4.0, 4.0, &format!("waiting for players{}", loading_dots(current_time_ms)), C_WHITE, 1.0);
            } else if options.show_ping {
                draw_text(canvas, 4.0, 4.0, &format!("{}ms", own.ping_ms), ping_color(own.ping_ms), 1.0);
            }
        }
        if self.shown && options.show_fps {
            draw_text(canvas, 4.0, 10.0, &format!("{}fps", macroquad::time::get_fps()), C_WHITE, 1.0);
        }
        if player_list_held && !self.solo {
            self.draw_player_list(canvas, own, others, current_time_ms);
        } else if let Some(entry) = watched.filter(|_| self.shown) {
            let left = ((VIEW_WIDTH - PLAYER_LIST_ENTRY_WIDTH) / 2.0).floor();
            self.draw_player_list_row(canvas, entry, false, self.sees_as_killer(own), left, WATCHED_ROW_TOP, current_time_ms);
        }
    }

    /// The killers' side is told nothing of how the survivors are doing (the server
    /// leaves it out of what it sends them), and its own rings are not shown either.
    fn sees_as_killer(&self, own: &HudPlayer) -> bool {
        !self.spectating && (own.is_exe || own.health.is_some_and(|health| health.is_killers_side()))
    }

    fn draw_title_card(&mut self, canvas: &mut Canvas) {
        if self.title_card < TITLE_CARD_IN_UNTIL {
            if self.card_x > 0.0 {
                self.card_x -= TITLE_CARD_SLIDE * step();
            }
            if self.card_y < 0.0 {
                self.card_y += TITLE_CARD_SLIDE * step();
            }
        }
        if self.title_card < TITLE_CARD_BLACK_SHOWN_UNTIL {
            let faded = (self.title_card.clamp(TITLE_CARD_BLACK_FROM, TITLE_CARD_BLACK_UNTIL) - TITLE_CARD_BLACK_FROM) / (TITLE_CARD_BLACK_UNTIL - TITLE_CARD_BLACK_FROM);
            canvas.draw_sprite_ext(sprite::SPR_BLACK, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, C_WHITE, 1.0 - faded);
        }
        if self.title_card > TITLE_CARD_OUT_FROM {
            if self.card_x < crate::client::canvas::VIEW_WIDTH {
                self.card_x += TITLE_CARD_SLIDE * step();
            }
            if self.card_y > -crate::client::canvas::VIEW_HEIGHT {
                self.card_y -= TITLE_CARD_SLIDE * step();
            }
        }
        if self.title_card < TITLE_CARD_GONE_AT {
            self.card_x = self.card_x.clamp(0.0, crate::client::canvas::VIEW_WIDTH);
            self.card_y = self.card_y.clamp(-crate::client::canvas::VIEW_HEIGHT, 0.0);
            canvas.draw_sprite(sprite::SPR_TITLECARD2, self.level_number, 0.0, self.card_y);
            canvas.draw_sprite(sprite::SPR_TITLECARD1, self.level_number, self.card_x, 0.0);
            self.title_card += TITLE_CARD_STEP * step();
        }
    }

    /// The player list while its key is held (drawPing of obj_level). Rows that do not
    /// fit under each other are split into two columns side by side.
    fn draw_player_list(&self, canvas: &mut Canvas, own: &HudPlayer, others: &[HudPlayer], current_time_ms: f64) {
        canvas.fill_view(crate::core::world::C_BLACK, PLAYER_LIST_DIM);
        let title_left = (PLAYER_LIST_TITLE_CENTER - canvas.font_text_size(TextFont::Header, PLAYER_LIST_TITLE).0 / 2.0).floor();
        canvas.draw_font_text(TextFont::Header, title_left, PLAYER_LIST_TITLE_TOP, PLAYER_LIST_TITLE, C_WHITE, 1.0, 1.0);

        let killer = self.sees_as_killer(own);
        let entries: Vec<&HudPlayer> = std::iter::once(own).chain(others).collect();
        let fitting = ((VIEW_HEIGHT - PLAYER_LIST_TOP) / PLAYER_LIST_ROW).floor().max(1.0) as usize;
        let columns = if entries.len() > fitting { 2 } else { 1 };
        let per_column = entries.len().div_ceil(columns);
        let span = columns as f64 * PLAYER_LIST_ENTRY_WIDTH + (columns - 1) as f64 * PLAYER_LIST_COLUMN_GAP;
        let first_left = ((VIEW_WIDTH - span) / 2.0).floor();
        for column in 0..columns {
            let left = first_left + column as f64 * (PLAYER_LIST_ENTRY_WIDTH + PLAYER_LIST_COLUMN_GAP);
            draw_text(canvas, left + PLAYER_LIST_NICKNAME_X, PLAYER_LIST_LABELS_TOP, "nickname", C_GREY, 1.0);
            draw_text(canvas, left + PLAYER_LIST_PING_X, PLAYER_LIST_LABELS_TOP, "ping", C_GREY, 1.0);
            let rows = entries.iter().skip(column * per_column).take(per_column);
            for (row, entry) in rows.enumerate() {
                let is_own = column == 0 && row == 0;
                self.draw_player_list_row(canvas, entry, is_own, killer, left, PLAYER_LIST_TOP + row as f64 * PLAYER_LIST_ROW, current_time_ms);
            }
        }
    }

    /// One player of the list: the row, with their badge in its circle, name and ping.
    /// A survivor shows the health icon of the bottom row, so the list tells how
    /// everyone is doing; EXE shows which EXE it is.
    #[allow(clippy::too_many_arguments)]
    fn draw_player_list_row(&self, canvas: &mut Canvas, entry: &HudPlayer, is_own: bool, killer: bool, left: f64, y: f64, current_time_ms: f64) {
        canvas.draw_sprite(sprite::SPR_TAB_ENTRY, 0.0, left + PLAYER_LIST_ENTRY_ORIGIN, y);
        draw_text(canvas, left + PLAYER_LIST_NAME_IN_ROW, y + PLAYER_LIST_TEXT_IN_ROW, entry.nickname, C_WHITE, 1.0);
        // A spectator's own row: no character, so no badge, and no ping of its own
        // (see the top left corner); it says what they are instead of looking dead.
        if is_own && self.spectating {
            draw_text(canvas, left + PLAYER_LIST_PING_IN_ROW, y + PLAYER_LIST_TEXT_IN_ROW, SPECTATOR_LABEL, C_GREY, 1.0);
            return;
        }
        if entry.is_exe {
            let exe = entry.health.map_or(0.0, |health| health.exe_character as usize as f64);
            let (x, top) = (left + PLAYER_LIST_EXE_BADGE_IN_ROW + LOBBY_ICON_ORIGIN, y + PLAYER_LIST_EXE_BADGE_IN_ROW + LOBBY_ICON_ORIGIN);
            canvas.draw_sprite(sprite::SPR_LOBBY_ICON, LOBBY_ICON_FIRST_EXE + exe, x, top);
        } else {
            let (x, bottom) = (left + PLAYER_LIST_HEALTH_BADGE_IN_ROW, y + PLAYER_LIST_HEALTH_BADGE_IN_ROW + HEALTH_BADGE_SIZE);
            draw_health_badge(canvas, entry, x, bottom, self.max_hp);
            draw_death_countdown(canvas, entry, is_own, x, bottom);
        }
        if let (false, Some(health)) = (killer || entry.is_exe, entry.health) {
            self.draw_player_list_items(canvas, health, left, y);
        }
        let (ping_text, color) = if entry.id == HOST_PLAYER {
            (HOST_LABEL.to_string(), C_WHITE)
        } else if entry.ping_ms <= 0 {
            (loading_dots(current_time_ms), C_RED)
        } else {
            (format!("{}ms", entry.ping_ms), ping_color(entry.ping_ms))
        };
        draw_text(canvas, left + PLAYER_LIST_PING_IN_ROW, y + PLAYER_LIST_TEXT_IN_ROW, &ping_text, color, 1.0);
    }

    /// A survivor's rings and, in Ravine Mist, shards, in their row.
    fn draw_player_list_items(&self, canvas: &mut Canvas, health: HealthView, left: f64, y: f64) {
        if health.is_killers_side() {
            return;
        }
        canvas.draw_sprite(sprite::SPR_RING, 0.0, left + PLAYER_LIST_RING_IN_ROW.0, y + PLAYER_LIST_RING_IN_ROW.1);
        draw_number(canvas, left + PLAYER_LIST_RINGS_IN_ROW.0, y + PLAYER_LIST_RINGS_IN_ROW.1, health.rings, C_WHITE);
        if self.room == RoomId::Ravinemist {
            let (shard_x, shard_y) = (left + PLAYER_LIST_SHARD_IN_ROW.0, y + PLAYER_LIST_SHARD_IN_ROW.1);
            canvas.draw_sprite(sprite::SPR_RAVINEMIST_SHARD_WITHOUT_GLOW, 0.0, shard_x, shard_y);
            draw_number(canvas, left + PLAYER_LIST_SHARDS_IN_ROW.0, y + PLAYER_LIST_SHARDS_IN_ROW.1, health.shards, C_WHITE);
        }
    }

    /// Draw_64 of the character objects: health and rings above this survivor.
    pub fn draw_above_own(&self, canvas: &mut Canvas, player: &Player, view: (f64, f64)) {
        if player.character == Character::Exe {
            // obj_exe Draw_64: no health above EXE, only the mark of being slowed.
            if player.is_slow {
                let (x, y) = ((player.x - view.0).ceil(), (player.y - view.1).ceil() - EXE_FROZEN_ABOVE);
                canvas.draw_sprite(sprite::SPR_FROZEN, 0.0, x, y);
            }
            return;
        }
        if player.hp <= 0 {
            return;
        }
        let color = if self.room == RoomId::Act9 { C_DKGRAY } else { C_WHITE };
        let above = match player.character {
            Character::Knux | Character::Amy | Character::Sally => 25.0,
            Character::Eggman => 36.0,
            _ => 20.0,
        };
        let (x, y) = ((player.x - view.0).ceil(), (player.y - view.1).ceil() - above);
        // A demon has no health left to show, only the empty grey ring of frame 0.
        if player.revival_times < 2 {
            let bar = if player.red_ring_timer > 0 { sprite::SPR_HP_RR } else { sprite::SPR_HP };
            canvas.draw_sprite_ext(bar, health_bar_frame(player.hp, self.max_hp), x, y, 1.0, 1.0, 0.0, color, 1.0);
        }
        if player.is_slow {
            canvas.draw_sprite(sprite::SPR_FROZEN, 0.0, x, y);
        }
        if player.revival_times < 2 {
            draw_number(canvas, x - 2.0, y - RINGS_ABOVE_BAR, player.rings, color);
        }
    }

    /// Draw_64 of obj_player_puppet: health, name and rings above someone else.
    pub fn draw_above_other(&self, canvas: &mut Canvas, other: &HudPlayer, view: (f64, f64)) {
        let Some(player) = other.player else { return };
        // CLIENT_ROUND_PAUSE: whoever is in their pause menu stands there as away.
        if player.paused {
            let (x, y) = ((player.x - view.0).ceil(), (player.y - view.1).ceil() - AFK_ABOVE);
            draw_text(canvas, x - text_width(canvas, AFK_SIGN) / 2.0, y, AFK_SIGN, C_WHITE, 1.0);
        }
        let plate = Plate { x: player.x, bar_above: OTHER_BAR_ABOVE, name_above: OTHER_NAME_ABOVE, name_width: text_width(canvas, other.nickname), name_shown: self.shown };
        self.draw_plate(canvas, other, player, &plate, view);
    }

    /// obj_majin_forest_controller Draw GUI: on Majin Forest someone near one end also shows
    /// at the other, where the room repeats itself, with slightly different spacing.
    pub fn draw_wrapped_above_other(&self, canvas: &mut Canvas, other: &HudPlayer, view: (f64, f64), room_width: f64) {
        let Some(player) = other.player else { return };
        let x = if (0.0..=VIEW_WIDTH).contains(&player.x) {
            room_width - VIEW_WIDTH + player.x
        } else if (room_width - VIEW_WIDTH..=room_width).contains(&player.x) {
            player.x - (room_width - VIEW_WIDTH)
        } else {
            return;
        };
        let plate = Plate { x, bar_above: WRAPPED_BAR_ABOVE, name_above: WRAPPED_NAME_ABOVE, name_width: other.nickname.len() as f64 * 8.0, name_shown: true };
        self.draw_plate(canvas, other, player, &plate, view);
    }

    fn draw_plate(&self, canvas: &mut Canvas, other: &HudPlayer, player: &Player, plate: &Plate, view: (f64, f64)) {
        if player.is_hiding {
            return;
        }
        let act9 = self.room == RoomId::Act9;
        let color = if act9 { C_DKGRAY } else { C_WHITE };
        let (x, y) = ((plate.x - view.0).floor(), (player.y - view.1).floor());
        let demon = other.is_exe || player.revival_times >= 2;
        if player.hp > 0 && !demon {
            let bar = if player.red_ring_timer > 0 { sprite::SPR_HP_RR } else { sprite::SPR_HP };
            canvas.draw_sprite_ext(bar, health_bar_frame(player.hp, self.max_hp), x, y - plate.bar_above, 1.0, 1.0, 0.0, color, 1.0);
        }
        if plate.name_shown {
            let name_color = if act9 {
                C_DKGRAY
            } else if demon {
                C_RED
            } else if player.red_ring_timer > 0 {
                make_color_rgb(0x99, 0x43, 0xAD)
            } else {
                C_GREEN
            };
            draw_text(canvas, x - plate.name_width / 2.0, y - plate.name_above, other.nickname, name_color, 1.0);
        }
        if !demon && player.hp > 0 {
            draw_number(canvas, x - 2.0, y - plate.bar_above - RINGS_ABOVE_BAR, player.rings, color);
        }
    }
}

/// Where a name plate stands above someone.
struct Plate {
    x: f64,
    bar_above: f64,
    name_above: f64,
    name_width: f64,
    name_shown: bool,
}

/// Over a downed survivor's health icon (bottom left corner at x, bottom): the
/// seconds left to revive them, red while EXE stands by the body.
fn draw_death_countdown(canvas: &mut Canvas, entry: &HudPlayer, is_own: bool, x: f64, bottom: f64) {
    let Some(health) = entry.health.filter(|health| health.revival_times == 0) else { return };
    let downed = if is_own { health.is_dead } else { health.hp <= 0 };
    let countdown = entry.death_timer.filter(|&(seconds, _)| (seconds as i32) < DEATH_COUNTDOWN_START);
    if let (true, Some((seconds, exe_near))) = (downed, countdown) {
        draw_number(canvas, x + COUNTDOWN_IN_BADGE.0, bottom - COUNTDOWN_IN_BADGE.1, seconds as i32, if exe_near { C_RED } else { C_WHITE });
    }
}

/// A survivor's health icon with its bottom left corner at (x, bottom): escaped,
/// demonized, or how many hits they have left, red ring and first death shown.
fn draw_health_badge(canvas: &mut Canvas, entry: &HudPlayer, x: f64, bottom: f64, max_hp: i32) {
    let character = entry.character as usize as f64 - 1.0;
    let Some(health) = entry.health else {
        canvas.draw_sprite(sprite::SPR_PLAYERESCAPED, character, x, bottom);
        return;
    };
    if health.revival_times >= 2 {
        canvas.draw_sprite(sprite::SPR_PLAYERHEALTH_DEMON, character, x, bottom);
        return;
    }
    let base = character * HEALTH_FRAMES_PER_CHARACTER;
    let frame = if health.hp <= 0 {
        base + if health.revival_times == 0 { DOWNED_FRAME } else { DEAD_FRAME }
    } else {
        base + hits_lost(health.hp, max_hp)
    };
    let icon = if health.red_ring { sprite::SPR_PLAYERHEALTH_REDRING } else { sprite::SPR_PLAYERHEALTH };
    canvas.draw_sprite(icon, frame, x, bottom);
    if health.revival_times >= 1 && health.hp > 0 {
        canvas.draw_sprite(sprite::SPR_PLAYERHEALTH_HIT, character, x, bottom);
    }
}

/// spr_hp frame of a health value: 5 full, 1 nearly empty.
/// spr_hp: one frame per hit left, the last of them for the last hit.
/// ponytail: the picture has five frames, so a server that gives survivors more hits
/// than that shows a full bar until the last five; draw a number beside it if that happens.
fn health_bar_frame(hp: i32, max_hp: i32) -> f64 {
    (HEALTH_STEPS - hits_lost(hp, max_hp)).max(1.0)
}

/// How many steps of the five the health has fallen by: none while untouched, all
/// five once it is gone. A server with a different `max_hp` spreads the same five over it.
fn hits_lost(hp: i32, max_hp: i32) -> f64 {
    let left = (hp as f64 / max_hp.max(1) as f64).clamp(0.0, 1.0);
    ((1.0 - left) * HEALTH_STEPS).round_ties_even()
}

/// string_copy("...", 0, (current_time / 500) % 4)
fn loading_dots(current_time_ms: f64) -> String {
    ".".repeat(((current_time_ms / 500.0) % 4.0).floor() as usize)
}

fn ping_color(ping: i32) -> u32 {
    if ping >= PING_BAD_MS {
        C_RED
    } else if ping >= PING_WARNING_MS {
        make_color_rgb(0xFF, 0xC4, 0x00)
    } else {
        ping_good_color()
    }
}

/// scr_number_spr: the number in the chat lettering, whose digits spr_number repeated,
/// moved half a digit to the left per extra digit.
fn draw_number(canvas: &mut Canvas, x: f64, y: f64, number: i32, color: u32) {
    let digits = number.to_string();
    let shift = (digits.len() as f64 - 1.0) * NUMBER_HALF_DIGIT;
    draw_text(canvas, x - shift, y, &digits, color, 1.0);
}

/// obj_playerui Draw GUI: how ready this player's abilities are, a gauge per
/// ability that slides in and turns white once ready. `up_or_down_held`: Exeller
/// shows the teleport gauge while either is held. With touch controls each gauge sits
/// on the button of its ability instead of in the corner, where a thumb would hide it.
#[allow(clippy::too_many_arguments)]
pub fn draw_ability_gauges(canvas: &mut Canvas, player: &Player, cfg: &GameplayConfig, keys: &KeyBindings, controls_enabled: bool, up_or_down_held: bool, shown: bool, touch: Option<&mut Vec<ButtonSign>>) {
    if !shown || player.is_dead {
        return;
    }
    let ticks = |seconds: f64| crate::core::config::ticks(seconds) as f64;
    let demon = player.revival_times >= 2;
    let frame_if = |on: bool| if on { 1.0 } else { 0.0 };
    // A gauge filled to `progress`, sliding in from the left, white once `ready`.
    let gauge = |button: Control, sprite: SpriteId, frame: f64, slide: f64, y: f64, progress: f64, ready: bool| {
        let color = if ready { C_WHITE } else { C_RED };
        (ButtonSign { button, sprite, frame, color, alpha: progress }, progress * slide, y)
    };
    let recharged = |timer: i32, recharge: f64| 1.0 - (timer.max(0) as f64) / recharge;
    // The free jump sign stands still: white and solid while a jump can be steered.
    let free_jump = controls_enabled && player.is_jumping;
    let free_jump_sign = |can_show: bool| {
        let (color, alpha) = match (can_show, free_jump) {
            (false, _) => (C_RED, 0.0),
            (true, true) => (C_WHITE, 1.0),
            (true, false) => (C_RED, 0.5),
        };
        (ButtonSign { button: Control::A, sprite: sprite::SPR_GUI_EXEFREEJUMP, frame: 0.0, color, alpha }, 3.0, 218.0)
    };

    // Each sign, where it stands without touch controls.
    let mut signs = Vec::new();
    match player.character {
        Character::Exe => match player.exe_character {
            ExeCharacter::Original => {
                let attack = 1.0 - (player.attack_timer.min(180) as f64) / 180.0;
                signs.push(gauge(Control::B, sprite::SPR_GUI_EXEATTACK, 0.0, 10.0, 240.0, attack, attack == 1.0));
                signs.push(free_jump_sign(true));
                let invisibility = (player.invis_timer.min(0) as f64) / -ticks(cfg.exe.invisibility_recharge_seconds);
                signs.push(gauge(Control::C, sprite::SPR_GUI_EXEINVISABILITY, 0.0, 4.0, 250.0, invisibility, invisibility == 1.0));
            }
            ExeCharacter::Chaos => {
                if player.stuck_timer <= 30 / 2 {
                    let attack = if player.slime_timer > 0 { 0.0 } else { 1.0 - (player.attack_timer.min(120) as f64) / 120.0 };
                    signs.push(gauge(Control::B, sprite::SPR_GUI_CHAOSATTACK, frame_if(!player.is_grounded), 10.0, 240.0, attack, attack == 1.0));
                } else {
                    let frame = match (player.is_grounded, player.stuck_dir) {
                        (true, dir) if dir > 0.0 => 1.0,
                        (true, dir) if dir < 0.0 => 0.0,
                        _ => 2.0,
                    };
                    signs.push(gauge(Control::B, sprite::SPR_GUI_CHAOSWALLDASH, frame, 10.0, 240.0, 1.0, true));
                }
                signs.push(free_jump_sign(player.slime_timer <= 0));
                let liquid = if player.slime_timer > 0 { 1.0 } else { (player.slime_timer.min(0) as f64) / -ticks(cfg.chaos.liquid_recharge_seconds) };
                signs.push(gauge(Control::C, sprite::SPR_GUI_CHAOSSLIME, 0.0, 4.0, 250.0, liquid, liquid == 1.0));
            }
            ExeCharacter::Exetior => {
                let attack = 1.0 - (player.attack_timer.min(180) as f64) / 180.0;
                signs.push(gauge(Control::B, sprite::SPR_GUI_EXETIORATTACK, frame_if(!player.is_grounded), 10.0, 240.0, attack, attack == 1.0));
                signs.push(free_jump_sign(true));
                let black_ring = 1.0 - player.bring_timer as f64 / ticks(cfg.exetior.black_ring_recharge_seconds);
                signs.push(gauge(Control::C, sprite::SPR_GUI_EXETIORRING, 0.0, 4.0, 250.0, black_ring, black_ring == 1.0));
            }
            ExeCharacter::Exeller => {
                let attack = 1.0 - (player.attack_timer.min(180) as f64) / 180.0;
                signs.push(gauge(Control::B, sprite::SPR_GUI_EXEATTACK, 0.0, 10.0, 240.0, attack, attack == 1.0));
                signs.push(free_jump_sign(true));
                let mut clone = 1.0 - player.clone_timer as f64 / ticks(cfg.exeller.clone_recharge_seconds);
                let mut clone_sprite = sprite::SPR_GUI_EXELLERCLONE;
                if up_or_down_held && player.clone_count > 0 {
                    clone_sprite = sprite::SPR_GUI_EXELLERCLONE2;
                    clone = 1.0;
                }
                if clone_sprite == sprite::SPR_GUI_EXELLERCLONE && player.clone_count >= 2 {
                    clone = 0.0;
                }
                signs.push(gauge(Control::C, clone_sprite, 0.0, 4.0, 250.0, clone, clone == 1.0));
            }
        },
        Character::Tails => {
            let flight = (player.fly_timer.min(0) as f64) / -ticks(cfg.tails.flight_recharge_seconds);
            signs.push(gauge(Control::A, sprite::SPR_GUI_TAILSFLY, frame_if(demon), 10.0, 240.0, flight, flight == 1.0));
            let recharge = if demon { cfg.tails.demonized_attack_recharge_seconds } else { cfg.tails.attack_recharge_seconds };
            let attack = 1.0 - player.attack_timer as f64 / ticks(recharge);
            signs.push(gauge(Control::B, sprite::SPR_GUI_TAILSATTACK, 0.0, 12.0, 250.0, attack, attack == 1.0));
        }
        Character::Knux => {
            let glide_recharge = ticks(cfg.knuckles.glide_recharge_seconds);
            let glide = 1.0 - (player.glide_timer as f64).min(glide_recharge) / glide_recharge;
            signs.push(gauge(Control::A, sprite::SPR_GUI_KNUXGLIDE, frame_if(demon), 10.0, 240.0, glide, glide == 1.0));
            let recharge = ticks(if demon { cfg.knuckles.demonized_attack_recharge_seconds } else { cfg.knuckles.attack_recharge_seconds });
            let attack = 1.0 - (player.attack_timer as f64).min(recharge) / recharge;
            signs.push(gauge(Control::B, sprite::SPR_GUI_KNUXATTACK, 0.0, 10.0, 258.0, attack, attack == 1.0));
        }
        Character::Eggman => {
            let double_jump = recharged(player.djump_recharge, ticks(cfg.eggman.double_jump_recharge_seconds));
            signs.push(gauge(Control::A, sprite::SPR_GUI_EGGDJUMP, 0.0, 10.0, 222.0, double_jump, double_jump == 1.0));
            let shield = recharged(player.shield_recharge, ticks(cfg.eggman.shield_recharge_seconds));
            signs.push(gauge(Control::B, sprite::SPR_GUI_EGGSHIELD, 0.0, 10.0, 240.0, shield, shield == 1.0));
            let tracker = recharged(player.tracker_recharge, ticks(cfg.eggman.tracker_recharge_seconds));
            signs.push(gauge(Control::C, sprite::SPR_GUI_EGGTRACK, 0.0, 10.0, 258.0, tracker, player.is_colliding && tracker == 1.0));
        }
        Character::Amy => {
            let big_jump = recharged(player.hjump_timer, ticks(cfg.amy.big_jump_recharge_seconds));
            signs.push(gauge(Control::A, sprite::SPR_GUI_AMYHJUMP, 0.0, 10.0, 240.0, big_jump, big_jump == 1.0));
            // The original shows Amy's hammer against Knuckles' recharge times.
            let recharge = if demon { cfg.knuckles.demonized_attack_recharge_seconds } else { cfg.knuckles.attack_recharge_seconds };
            let attack = recharged(player.attack_timer, ticks(recharge));
            signs.push(gauge(Control::B, sprite::SPR_GUI_AMYATTACK, 0.0, 10.0, 258.0, attack, attack == 1.0));
        }
        Character::Cream => {
            let flight = recharged(player.fly_timer, ticks(cfg.cream.fly_recharge_seconds));
            signs.push(gauge(Control::A, sprite::SPR_GUI_CREAMFLY, frame_if(demon), 10.0, 222.0, flight, flight == 1.0));
            let dash = recharged(player.dash_timer, ticks(cfg.cream.dash_recharge_seconds));
            signs.push(gauge(Control::B, sprite::SPR_GUI_CREAMDASH, 0.0, 10.0, 240.0, dash, dash == 1.0));
            // Undemonized, the original measures the rings against the dash recharge.
            let recharge = if demon { cfg.cream.demonized_rings_recharge_seconds } else { cfg.cream.dash_recharge_seconds };
            let rings = recharged(player.rings_timer, ticks(recharge));
            signs.push(gauge(Control::C, sprite::SPR_GUI_CREAMRINGS, frame_if(demon), 10.0, 257.0, rings, rings == 1.0 && !player.is_colliding));
        }
        Character::Sally => {
            let recharge = if demon { cfg.sally.demonized_attack_recharge_seconds } else { cfg.sally.attack_recharge_seconds };
            let attack = recharged(player.attack_timer, ticks(recharge));
            signs.push(gauge(Control::B, sprite::SPR_GUI_SALLYATTACK, frame_if(demon), 10.0, 240.0, attack, attack == 1.0 && !player.is_grounded));
            let recharge = if demon { cfg.sally.demonized_shield_recharge_seconds } else { cfg.sally.shield_recharge_seconds };
            let shield = recharged(player.shield_recharge, ticks(recharge));
            signs.push(gauge(Control::C, sprite::SPR_GUI_SALLYSHIELD, frame_if(demon), 10.0, 258.0, shield, shield == 1.0));
        }
    }
    // With touch controls the emotion buttons show their faces already (touch.rs).
    let can_emote = player.character != Character::Exe && (player.state == crate::core::player::IDLE || player.emotion);
    if can_emote && touch.is_none() {
        // spr_gui_emotions at (470, 222), its faces one under another every 14 px.
        for (row, button) in EMOTION_BUTTONS.into_iter().enumerate() {
            let Some((face, _)) = button.icon(false) else { continue };
            let face = ButtonSign { button, sprite: face, frame: 0.0, color: C_WHITE, alpha: EMOTION_FACE_ALPHA };
            signs.push((face, EMOTION_FACES_X, EMOTION_FACES_Y + row as f64 * EMOTION_FACE_STEP));
        }
    }
    // With touch controls they are drawn over their buttons (touch.rs), which say
    // which they are; else here, with the keys they are used with written after them.
    match touch {
        Some(on_buttons) => on_buttons.extend(signs.into_iter().map(|(sign, _, _)| sign)),
        None => {
            for (sign, x, y) in signs {
                canvas.draw_sprite_ext(sign.sprite, sign.frame, x, y, 1.0, 1.0, 0.0, sign.color, sign.alpha);
                draw_hint(canvas, &sign, x, y, keys);
            }
        }
    }
}

/// A key a sign names, as the player has it bound.
#[derive(Clone, Copy)]
enum Bound {
    Up,
    Down,
    Jump,
    Special1,
    Special2,
    Emotion1,
    Emotion2,
    Emotion3,
}

impl Bound {
    fn key(self, keys: &KeyBindings) -> macroquad::input::KeyCode {
        match self {
            Bound::Up => keys.up.0,
            Bound::Down => keys.down.0,
            Bound::Jump => keys.jump.0,
            Bound::Special1 => keys.special1.0,
            Bound::Special2 => keys.special2.0,
            Bound::Emotion1 => keys.emotion1.0,
            Bound::Emotion2 => keys.emotion2.0,
            Bound::Emotion3 => keys.emotion3.0,
        }
    }
}

/// What a sign says after its picture, written left to right.
#[derive(Clone, Copy)]
enum Say {
    /// In the menu lettering, which is the lettering the pictures had.
    Text(&'static str),
    Key(Bound),
    /// A frame of spr_gui_hint_directions: which way to hold.
    Directions(f64),
    /// In the chat lettering: the small "(hold)" under Tails' shot.
    Small(&'static str),
}

/// A line a sign says, from where the picture had it (its own pixels, from its origin's corner).
struct HintLine {
    x: f64,
    top: f64,
    says: &'static [Say],
}

const fn line(x: f64, top: f64, says: &'static [Say]) -> HintLine {
    HintLine { x, top, says }
}

use Bound::*;
use Say::{Directions, Key, Small, Text};

/// The keys the gui signs had drawn in after their pictures ("- x", "- ↓ + z"), now
/// written as the player binds them. By sprite and, where the frame says something
/// else, by frame.
const HINTS: &[(SpriteId, Option<u8>, &[HintLine])] = &[
    (sprite::SPR_GUI_AMYATTACK, None, &[line(18.0, 5.0, &[Text("-"), Key(Special1)])]),
    (sprite::SPR_GUI_AMYHJUMP, None, &[line(12.0, 1.0, &[Text("-"), Key(Down), Text("+"), Key(Jump)])]),
    (sprite::SPR_GUI_CHAOSATTACK, Some(0), &[line(14.0, 3.0, &[Text("-"), Key(Special1)])]),
    (sprite::SPR_GUI_CHAOSATTACK, Some(1), &[line(14.0, 3.0, &[Text("-"), Key(Special1), Text("+"), Directions(0.0)])]),
    (sprite::SPR_GUI_CHAOSSLIME, None, &[line(13.0, 3.0, &[Text("-"), Key(Special2)])]),
    (sprite::SPR_GUI_CHAOSWALLDASH, Some(0), &[line(14.0, 3.0, &[Text("-"), Key(Special1), Text("+"), Directions(1.0)])]),
    (sprite::SPR_GUI_CHAOSWALLDASH, Some(1), &[line(14.0, 3.0, &[Text("-"), Key(Special1), Text("+"), Directions(2.0)])]),
    (sprite::SPR_GUI_CHAOSWALLDASH, Some(2), &[line(14.0, 3.0, &[Text("-"), Key(Special1), Text("+"), Directions(3.0)])]),
    (sprite::SPR_GUI_CREAMDASH, None, &[line(16.0, 2.0, &[Text("-"), Key(Special1)])]),
    (sprite::SPR_GUI_CREAMFLY, None, &[line(20.0, 6.0, &[Text("-"), Key(Jump)])]),
    (sprite::SPR_GUI_CREAMRINGS, None, &[line(16.0, 2.0, &[Text("-"), Key(Special2)])]),
    (sprite::SPR_GUI_EGGDJUMP, None, &[line(15.0, 3.0, &[Text("-"), Key(Jump)])]),
    (sprite::SPR_GUI_EGGSHIELD, None, &[line(15.0, 0.0, &[Text("-"), Key(Special1)])]),
    (sprite::SPR_GUI_EGGTRACK, None, &[line(18.0, 1.0, &[Text("-"), Key(Down), Text("+"), Key(Special2)])]),
    (sprite::SPR_GUI_EMOTION_ANGER, None, &[line(13.0, 2.0, &[Text("-"), Key(Emotion1)])]),
    (sprite::SPR_GUI_EMOTION_HAPPINESS, None, &[line(13.0, 2.0, &[Text("-"), Key(Emotion3)])]),
    (sprite::SPR_GUI_EMOTION_MISERY, None, &[line(13.0, 2.0, &[Text("-"), Key(Emotion2)])]),
    (sprite::SPR_GUI_EXEATTACK, None, &[line(14.0, 0.0, &[Text("-"), Key(Special1)])]),
    // The minigame's spin charge (obj_minigame), which spins with jump.
    (sprite::SPR_MINISNOC_CHARG, None, &[line(12.0, 0.0, &[Text("-"), Key(Jump)])]),
    (sprite::SPR_GUI_EXEFREEJUMP, None, &[line(21.0, 6.0, &[Text("-"), Key(Jump)])]),
    (sprite::SPR_GUI_EXEINVISABILITY, None, &[line(20.0, 5.0, &[Text("-"), Key(Special2)])]),
    (sprite::SPR_GUI_EXELLERCLONE, None, &[line(15.0, 5.0, &[Text("-"), Key(Special2)])]),
    (sprite::SPR_GUI_EXELLERCLONE2, None, &[line(13.0, 2.0, &[Text("-"), Key(Down), Text("/"), Key(Up), Text("+"), Key(Special2)])]),
    (sprite::SPR_GUI_EXETIORATTACK, Some(0), &[line(14.0, 0.0, &[Text("-"), Key(Special1)])]),
    (sprite::SPR_GUI_EXETIORATTACK, Some(1), &[line(14.0, 0.0, &[Text("-"), Key(Special1), Text("+"), Key(Down)])]),
    (sprite::SPR_GUI_EXETIORRING, None, &[line(15.0, 3.0, &[Text("-"), Key(Special2)])]),
    (sprite::SPR_GUI_KNUXATTACK, None, &[line(14.0, 0.0, &[Text("-"), Key(Special1)])]),
    (sprite::SPR_GUI_KNUXGLIDE, None, &[line(18.0, 3.0, &[Text("-"), Key(Jump)])]),
    (sprite::SPR_GUI_SALLYATTACK, None, &[line(12.0, 1.0, &[Text("-"), Key(Special1)])]),
    (sprite::SPR_GUI_SALLYSHIELD, None, &[line(14.0, 1.0, &[Text("-"), Key(Special2)])]),
    (sprite::SPR_GUI_TAILSATTACK, None, &[line(19.0, 2.0, &[Text("-"), Key(Special1)]), line(2.0, 12.0, &[Small("(hold)")])]),
    (sprite::SPR_GUI_TAILSFLY, None, &[line(21.0, 0.0, &[Text("-"), Key(Jump)])]),
];

/// Air between the pieces of a hint, as the pictures left it.
const HINT_SPACING: f64 = 1.0;

/// The keys a sign drawn at (x, y) is used with, in its color and opacity.
pub fn draw_hint(canvas: &mut Canvas, sign: &ButtonSign, x: f64, y: f64, keys: &KeyBindings) {
    let frame = sign.frame as u8;
    let Some((_, _, lines)) = HINTS.iter().find(|(sprite, only, _)| *sprite == sign.sprite && only.is_none_or(|only| only == frame)) else { return };
    let meta = canvas.sprites.get(sign.sprite);
    let (left, top) = (x - meta.origin_x, y - meta.origin_y);
    for line in lines.iter() {
        let (mut pen, line_top) = (left + line.x, top + line.top);
        for say in line.says {
            match *say {
                Text(text) => {
                    canvas.draw_menu_text(pen, line_top, text, sign.color, sign.alpha);
                    pen += canvas.menu_text_width(text);
                }
                Key(bound) => {
                    let text = crate::client::input::key_sign(bound.key(keys));
                    canvas.draw_menu_text(pen, line_top, text, sign.color, sign.alpha);
                    pen += canvas.menu_text_width(text);
                }
                Directions(frame) => {
                    canvas.draw_sprite_ext(sprite::SPR_GUI_HINT_DIRECTIONS, frame, pen, line_top, 1.0, 1.0, 0.0, sign.color, sign.alpha);
                    pen += canvas.sprites.get(sprite::SPR_GUI_HINT_DIRECTIONS).width as f64;
                }
                Small(text) => pen += draw_text(canvas, pen, line_top, text, sign.color, sign.alpha),
            }
            pen += HINT_SPACING;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Health is counted in hits now, and the pictures still have five steps: a
    /// survivor with every hit left shows a full icon and a full bar, one hit from
    /// the end shows the last of each.
    #[test]
    fn the_health_pictures_follow_the_hits_a_survivor_has() {
        let max_hp = GameplayConfig::default().hurt.max_hp;
        assert_eq!(hits_lost(max_hp, max_hp), 0.0, "untouched");
        assert_eq!(hits_lost(1, max_hp), HEALTH_STEPS - 1.0, "one hit left");
        assert_eq!(hits_lost(0, max_hp), HEALTH_STEPS, "gone");
        assert_eq!(health_bar_frame(max_hp, max_hp), HEALTH_STEPS, "a full bar");
        assert_eq!(health_bar_frame(1, max_hp), 1.0, "the last of the bar");
        // A server that gives survivors more or fewer hits spreads the same five steps over them.
        assert_eq!(hits_lost(5, 10), (HEALTH_STEPS / 2.0).round_ties_even(), "half of ten hits");
    }
}
