//! The game client: window, drawing, sound, input and the screens (rooms) of the
//! original. `AdvancedServer` starts it, with no options or with --play.

pub mod achievements;
mod audio;
mod canvas;
mod window_shape;
mod device;
mod credits;
mod font;
mod gamepad;
mod input;
mod launch;
pub use launch::USAGE;
mod levels;
mod net;
mod ogg_stream;
mod options;
mod palette;
mod room;
mod screens;
mod text;
mod touch;
mod unlockables;
pub mod updater;

use audio::Audio;
use canvas::Canvas;
use input::Input;
use net::NetClient;
use macroquad::prelude::*;
use options::Options;
use screens::Screen;
use unlockables::Unlockables;
use std::path::PathBuf;
use std::sync::Arc;
use crate::core::resources::sprites::Sprites;
use crate::core::config::ticks_per_second;
use crate::core::rooms::ids::RoomId;

pub const GAME_TITLE: &str = "Sonic.EXE Multiplayer";

/// The fonts, in Fonts/: the menu's words, the big title letters, the chat's and the
/// headers'. All but big are laid out by tools/build_fonts.py (Fonts.toml).
const MENU_FONT: &str = "menu";
const BIG_FONT: &str = "big.png";
const LETTER_FONT: &str = "letter";
const HEADER_FONT: &str = "Header";

/// The window icon in Textures, and its size in pixels.
const ICON_FILE: &str = "icon.png";
const ICON_SIZE: u32 = 64;

/// After a stall (a dragged window, a breakpoint) the game catches up at most this
/// many ticks in one frame and forgets the rest of the lost time.
const MAX_TICKS_PER_FRAME: u32 = 5;
/// A frame no longer than this many ticks is an ordinary one: a room that opened has
/// settled once its frames come this fast (see `settling` in run).
const SETTLED_FRAME_TICKS: f64 = 2.0;

/// What every screen works with; the GameMaker persistent objects (obj_config,
/// obj_controls) and built-in systems live here.
pub struct Context {
    pub canvas: Canvas,
    pub input: Input,
    pub audio: Audio,
    pub options: Options,
    /// obj_unlockables: skins, pet and lobby icon.
    pub unlockables: Unlockables,
    /// obj_achivements
    pub achievements: achievements::Achievements,
    /// obj_netclient
    pub net: NetClient,
    /// The main menu's room, read while the logos show (screens/logo.rs), so the menu
    /// opens at once when they end; taken by the menu as it opens.
    pub prepared_menu: Option<room::Room>,
    /// global.errorCode: the message room_message shows.
    pub error_code: u8,
    /// global.muteChat
    pub chat_muted: bool,
    /// obj_controls.prevScreenMode: the window size F4 returns to from fullscreen.
    pub windowed_screen_mode: u8,
    pub resources_folder: PathBuf,
    /// game_end() was called.
    pub quit: bool,
    /// --netlog
    pub netlog: bool,
    /// obj_controls' touch controls.
    pub touch: touch::TouchControls,
    /// obj_controls' gamepad.
    pub gamepad: gamepad::Gamepad,
    /// IServer: the server this game started and plays in (net.rs host).
    pub hosted: Option<crate::hosting::Hosted>,
    /// The main menu's updater, which looks for a new release from the start.
    pub updater: updater::Updater,
}

/// Whether a window can be opened here. On Linux a window needs a Wayland session,
/// which says where it is in WAYLAND_DISPLAY; a server reached over SSH has none.
/// Windows and macOS always have one.
pub fn graphics_available() -> bool {
    if !cfg!(target_os = "linux") {
        return true;
    }
    let set = |name: &str| std::env::var_os(name).is_some_and(|value| !value.is_empty());
    set("WAYLAND_DISPLAY")
}

pub fn run() {
    // Seeded first: the default options pick a random nickname, --character a random pick.
    rand::srand(miniquad::date::now() as u64);
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let launch = match launch::Launch::parse(&arguments) {
        Ok(launch) => launch,
        Err(error) => {
            eprintln!("{error}\n\n{}", launch::USAGE);
            std::process::exit(2);
        }
    };
    let options = Options::load();
    audio::name_the_stream(GAME_TITLE);
    let window = options::window_size(options.screen_mode);
    let conf = Conf {
        window_title: GAME_TITLE.to_string(),
        window_width: window.width,
        window_height: window.height,
        fullscreen: window.fullscreen,
        window_resizable: true,
        icon: window_icon(),
        platform: miniquad::conf::Platform {
            // display_reset(0, true) in obj_logo: vsync on.
            swap_interval: Some(1),
            // The window's app id names the program, not the library: the desktop tells
            // two games' windows and their sounds apart by it and the process.
            linux_wm_class: "AdvancedServer",
            // A Wayland window: the desktop knows its process, which it matches the
            // game's sound to.
            linux_backend: miniquad::conf::LinuxBackend::WaylandOnly,
            ..Default::default()
        },
        ..Default::default()
    };
    macroquad::Window::from_config(conf, main_loop(options, launch));
}

