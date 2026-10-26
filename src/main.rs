// On Windows the program is a window program, so the game opens no console; what
// needs one opens it (windows_console below).
#![cfg_attr(windows, windows_subsystem = "windows")]

mod palette_check;
mod client;
mod colors;
mod config;
mod hosting;
mod local_link;
mod format;
mod connection;
mod terminal;
mod maps;
mod moderation;
mod packet;
mod player;
mod server;
mod core;
mod states;
mod status;
mod vote;
mod worker;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use packet::VERSION;

/// What a release build says when Resources.tar is not beside it. Both the server and
/// the game need it: the server plays the rounds on the same maps.
#[cfg(not(debug_assertions))]
const MISSING_RESOURCES: &str = "\
Игра не может найти папку с ресурсами, учтите, что папку с ресурсами извлекать не надо, \
это сделано для экономного чтения ассетов с диска (Да, та самая оригинальная TD2D на \
GameMaker или Sonic Mania на RetroEngine не зря имеет один или ~5 файлов с ассетами). \
Я лишь сделал свой вариант data.win, чтобы вы могли их разорхивировать, поменять \
что-нибудь, И УПАКОВАТЬ ОБРАТНО!

Еще одна причина почему игра не может найти файл с ассетами (a. k. a.: ресурсы), это \
если ты запустил игру из архива. ТВОЙ АРХИВАТОР ПЕРЕД ЗАПУСКОМ ЧЕГО-ТО РАСПАКОВЫВАЕТ \
ТОЛЬКО ВЫБРАННЫЙ ФАЙЛ из архива во временную папку в глубинке твоего HDD и которая \
удаляется при перезагрузке, чтобы запустить. Так что не думай, что запуск из архива это \
какая-то магия. Это магия только чтобы ты слушал музыку, которую твой друг скинул в .zip \
архиве. А с игрой это не работает! Ей нужен конкретный файл, чтобы он лежал рядом с ней!

Пожалуйста, не пишите эту проблему разработчику И НОРМАЛЬНО ВСЕ РАСПАКУЙТЕ! \
Правая кнопка мыши -> 7-zip, патый всерху вниз вариант \"Extract to FOLDER_NAME/\"";

/// Says `text` and ends the program: in the terminal, and on Windows also in a dialog,
/// as a program started with a double click may have no terminal anyone reads.
#[cfg(not(debug_assertions))]
fn fatal_error(text: &str) -> ! {
    eprintln!("{text}");
    #[cfg(windows)]
    show_error_dialog(client::GAME_TITLE, text);
    std::process::exit(1);
}

/// MessageBoxW of user32.dll, which every Windows has, so no library is needed for it.
#[cfg(all(windows, not(debug_assertions)))]
fn show_error_dialog(title: &str, text: &str) {
    #[link(name = "user32")]
    unsafe extern "system" {
        fn MessageBoxW(window: *mut std::ffi::c_void, text: *const u16, caption: *const u16, kind: u32) -> i32;
    }
    const MB_ICONERROR: u32 = 0x10;
    let wide = |text: &str| text.encode_utf16().chain([0]).collect::<Vec<u16>>();
    let (text, title) = (wide(text), wide(title));
    // Safe: both strings end with the 0 Windows looks for and outlive the call.
    unsafe {
        MessageBoxW(std::ptr::null_mut(), text.as_ptr(), title.as_ptr(), MB_ICONERROR);
    }
}

/// The consoles of a window program (Windows): the program has none until it opens one.
/// Elsewhere a program has the terminal it was started from, and these do nothing.
mod windows_console {
    #[cfg(windows)]
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn AllocConsole() -> i32;
        fn AttachConsole(process: u32) -> i32;
    }
    #[cfg(windows)]
    const ATTACH_PARENT_PROCESS: u32 = u32::MAX;

    /// A console window of the program's own.
    pub fn open_window() {
        // Safe: no arguments; a program that has a console already keeps it.
        #[cfg(windows)]
        unsafe {
            AllocConsole();
        }
    }

    /// The command prompt the program was started from, or a window of its own when it
    /// was started with a double click. Returns whether it is a window of its own.
    pub fn use_the_prompts_or_open() -> bool {
        // Safe: no pointers are passed.
        #[cfg(windows)]
        unsafe {
            if AttachConsole(ATTACH_PARENT_PROCESS) == 0 {
                AllocConsole();
                return true;
            }
        }
        false
    }

    /// A window of the program's own closes with it: it waits until what it says was read.
    pub fn wait_for_enter_in(own_window: bool) {
        if cfg!(windows) && own_window {
            println!("press enter to close");
            let _ = std::io::stdin().read_line(&mut String::new());
        }
    }
}

/// Any of these on the command line starts the server instead of the game.
const SERVER_FLAGS: [&str; 3] = ["--host", "--server", "--dedicated"];

