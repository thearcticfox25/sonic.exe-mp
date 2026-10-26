//! Starting and stopping the server's lobbies: by `--host` (main.rs), which then reads
//! the terminal, or by the game's IServer button, whose player is in the first lobby
//! through local_link, while the terminal (if the game was started from one) still
//! takes the server's commands as it does for `--host`.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, SyncSender};
use std::sync::Arc;

use crate::config::{self, load_config, CONFIG};
use crate::local_link::ServerEnd;
use crate::server::ServerShared;
use crate::terminal::{self, Console, ConsoleCmd};
use crate::worker::server_worker;

/// Reads Config.toml (once a process) and sets up what every lobby shares: the log,
/// the gameplay numbers, the moderation lists and the status file. A configuration
/// that cannot work is refused, unless the owner said they know better.
pub fn load_settings() -> Result<(), String> {
    if CONFIG.get().is_some() {
        return Ok(());
    }
    let cfg = load_config().map_err(|error| format!("failed to load config: {error}"))?;
    if !cfg.verify() {
        if !cfg.miscellaneous.other.ignore_inadequate_configuration {
            log::error!("Ильич, ты чо, долбаēб что ли?");
            return Err("configuration verification failed. fix the config or set ignore_inadequate_configuration = true.".to_string());
        }
        log::warn!("configuration verification failed but ignore_inadequate_configuration is true. continuing.");
    }
    crate::core::config::set_tick_rate(cfg.server_config.networking.tick_rate);
    CONFIG.set(cfg).ok();
    terminal::init_file_logging();
    // Read (and PersistentStorage/Core written) now, not at the first round, so the
    // folder is there to be edited as soon as the server has started once.
    crate::core::config::gameplay_config();
    // RUST_LOG, when set, wins over the config file: it is the developer's
    // override and init_logger already honoured it.
    if std::env::var("RUST_LOG").is_err() {
        log::set_max_level(terminal::level_filter_for(config::cfg().server_config.window.logging.log_level));
    }
    crate::moderation::init_moderation();
    crate::status::init_status();
    Ok(())
}

/// The running lobbies: their threads, the console's way into each, and the live
/// peer counts the console and the lobby-redirect logic read out of them.
pub struct Hosted {
    pub running: Arc<AtomicBool>,
    senders: Vec<SyncSender<ConsoleCmd>>,
    shared: Vec<Arc<ServerShared>>,
    threads: Vec<std::thread::JoinHandle<()>>,
    /// The terminal's thread in a game that hosts, which reads it beside the game.
    console: Option<std::thread::JoinHandle<()>>,
}

/// Starts one worker thread per configured lobby; the first one also serves
/// `host_game`, the game that started it.
pub fn start(running: Arc<AtomicBool>, host_game: Option<ServerEnd>) -> Hosted {
    let cfg = config::cfg();
    let base_port = cfg.server_config.networking.port;
    let server_count = cfg.server_config.networking.server_count;

    terminal::print_always(module_path!(), &format!("starting {} server(s) on base port {}", server_count, base_port));

    // Built up front so every worker can see every sibling instance's live peer
    // count, which is what lets a full lobby redirect a player to a free one
    // instead of rejecting them (see connection::handle_identity).
    let shared: Vec<Arc<ServerShared>> = (0..server_count).map(|_| Arc::new(ServerShared::new())).collect();

    let mut senders = Vec::new();
    let mut threads = Vec::new();
    let mut host_game = host_game;
    for id in 0..server_count {
        let (tx, rx) = mpsc::sync_channel::<ConsoleCmd>(64);
        senders.push(tx);
        let running = running.clone();
        let own_shared = shared[id as usize].clone();
        let all_shared = shared.clone();
        let port = base_port + id;
        let local = host_game.take();
        threads.push(std::thread::spawn(move || {
            supervise_instance(id, port, running, rx, own_shared, all_shared, local);
        }));
    }
    Hosted { running, senders, shared, threads, console: None }
}

impl Hosted {
    /// The terminal takes commands (server_config.window.terminal.enable) or only shows
    /// the log, until the server is stopped.
    pub fn run_console(&self) {
        // Let every instance bind and print its "listening" line before the console
        // prompt appears in the middle of them.
        std::thread::sleep(std::time::Duration::from_millis(150));
        if config::cfg().server_config.window.terminal.enable {
            Console::new(self.senders.clone(), self.shared.clone()).run(self.running.clone());
        } else {
            Console::show_log_only(self.running.clone());
        }
    }

    /// IServer started from a terminal: the terminal is the server's, as for `--host`,
    /// read beside the game until the server stops.
    pub fn start_console_thread(&mut self) {
        let (senders, shared, running) = (self.senders.clone(), self.shared.clone(), self.running.clone());
        self.console = Some(std::thread::spawn(move || {
            if config::cfg().server_config.window.terminal.enable {
                Console::new(senders, shared).run(running);
            } else {
                Console::show_log_only(running);
            }
        }));
    }

    pub fn stop(self) {
        self.running.store(false, Ordering::Relaxed);
        for handle in self.threads.into_iter().chain(self.console) {
            let _ = handle.join();
        }
        crate::status::save_status();
        log::info!("server stopped.");
    }
}

/// Runs one server instance, restarting it if it panics.
///
/// A panic inside the worker unwinds only its own thread, but without a restart
/// the instance would be silently dead for the life of the process. A normal
/// return (clean shutdown, or a fatal bind error) ends the loop instead.
fn supervise_instance(
    server_id: u16,
    port: u16,
    running: Arc<AtomicBool>,
    cmd_rx: mpsc::Receiver<ConsoleCmd>,
    shared: Arc<ServerShared>,
    all_shared: Vec<Arc<ServerShared>>,
    host_game: Option<ServerEnd>,
) {
    // ponytail: the host's channel goes to the worker's first run only; after a panic
    // the restarted lobby has no host game, which then sees its server gone.
    let mut host_game = host_game;
    while running.load(Ordering::Relaxed) {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            server_worker(server_id, port, running.clone(), &cmd_rx, shared.clone(), all_shared.clone(), host_game.take());
        }));
        if result.is_ok() {
            break;
        }
        log::error!("{} worker panicked; restarting instance in 1s", crate::server::lobby_tag(server_id));
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