/// The game's icon (options/windows/icons/icon.ico of the original): Textures/icon.png,
/// 64 by 64, and the 32 and 16 pixel ones a window also takes made from it. Without the
/// file the window keeps the default.
fn window_icon() -> Option<miniquad::conf::Icon> {
    let path = crate::core::resources::content_folder().join(crate::core::resources::TEXTURES).join(ICON_FILE);
    let image = crate::format::apng::read(&path).inspect_err(|error| eprintln!("no window icon: {error:#}")).ok()?;
    if (image.width, image.height) != (ICON_SIZE, ICON_SIZE) {
        eprintln!("no window icon: {} is not {ICON_SIZE} by {ICON_SIZE}", path.display());
        return None;
    }
    let big = image.frames.into_iter().next()?;
    Some(miniquad::conf::Icon {
        small: shrink_icon(&big, 4).try_into().ok()?,
        medium: shrink_icon(&big, 2).try_into().ok()?,
        big: big.try_into().ok()?,
    })
}

/// The icon made `factor` times smaller: each pixel the average of the square of pixels
/// it stands for, which is how the 16 and 32 pixel icons of the original were made.
fn shrink_icon(rgba: &[u8], factor: u32) -> Vec<u8> {
    let size = ICON_SIZE / factor;
    let mut out = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            for channel in 0..4 {
                let mut sum = 0u32;
                for dy in 0..factor {
                    for dx in 0..factor {
                        let index = (((y * factor + dy) * ICON_SIZE + x * factor + dx) * 4 + channel) as usize;
                        sum += rgba[index] as u32;
                    }
                }
                out.push(((sum + factor * factor / 2) / (factor * factor)) as u8);
            }
        }
    }
    out
}

async fn main_loop(options: Options, launch: launch::Launch) {
    let mut context = match create_context(options, &launch) {
        Ok(context) => context,
        Err(error) => {
            eprintln!("the game cannot start: {error:#}");
            return;
        }
    };
    let first_room = first_room(&launch.start, &mut context);
    let mut screen = match Screen::open(first_room, &mut context) {
        Ok(screen) => screen,
        Err(error) => {
            eprintln!("the game cannot start: {error:#}");
            return;
        }
    };
    let mut unspent_seconds = 0.0;
    // A room just opened: loading it, and uploading its pictures the first time they
    // are drawn, takes long frames. Caught up on, that time would play the room's first
    // moments unseen (the menu's white flash among them), so while the frames stay that
    // long the room advances one tick a frame, as if no time had been lost.
    let mut settling = true;
    // The window's close button ends the loop below rather than the program, so the
    // server is told this player left (NetClient::leave_server).
    prevent_quit();
    let server_ended = server_end_watch(&launch.start);
    let mut window_shape = window_shape::WindowShape::default();

    while !context.quit {
        if is_quit_requested() || server_ended.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }
        context.input.collect_frame_events();
        context.canvas.heat_haze_on = context.options.heat_haze;
        window_shape.keep_16_9();
        let tick_seconds = 1.0 / ticks_per_second();
        let frame_seconds = get_frame_time() as f64;
        if settling {
            unspent_seconds = tick_seconds;
            settling = frame_seconds > SETTLED_FRAME_TICKS * tick_seconds;
        } else {
            unspent_seconds += frame_seconds;
        }
        let mut ticks = 0;
        while unspent_seconds >= tick_seconds && ticks < MAX_TICKS_PER_FRAME {
            unspent_seconds -= tick_seconds;
            ticks += 1;
            let next_room = screen.tick(&mut context);
            context.achievements.tick();
            if context.achievements.take_new_unlock() {
                context.audio.play(crate::core::resources::names::sound::SND_ACHIVEMENT, false);
            }
            let Some(room) = next_room else { continue };
            match Screen::open(room, &mut context) {
                Ok(next_screen) => {
                    screen = next_screen;
                    // The new room starts on the next frame, its first one.
                    settling = true;
                    break;
                }
                Err(error) => {
                    eprintln!("room {room:?} failed to open: {error:#}");
                    context.quit = true;
                    break;
                }
            }
        }
        if ticks == MAX_TICKS_PER_FRAME {
            unspent_seconds = 0.0;
        }
        // Only over a picture a tick has just drawn: a frame without a tick shows the last
        // picture again, which has these on it already, and drawing them twice made the
        // see-through touch controls flicker darker.
        if ticks > 0 {
            context.canvas.set_view(0.0, 0.0);
            if context.options.touch_controls {
                context.touch.draw(&mut context.canvas, &context.options.touch_layout);
            }
            context.achievements.draw(&mut context.canvas);
        }
        context.canvas.present();
        next_frame().await;
    }
    context.net.leave_server();
    if let Some(hosted) = context.hosted.take() {
        hosted.stop();
    }
}