fn main() {
    if std::env::args().any(|argument| argument == "--help") {
        let own_window = windows_console::use_the_prompts_or_open();
        println!("{}", client::USAGE);
        windows_console::wait_for_enter_in(own_window);
        return;
    }
    #[cfg(not(debug_assertions))]
    if !core::resources::enter_program_folder() {
        fatal_error(MISSING_RESOURCES);
    }
    if std::env::args().any(|argument| argument == "--update") {
        // It asks a question, which a command prompt would take the answer to as well.
        windows_console::open_window();
        client::updater::update_from_command_line();
        windows_console::wait_for_enter_in(true);
        return;
    }
    // The game, unless the server is asked for: started with a double click, the
    // program is something to play. Without graphics there is no game to show, and
    // what the program can still be is the server.
    let server_asked_for = std::env::args().any(|argument| SERVER_FLAGS.contains(&argument.as_str()));
    if !server_asked_for && client::graphics_available() {
        client::run();
        return;
    }
    // The server's terminal reads the keyboard, which it would fight a command prompt over.
    windows_console::open_window();

    terminal::init_logger();
    print_banner();
    if !server_asked_for {
        log::warn!("no graphics here to start the game (no Wayland display), so the server is started instead");
    }

    if let Err(error) = hosting::load_settings() {
        log::error!("{error}");
        std::process::exit(1);
    }
    let running = install_signal_handlers();
    let hosted = hosting::start(running.clone(), None);
    hosted.run_console();
    hosted.stop();
}

/// Prints the startup banner. Goes through print_always rather than log::info so
/// it shows even at log_level 0 (see Logging::log_level in config.rs).
fn print_banner() {
    let target = module_path!();
    for line in [
        "---------AdvancedServer---------".to_string(),
        "Created by: The Arctic Fox".to_string(),
        "Contains code referenced from: BetterServer (MIT)".to_string(),
        format!("Version: {VERSION}"),
        format!(
            "Built on {} at {} for {} via {}",
            env!("BUILD_DATE"),
            env!("BUILD_TIME"),
            env!("BUILD_TARGET_OS"),
            env!("BUILD_RUSTC_VERSION"),
        ),
        "Licensed under GNU AGPL 3.0. Source code available at: \
         https://thearcticfox25.github.io/AdvancedServer/".to_string(),
        "--------------------------------".to_string(),
    ] {
        terminal::print_always(target, &line);
    }
}

/// Installs the signal handlers and returns the flag every loop in the process
/// polls: false means "shut down".
///
/// signal_hook::flag::register can only ever set a flag to `true`, never back to
/// `false`, so the shutdown signals set a flag of their own and a watcher thread
/// translates that into `running = false`.
fn install_signal_handlers() -> Arc<AtomicBool> {
    let running = Arc::new(AtomicBool::new(true));

    #[cfg(not(windows))]
    let shutdown_signals = &[
        signal_hook::consts::SIGINT,
        signal_hook::consts::SIGTERM,
        signal_hook::consts::SIGHUP,
    ];
    #[cfg(windows)]
    let shutdown_signals = &[
        signal_hook::consts::SIGINT,
        signal_hook::consts::SIGTERM,
    ];

    let term_requested = Arc::new(AtomicBool::new(false));
    for &sig in shutdown_signals {
        if let Err(error) = signal_hook::flag::register(sig, term_requested.clone()) {
            log::warn!("failed to register signal {} handler: {}", sig, error);
        }
    }

    let watcher_flag = running.clone();
    std::thread::spawn(move || {
        while !term_requested.load(Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        watcher_flag.store(false, Ordering::Relaxed);
    });

    #[cfg(not(windows))]
    if let Err(error) = signal_hook::flag::register(
        signal_hook::consts::SIGPIPE,
        Arc::new(AtomicBool::new(false)),
    ) {
        log::warn!("failed to ignore sigpipe: {}", error);
    }

    install_crash_handler();
    running
}

/// Best-effort crash handling: on SIGABRT, try to flush round stats to
/// Status.json before the process dies. Uses try_lock (see
/// save_status_best_effort) rather than the normal save_status, since abort() on
/// this same thread could have been triggered while it already held that lock.
///
/// SIGSEGV is deliberately NOT handled here: signal-hook-registry refuses to
/// register it at all (it is in its hard-coded FORBIDDEN list, alongside
/// SIGILL/SIGFPE) because a segfault leaves the process in a state where running
/// arbitrary handler code is unsound, and Rust's std already installs its own
/// SIGSEGV handler to detect stack overflow specifically. Overriding that via a
/// lower-level unchecked API would fight the runtime's own safety net for a case
/// (memory corruption via unsafe/FFI code) where nothing
/// could be safely saved anyway.
#[cfg(not(windows))]
fn install_crash_handler() {
    let registered = unsafe {
        signal_hook::low_level::register(signal_hook::consts::SIGABRT, || {
            status::save_status_best_effort();
            signal_hook::low_level::exit(1);
        })
    };
    if let Err(error) = registered {
        log::warn!("failed to register SIGABRT crash handler: {}", error);
    }
}

#[cfg(windows)]
fn install_crash_handler() {}