/// The server's ?gui window closes with the server: the server holds the window's
/// input open while it runs, and its end, however the server ended, sets the flag.
/// Any other game never sets it.
fn server_end_watch(start: &launch::Start) -> Arc<std::sync::atomic::AtomicBool> {
    let ended = Arc::new(std::sync::atomic::AtomicBool::new(false));
    if matches!(start, launch::Start::Observe { .. }) {
        let flag = ended.clone();
        std::thread::spawn(move || {
            let _ = std::io::copy(&mut std::io::stdin(), &mut std::io::sink());
            flag.store(true, std::sync::atomic::Ordering::Relaxed);
        });
    }
    ended
}

/// Where --map, --join or nothing on the command line starts the game.
fn first_room(start: &launch::Start, context: &mut Context) -> RoomId {
    match start {
        launch::Start::Logo => RoomId::Logo,
        launch::Start::Alone { level, pick, freecam } => {
            context.net.choose_alone(*pick, *freecam);
            levels::LEVELS[*level].room
        }
        launch::Start::Observe { address, key } => {
            context.net.observer_key = Some(key.clone());
            context.net.join(address, None);
            RoomId::Connecting
        }
        launch::Start::Join(address, lobby) => {
            // Saved as if typed on the join page, so the menu shows it next time.
            let address = address.clone().unwrap_or_else(|| context.options.ip.clone());
            context.options.ip = address.clone();
            context.options.save();
            // Lobby N listens on the N-th port from the first one, like `.lobby N` moves to.
            let (_, first_port) = net::split_port(&address);
            let lobby_port = lobby.map(|number| first_port.saturating_add(number - 1));
            context.net.join(&address, lobby_port);
            RoomId::Connecting
        }
    }
}

fn create_context(options: Options, launch: &launch::Launch) -> anyhow::Result<Context> {
    let content = crate::core::resources::content_folder();
    let sprites = Arc::new(Sprites::index(&content.join(crate::core::resources::TEXTURES))?);
    let fonts = content.join(crate::core::resources::FONTS);
    Ok(Context {
        canvas: Canvas::new(
                sprites,
                [
                    font::Font::load_laid_out(&fonts, MENU_FONT)?,
                    font::Font::load(&fonts.join(BIG_FONT))?,
                    font::Font::load_laid_out(&fonts, LETTER_FONT)?,
                    font::Font::load_laid_out(&fonts, HEADER_FONT)?,
                ],
            ),
        input: Input::default(),
        // The server's ?gui window is silent: it does not even open the sound device.
        audio: Audio::new(&content.join(crate::core::resources::SOUNDS), !matches!(launch.start, launch::Start::Observe { .. }))?,
        options,
        unlockables: Unlockables::load(),
        achievements: achievements::Achievements::load(),
        net: NetClient::new(),
        prepared_menu: None,
        error_code: 0,
        chat_muted: false,
        windowed_screen_mode: 0,
        resources_folder: content.clone(),
        quit: false,
        netlog: launch.netlog,
        touch: touch::TouchControls::default(),
        gamepad: gamepad::Gamepad::new(),
        hosted: None,
        updater: updater::Updater::start(),
    })
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_smaller_icon_pixel_is_the_average_of_its_square() {
        // Left half white, right half black: made smaller, each side keeps its color,
        // and the edge between them stays where it was.
        let mut icon = Vec::new();
        for _ in 0..ICON_SIZE {
            for x in 0..ICON_SIZE {
                let value = if x < ICON_SIZE / 2 { 255 } else { 0 };
                icon.extend([value, value, value, 255]);
            }
        }
        let half = shrink_icon(&icon, 2);
        assert_eq!(half.len(), 32 * 32 * 4);
        assert_eq!((half[0], half[31 * 4]), (255, 0));
        let quarter = shrink_icon(&icon, 4);
        assert_eq!(quarter.len(), 16 * 16 * 4);
        assert_eq!((quarter[7 * 4], quarter[8 * 4], quarter[3]), (255, 0, 255));
    }
}
