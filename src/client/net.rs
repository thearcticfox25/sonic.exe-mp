//! obj_netclient: the connection to the server and what the client knows about
//! the round (net_wrapper, net_client, net_identity, net_tcpprocess and the
//! per-state packet handlers). Packets update this data; what the rooms should
//! show comes back as notices, because in the original the handlers reached
//! into room objects (with(obj_lobby) ...).

use crate::client::Context;
use rusty_enet as enet;
use std::collections::BTreeMap;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::sync::Arc;
use crate::packet::{Packet, PacketType, ANY_CHARACTER, CHOOSE_VOTEKICK, DEFAULT_PORT, EXE_COUNT, NO_CHARACTER, SURVIVOR_COUNT, VERSION};
use crate::core::resources::names::sound;
use crate::core::events::{EventSource, SimEvent};
use crate::core::player::Buttons;
use crate::core::rooms::ids::RoomId;
use crate::core::snapshot::{RoundSetup, Snapshot};

/// ENet channels the server opens.
const CHANNEL_COUNT: usize = 2;
const CHANNEL: u8 = 0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NetState {
    Pending,
    Lobby,
    Vote,
    CharSelect,
    Game,
    Results,
}

/// An entry of obj_netclient.players.
#[derive(Clone, Debug)]
pub struct Player {
    pub nickname: String,
    pub icon: u8,
    pub is_ready: bool,
    /// The player's preference cards (packet::ANY_CHARACTER for no wish).
    pub preferred_exe: u8,
    pub preferred_survivor: u8,
    /// The character they play, by their side (packet::NO_CHARACTER while none is
    /// chosen); the side is whether they are on the round's list of EXE.
    pub character: u8,
    /// The pet that follows them in rounds (obj_unlockables.pet), NO_PET for none.
    /// Their pet's number (from 1), or NO_PET.
    pub pet: u8,
}

/// What a room should do about a packet.
#[derive(Clone, Debug, PartialEq)]
pub enum Notice {
    /// room_goto
    GoTo(RoomId),
    /// room_message showing global.errorCode.
    ShowError(u8),
    /// lobby_player_joined; `play_sound` is false for the player list sent on arrival.
    PlayerJoined { id: u16, play_sound: bool },
    /// lobby_player_left
    PlayerLeft(u16),
    /// SERVER_LOBBY_CORRECT: this client appears in the lobby and may act.
    JoinedLobby,
    ChatMessage { sender: String, text: String },
    Countdown { counting: bool, seconds: u8 },
    /// SERVER_VOTE_MAPS: the vote starts, on NetClient::vote_maps.
    VoteMaps,
    VoteTime(u8),
    VoteCounts([u8; 3]),
    /// SERVER_LOBBY_EXE: the vote is over; `map` will be played.
    ExeChosen { map: u16 },
    /// SERVER_IDENTITY_RESPONSE while a round is running.
    WaitForRound,
    WaitingPlayer { id: u16, nickname: String, in_round: bool, exe: bool, character: u8, icon: u8 },
    WaitingPlayerLeft(u16),
    RoundTime(u16),
    /// SERVER_GAME_SUDDEN_DEATH: the server says sudden death has begun.
    SuddenDeath,
    /// The server asks this client to pick a player to vote kick, kick, ban or make
    /// operator (packet::CHOOSE_*).
    ChoosePlayer(u8),
    /// Getting ready was refused: another ready player claimed this survivor (1-based).
    SurvivorTaken(u8),
    /// Getting ready was refused: another ready player claimed this EXE character (1-based).
    ExeTaken(u8),
    /// SERVER_CHAR_TIME_SYNC: seconds left on the character select stage.
    CharacterTime(u8),
    /// SERVER_LOBBY_GAME_START: the round begins.
    RoundStarts,
    /// SERVER_GAME_EXE_WINS, SERVER_GAME_SURVIVOR_WIN, SERVER_GAME_TIME_OVER.
    /// The round is over, and whether the server lets it count for achievements.
    RoundEnded(crate::core::game::RoundEnding, bool),
    /// SERVER_REVIVAL_RINGSUB: a teammate this player revived is up; their rings paid for it.
    RevivedTeammate,
    /// SERVER_GAME_SPAWN_RING: the big ring appears on the ring spawn of this number, or becomes ready.
    BigRing { ready: bool, spawn: u8 },
    /// SERVER_PLAYER_ESCAPED: this player got out through the big ring.
    Escaped,
    /// SERVER_GAME_PLAYER_ESCAPED: someone else did.
    OtherEscaped(u16),
    /// RESULTS_DATA: one player's row of the results.
    ResultsRow(crate::client::screens::ResultsRow),
}

pub struct NetClient {
    host: Option<enet::Host<UdpSocket>>,
    server: Option<enet::PeerID>,
    /// IServer: the server this game started, reached through memory instead of ENet
    /// (local_link). Set before connect; this game is its player 0.
    pub local: Option<crate::local_link::GameEnd>,
    pub ip: String,
    pub port: u16,
    /// want_lobby: the lobby the server redirected to, -1 for the default one.
    pub wanted_lobby: i32,
    pub is_connected: bool,
    pub is_ready: bool,
    pub is_server_ready: bool,
    /// nid: this client's player id.
    pub id: Option<u16>,
    pub state: NetState,
    pub players: BTreeMap<u16, Player>,
    /// exeId: the round's EXE players; the first is the one the round rules know.
    pub exe_ids: Vec<u16>,
    /// chance: this client's percent chance to be EXE.
    pub exe_chance: u8,
    /// lvlId: the map of the round.
    pub level: Option<u16>,
    /// The server lets any number of players be the same character.
    pub duplicate_characters_allowed: bool,
    /// How many EXE a round has; with more than one, EXE characters are claimed like survivors.
    pub exe_count: u8,
    /// The server lets this player pick their character ahead, on the preference cards
    /// when getting ready (lobby_preselection); without it getting ready is just ready.
    pub preselects: bool,
    /// The server runs the character select stage this round (it sent its clock).
    pub character_stage: bool,
    /// global.character and global.exeCharacter: what this client plays this round, by
    /// its side (packet::NO_CHARACTER while unknown), and whether that side is EXE.
    pub character: u8,
    pub is_exe: bool,
    /// Sandbox only (the page has a button for each): the survivor is played
    /// demonized, or the level is watched with a free camera and nobody to play.
    pub demonized: bool,
    pub freecam: bool,
    /// This client watches the round and plays no part in it (states::spectate):
    /// it was not ready, or chose the freecam card, when the lobby started the
    /// round, or it came in after the lobby (states.gameplay.allow_spectators).
    pub is_spectating: bool,
    /// SERVER_VOTE_MAPS: the three maps of the vote.
    pub vote_maps: [u8; 3],
    /// This is the server's ?gui window (states::host_gui), with the key it was given:
    /// it watches every round, and between rounds shows who is connected to pick from.
    pub observer_key: Option<String>,
    pub game_ends: bool,
    /// global.errorCode: set once the first error is shown, so later disconnects stay quiet.
    pub error_shown: bool,
    /// The players and gameplay numbers of the running round, as the server sent them.
    pub round_setup: Option<Arc<RoundSetup>>,
    /// The newest snapshot of the round the level has not taken yet.
    pub round_snapshot: Option<Snapshot>,
    /// The events of every snapshot received since the level last took them.
    pub round_events: Vec<(EventSource, SimEvent)>,
    /// ping: this client's round trip time in milliseconds (PING), 0 before the first.
    pub ping: u16,
    /// pings: everyone else's round trip time (SERVER_GAME_PING).
    pub pings: BTreeMap<u16, u16>,
    /// deadTimer and deadColor of downed players (SERVER_GAME_DEATHTIMER_TICK):
    /// seconds left and whether EXE is near the body.
    pub death_timers: BTreeMap<u16, (u8, bool)>,
    /// fromPallete, toPallete and palleteName of everyone else (PLAYER_PALETTE).
    pub player_skins: BTreeMap<u16, PlayerSkin>,
    /// The colors of everyone else's pet (PET_PALETTE).
    pub pet_skins: BTreeMap<u16, crate::client::unlockables::PetPalette>,
    /// obj_revival_puppet of downed players: whether teammates are reviving them
    /// (SERVER_REVIVAL_STATUS) and how far along (SERVER_REVIVAL_PROGRESS).
    pub revivals: BTreeMap<u16, (bool, f64)>,
    newest_snapshot_tick: Option<u32>,
    /// The latest keyframe (SERVER_ROUND_SNAPSHOT): its tick and encoding, which the
    /// deltas after it (SERVER_ROUND_DELTA) are taken from.
    keyframe: Option<(u32, Vec<u8>)>,
}

impl NetClient {
    pub fn new() -> NetClient {
        NetClient {
            host: None,
            local: None,
            server: None,
            ip: String::new(),
            port: DEFAULT_PORT,
            wanted_lobby: -1,
            is_connected: false,
            is_ready: false,
            is_server_ready: false,
            id: None,
            state: NetState::Pending,
            players: BTreeMap::new(),
            exe_ids: Vec::new(),
            exe_chance: 0,
            level: None,
            duplicate_characters_allowed: false,
            exe_count: 1,
            preselects: true,
            character_stage: false,
            character: NO_CHARACTER,
            is_exe: false,
            demonized: false,
            freecam: false,
            is_spectating: false,
            vote_maps: [0; 3],
            observer_key: None,
            game_ends: false,
            error_shown: false,
            round_setup: None,
            round_snapshot: None,
            round_events: Vec::new(),
            ping: 0,
            pings: BTreeMap::new(),
            death_timers: BTreeMap::new(),
            player_skins: BTreeMap::new(),
            pet_skins: BTreeMap::new(),
            revivals: BTreeMap::new(),
            newest_snapshot_tick: None,
            keyframe: None,
        }
    }

    /// net_send_palette: this player's skin, the colors it replaces and the ones drawn instead.
    pub fn send_skin(&mut self, name: &str, from: &crate::client::palette::Colors, to: &crate::client::palette::Colors) {
        let Some(id) = self.id else { return };
        for (is_from, colors) in [(true, from), (false, to)] {
            let mut packet = Packet::passthrough(PacketType::PLAYER_PALETTE);
            let _ = packet.write_u8(u8::from(is_from));
            let _ = packet.write_u16(id);
            let _ = packet.write_str(name);
            // Four bytes a color, as the original sends: red, green, blue and an opaque alpha.
            let _ = packet.write_u8((colors.len() * crate::core::palettes::BYTES_PER_SKIN_COLOR as usize) as u8);
            for &[red, green, blue] in colors {
                for channel in [red, green, blue, u8::MAX] {
                    let _ = packet.write_u8(channel);
                }
            }
            self.send_reliable(&packet);
        }
    }

    /// net_send_palette, its pet half: the colors this player's pet wears.
    pub fn send_pet_skin(&mut self, from: &crate::client::palette::Colors, to: &crate::client::palette::Colors) {
        let Some(id) = self.id else { return };
        for (is_from, colors) in [(true, from), (false, to)] {
            let mut packet = Packet::passthrough(PacketType::PET_PALETTE);
            let _ = packet.write_u8(u8::from(is_from));
            let _ = packet.write_u16(id);
            let _ = packet.write_u8((colors.len() * crate::core::palettes::BYTES_PER_SKIN_COLOR as usize) as u8);
            for &[red, green, blue] in colors {
                for channel in [red, green, blue, u8::MAX] {
                    let _ = packet.write_u8(channel);
                }
            }
            self.send_reliable(&packet);
        }
    }

    /// CLIENT_ROUND_INPUT: the buttons of the latest ticks, oldest first, the last
    /// one for `newest_tick`. Several ticks per packet, so a lost packet loses nothing.
    pub fn send_round_input(&mut self, newest_tick: u32, buttons: &[Buttons]) {
        let mut packet = Packet::new(PacketType::CLIENT_ROUND_INPUT);
        let _ = packet.write_u32(newest_tick);
        let _ = packet.write_u8(buttons.len() as u8);
        for pressed in buttons {
            let _ = packet.write_u16(pressed.0);
        }
        self.send_unreliable(&packet);
    }

    /// CLIENT_ROUND_PAUSE: this player opened or closed their pause menu.
    pub fn send_round_pause(&mut self, paused: bool) {
        let mut packet = Packet::new(PacketType::CLIENT_ROUND_PAUSE);
        let _ = packet.write_u8(u8::from(paused));
        self.send_reliable(&packet);
    }

    /// Round packets can be longer than a Packet holds, so they are read from the raw bytes.
    fn receive_round_packet(&mut self, data: &[u8]) -> bool {
        let Some(packet_type) = data.get(1).copied().and_then(PacketType::from_u8) else { return false };
        match packet_type {
            PacketType::SERVER_ROUND_SNAPSHOT => {
                let encoded = &data[2..];
                if let Some(snapshot) = Snapshot::decode(encoded) {
                    self.keyframe = Some((snapshot.tick, encoded.to_vec()));
                    self.accept_snapshot(snapshot);
                }
                true
            }
            PacketType::SERVER_ROUND_DELTA => {
                let Some(keyframe_tick) = data.get(2..6).map(|bytes| u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])) else { return true };
                // ENet delivers the keyframe first; a delta of an older one is of no use.
                let Some((_, keyframe)) = self.keyframe.as_ref().filter(|(tick, _)| *tick == keyframe_tick) else { return true };
                let snapshot = crate::core::snapshot::undelta(keyframe, &data[6..]).and_then(|encoded| Snapshot::decode(&encoded));
                if let Some(snapshot) = snapshot {
                    self.accept_snapshot(snapshot);
                }
                true
            }
            PacketType::SERVER_ROUND_CONFIG => {
                self.round_setup = RoundSetup::decode(&data[2..]).map(Arc::new);
                // The round is simulated in the server's steps, whatever rate it runs at.
                if let Some(setup) = self.round_setup.as_ref() {
                    crate::core::config::set_tick_rate(setup.tick_rate);
                }
                self.newest_snapshot_tick = None;
                self.keyframe = None;
                self.round_events.clear();
                true
            }
            _ => false,
        }
    }

    /// Unreliable: a snapshot overtaken by a newer one is of no use.
    fn accept_snapshot(&mut self, mut snapshot: Snapshot) {
        if self.newest_snapshot_tick.is_some_and(|newest| snapshot.tick <= newest) {
            return;
        }
        self.newest_snapshot_tick = Some(snapshot.tick);
        // ponytail: the events of a lost snapshot are lost (sounds, effects); resend unconfirmed events if that shows
        self.round_events.append(&mut snapshot.events);
        self.round_snapshot = Some(snapshot);
    }

    /// net_join: where to connect. Returns false when a connection is already up.
    /// `address` is a name or an IP, with ":port" when the server is not on the default
    /// port. A sibling lobby's port (SERVER_LOBBY_CHANGELOBBY) wins over the typed one.
    pub fn join(&mut self, address: &str, lobby_port: Option<u16>) -> bool {
        let (host, port) = split_port(address);
        self.ip = host.to_string();
        self.port = lobby_port.unwrap_or(port);
        self.wanted_lobby = lobby_port.map_or(-1, i32::from);
        !self.is_connected
    }

    /// A Sandbox pick: there is no server, so the level runs the round alone
    /// (see level::Level::open). `pick` is (character, is_exe, demonized).
    pub fn choose_alone(&mut self, (character, is_exe, demonized): (u8, bool, bool), freecam: bool) {
        self.character = character;
        self.is_exe = is_exe;
        self.demonized = demonized;
        self.freecam = freecam;
    }

    /// disnet_connect: starts connecting. False when the address cannot be used.
    pub fn connect(&mut self) -> bool {
        if let Some(local) = &self.local {
            return local.to_server.send(crate::local_link::ToServer::Connect).is_ok();
        }
        let Some(address) = resolve(&self.ip, self.port) else { return false };
        // The socket of the server's kind: an IPv6 one cannot be opened at all where
        // IPv6 is switched off (ipv6.disable=1), so IPv4 servers never ask for one.
        let any_port = if address.is_ipv4() { "0.0.0.0:0" } else { "[::]:0" };
        let Ok(socket) = UdpSocket::bind(any_port) else { return false };
        let settings = enet::HostSettings { peer_limit: 1, channel_limit: CHANNEL_COUNT, ..Default::default() };
        let Ok(mut host) = enet::Host::new(socket, settings) else { return false };
        let Ok(peer) = host.connect(address, CHANNEL_COUNT, 0) else { return false };
        self.server = Some(peer.id());
        self.host = Some(host);
        true
    }

    /// disnet_reset: drops the connection without telling the server.
    /// Tells the server this client is leaving and drops the connection: the server lets
    /// the player go at once instead of waiting for the connection to time out.
    pub fn leave_server(&mut self) {
        if let Some(local) = &self.local {
            let _ = local.to_server.send(crate::local_link::ToServer::Leave);
        }
        if let (Some(host), Some(server)) = (self.host.as_mut(), self.server) {
            if let Some(peer) = host.get_peer_mut(server) {
                // Sent before it returns: ENet's disconnect_now flushes it out.
                peer.disconnect_now(0);
            }
        }
        self.disconnect();
    }

    pub fn disconnect(&mut self) {
        self.local = None;
        self.host = None;
        self.server = None;
        self.is_connected = false;
    }

    pub fn send_reliable(&mut self, packet: &Packet) {
        self.send(enet::Packet::reliable(packet.data()));
    }

    pub fn send_unreliable(&mut self, packet: &Packet) {
        self.send(enet::Packet::unreliable(packet.data()));
    }

    fn send(&mut self, packet: enet::Packet) {
        if let Some(local) = &self.local {
            // A closed channel means the server stopped; its disconnect reports it.
            let _ = local.to_server.send(crate::local_link::ToServer::Packet(packet.data().to_vec()));
            return;
        }
        let (Some(host), Some(server)) = (self.host.as_mut(), self.server) else { return };
        if let Some(peer) = host.get_peer_mut(server) {
            // A failed send means the connection is going away; the disconnect event reports it.
            let _ = peer.send(CHANNEL, &packet);
        }
    }

    /// Everything ENet received since the last call, oldest first.
    fn receive(&mut self) -> Vec<enet::EventNoRef> {
        if let Some(local) = &self.local {
            return receive_local(local);
        }
        let Some(host) = self.host.as_mut() else { return Vec::new() };
        let mut events = Vec::new();
        loop {
            match host.service() {
                Ok(Some(event)) => events.push(event.no_ref()),
                Ok(None) => break,
                Err(error) => {
                    eprintln!("network error: {error}");
                    break;
                }
            }
        }
        events
    }

    /// The "connected" branch of net_poll: a fresh round of bookkeeping.
    fn on_connected(&mut self) {
        self.players.clear();
        self.is_ready = false;
        self.state = NetState::Pending;
        self.exe_ids.clear();
        self.exe_chance = 0;
        self.level = None;
        self.is_connected = true;
    }

    pub fn request_lobby_players(&mut self) {
        self.send_reliable(&Packet::new(PacketType::LOBBY_PLAYER));
    }
}

/// What the server this game started said, as the ENet events it stands for: the game
/// reads its own server as it would read any other. A server gone without a word (it
/// stopped) is a disconnect for shutting down.
fn receive_local(local: &crate::local_link::GameEnd) -> Vec<enet::EventNoRef> {
    use crate::local_link::ToGame;
    let server = enet::PeerID(0);
    let mut events: Vec<enet::EventNoRef> = local
        .from_server
        .try_iter()
        .map(|message| match message {
            ToGame::Connected => enet::EventNoRef::Connect { peer: server, data: 0 },
            ToGame::Packet(data) => enet::EventNoRef::Receive { peer: server, channel_id: CHANNEL, packet: enet::Packet::reliable(data.as_slice()) },
            ToGame::Disconnected(reason) => enet::EventNoRef::Disconnect { peer: server, data: reason },
        })
        .collect();
    if matches!(local.from_server.try_recv(), Err(std::sync::mpsc::TryRecvError::Disconnected)) {
        events.push(enet::EventNoRef::Disconnect { peer: server, data: crate::server::DisconnectReason::Shutdown as u32 });
    }
    events
}

/// obj_level Alarm_2: some time after the round's end the client returns to the
/// lobby by itself, forgetting the round's players.
pub fn leave_round(context: &mut Context) -> RoomId {
    context.audio.stop_all();
    let net = &mut context.net;
    // A spectator was never in the lobby: they go back to the waiting screen they
    // came from and wait for the next round there (states::spectate); the server's
    // ?gui window goes back to picking from who is connected.
    if net.is_spectating {
        net.is_spectating = false;
        net.state = NetState::Pending;
        net.players.clear();
        net.exe_ids.clear();
        return if net.observer_key.is_some() { RoomId::Lobby } else { RoomId::Waiting };
    }
    net.character = NO_CHARACTER;
    net.is_exe = false;
    net.players.clear();
    net.exe_ids.clear();
    net.state = NetState::Lobby;
    RoomId::Lobby
}

/// net_reset: forgets the connection and starts over at the main menu (game_restart).
/// A server this game started stops with it.
pub fn reset(context: &mut Context) -> RoomId {
    context.net.leave_server();
    context.net = NetClient::new();
    if let Some(hosted) = context.hosted.take() {
        hosted.stop();
    }
    RoomId::Menu
}

/// IServer (the original's ServerStart): starts the server's lobbies in this process,
/// with this game as player 0 of the first one, reached through memory (local_link).
/// The game then connects to it as to any server. A terminal the game was started
/// from becomes the server's, as for `--host`.
pub fn host(context: &mut Context) -> Result<(), String> {
    use std::io::IsTerminal;
    crate::terminal::init_logger();
    crate::hosting::load_settings()?;
    let (game_end, server_end) = crate::local_link::pair();
    let running = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let mut hosted = crate::hosting::start(running, Some(server_end));
    if std::io::stdin().is_terminal() {
        hosted.start_console_thread();
    }
    context.hosted = Some(hosted);
    context.net.local = Some(game_end);
    context.net.ip = "127.0.0.1".to_string();
    Ok(())
}

/// The server's address: its IPv4 one when it has one, which works everywhere, else its IPv6 one.
fn resolve(host: &str, port: u16) -> Option<SocketAddr> {
    let addresses: Vec<SocketAddr> = (host, port).to_socket_addrs().ok()?.collect();
    addresses.iter().find(|address| address.is_ipv4()).or(addresses.first()).copied()
}

/// net_poll plus obj_netclient's handling of what arrives, for one step.
pub fn update(context: &mut Context) -> Vec<Notice> {
    let mut notices = Vec::new();
    for event in context.net.receive() {
        match event {
            enet::EventNoRef::Connect { .. } => {
                // The server waits for this and answers with SERVER_IDENTITY_RESPONSE.
                send_identity(context);
                context.net.on_connected();
                let watching_room = if context.net.observer_key.is_some() { RoomId::Lobby } else { RoomId::Waiting };
                notices.push(Notice::GoTo(watching_room));
                return notices;
            }
            enet::EventNoRef::Disconnect { data, .. } => {
                // The server's ?gui window has nothing to go back to: it closes.
                if context.net.observer_key.is_some() {
                    context.quit = true;
                }
                context.net.disconnect();
                if !context.net.error_shown {
                    context.net.error_shown = true;
                    // ENet's disconnect data is a u32; the reasons fit its low byte.
                    notices.push(Notice::ShowError(u8::try_from(data).unwrap_or(u8::MAX)));
                }
                return notices;
            }
            enet::EventNoRef::Receive { packet, .. } => {
                if context.net.receive_round_packet(packet.data()) {
                    continue;
                }
                let mut packet = Packet::from_data(packet.data());
                if process_reliable(context, &mut packet, &mut notices) {
                    return notices;
                }
            }
        }
    }
    notices
}

/// net_tcpprocess. Returns true when the rest of this step's packets must wait
/// for the next room, as `return true` did in the original.
fn process_reliable(context: &mut Context, packet: &mut Packet, notices: &mut Vec<Notice>) -> bool {
    let passthrough = packet.read_u8().unwrap_or(0) != 0;
    let Some(packet_type) = packet.read_u8().and_then(PacketType::from_u8) else { return false };
    let net = &mut context.net;
    match packet_type {
        PacketType::SERVER_PLAYER_LEFT if !passthrough => {
            let id = packet.read_u16().unwrap_or(0);
            player_left(net, id, notices);
            false
        }
        PacketType::SERVER_LOBBY_COUNTDOWN if !passthrough => {
            if matches!(net.state, NetState::Lobby | NetState::CharSelect) {
                let counting = packet.read_u8().unwrap_or(0) != 0;
                let seconds = packet.read_u8().unwrap_or(0);
                notices.push(Notice::Countdown { counting, seconds });
                context.audio.play(sound::SND_CLOCK, false);
            }
            false
        }
        PacketType::SERVER_LOBBY_CHANGELOBBY => {
            // The server is full and points to a sibling lobby on another port.
            let port = packet.read_u32().unwrap_or(0);
            net.disconnect();
            let host = net.ip.clone();
            if net.join(&host, u16::try_from(port).ok()) {
                notices.push(Notice::GoTo(RoomId::Connecting));
            }
            false
        }
        PacketType::SERVER_LOBBY_EXE_CHANCE if !passthrough => {
            net.exe_chance = packet.read_u8().unwrap_or(0);
            false
        }
        PacketType::SERVER_SPECTATE_START if !passthrough => {
            // The round is already running: nothing is waited for and nothing of this
            // client is in it. Its own player stays unknown, which is what the level
            // reads to know it is watching and not playing.
            net.is_spectating = true;
            net.state = NetState::Game;
            net.is_server_ready = true;
            net.round_snapshot = None;
            net.round_events.clear();
            net.character = NO_CHARACTER;
            net.is_exe = false;
            net.ping = 0;
            net.death_timers.clear();
            net.player_skins.clear();
            net.revivals.clear();
            match net.level.and_then(|map| crate::client::levels::LEVELS.get(map as usize)) {
                Some(level) => notices.push(Notice::GoTo(level.room)),
                None => notices.push(Notice::ShowError(0)),
            }
            true
        }
        PacketType::SERVER_GAME_BACK_TO_LOBBY if !passthrough => {
            context.audio.stop_all();
            // Whatever this client was in the round that just ended, it is in the lobby now.
            net.is_spectating = false;
            net.players.clear();
            net.exe_ids.clear();
            net.game_ends = false;
            net.state = NetState::Lobby;
            net.request_lobby_players();
            notices.push(Notice::GoTo(RoomId::Lobby));
            true
        }
        _ => match net.state {
            NetState::Pending => pending_state(context, packet_type, passthrough, packet, notices),
            NetState::Lobby => lobby_state(context, packet_type, passthrough, packet, notices),
            NetState::Vote => vote_state(context, packet_type, passthrough, packet, notices),
            NetState::CharSelect => character_state(context, packet_type, passthrough, packet, notices),
            NetState::Game => game_state(context, packet_type, passthrough, packet, notices),
            NetState::Results => results_state(context, packet_type, passthrough, packet, notices),
        },
    }
}

fn player_left(net: &mut NetClient, id: u16, notices: &mut Vec<Notice>) {
    if net.state == NetState::Pending {
        notices.push(Notice::WaitingPlayerLeft(id));
    }
    if !net.players.contains_key(&id) {
        return;
    }
    if matches!(net.state, NetState::Lobby | NetState::Vote | NetState::CharSelect) {
        notices.push(Notice::PlayerLeft(id));
    }
    net.players.remove(&id);
}

/// net_state_pending: identifying, and the waiting room while a round runs.
fn pending_state(context: &mut Context, packet_type: PacketType, passthrough: bool, packet: &mut Packet, notices: &mut Vec<Notice>) -> bool {
    let net = &mut context.net;
    match packet_type {
        PacketType::SERVER_IDENTITY_RESPONSE if !passthrough => {
            let lobby_is_open = packet.read_u8().unwrap_or(0) != 0;
            net.id = packet.read_u16();
            net.duplicate_characters_allowed = packet.read_u8().unwrap_or(0) != 0;
            net.exe_count = packet.read_u8().unwrap_or(1);
            net.preselects = packet.read_u8().unwrap_or(1) != 0;
            if !lobby_is_open {
                context.audio.play_music(sound::MUS_WAITING);
                notices.push(Notice::WaitForRound);
                return false;
            }
            net.is_ready = true;
            net.state = NetState::Lobby;
            notices.push(Notice::GoTo(RoomId::Lobby));
            net.request_lobby_players();
            return true;
        }
        // Who is exe and on which map: only a client that watches the round is told
        // while waiting (states.gameplay.allow_spectators). It goes on to the
        // character select stage (SERVER_CHAR_TIME_SYNC, the waiting screen) or into
        // the round (SERVER_SPECTATE_START), which both need it.
        PacketType::SERVER_LOBBY_EXE if !passthrough => {
            net.is_spectating = true;
            exe_chosen(context, packet, notices);
        }
        PacketType::SERVER_VOTE_MAPS if !passthrough => return vote_started(net, packet, notices),
        PacketType::LOBBY_PLAYER if !passthrough => {
            let id = packet.read_u16().unwrap_or(0);
            let _ready = packet.read_u8();
            let nickname = packet.read_str().unwrap_or_default();
            let icon = packet.read_u8().unwrap_or(0);
            net.players.insert(id, Player::new(nickname, icon, false));
        }
        PacketType::SERVER_GAME_TIME_SYNC if !passthrough => {
            notices.push(Notice::RoundTime(packet.read_u16().unwrap_or(0)));
        }
        PacketType::SERVER_WAITING_PLAYER_INFO if !passthrough => {
            let in_round = packet.read_u8().unwrap_or(0) != 0;
            let id = packet.read_u16().unwrap_or(0);
            let nickname = packet.read_str().unwrap_or_default();
            let (exe, character, icon) = if in_round {
                let exe = packet.read_u8().unwrap_or(0) != 0;
                (exe, packet.read_u8().unwrap_or(NO_CHARACTER), 0)
            } else {
                (false, NO_CHARACTER, packet.read_u8().unwrap_or(0))
            };
            notices.push(Notice::WaitingPlayer { id, nickname, in_round, exe, character, icon });
            if !context.audio.is_playing(sound::SND_RING) {
                context.audio.play(sound::SND_RING, false);
            }
        }
        PacketType::CHAT_MESSAGE if !passthrough => chat_message(net, packet, notices),
        // obj_menu_waitkick: the waiting room picks for operators only, never for a vote.
        PacketType::LOBBY_CHOOSE_PLAYER => {
            let action = packet.read_u8().unwrap_or(CHOOSE_VOTEKICK);
            if action != CHOOSE_VOTEKICK {
                notices.push(Notice::ChoosePlayer(action));
            }
        }
        PacketType::SERVER_RESULTS => return show_results(net, packet, notices),
        _ => {}
    }
    false
}

/// PLAYER_PALETTE: half of a player's skin, the colors it replaces or the ones drawn instead.
fn read_skin(net: &mut NetClient, packet: &mut Packet) {
    let (Some(is_from), Some(id), Some(name), Some(size)) = (packet.read_u8(), packet.read_u16(), packet.read_str(), packet.read_u8()) else { return };
    let channels: Option<Vec<u8>> = (0..size).map(|_| packet.read_u8()).collect();
    let Some(channels) = channels else { return };
    // Skins are opaque: the alpha byte of each color is not needed.
    let colors: crate::client::palette::Colors = channels.chunks_exact(4).map(|color| [color[0], color[1], color[2]]).collect();
    let skin = net.player_skins.entry(id).or_insert_with(|| PlayerSkin {
        name: String::new(),
        from: crate::client::palette::default_colors(),
        to: crate::client::palette::default_colors(),
    });
    skin.name = name;
    if is_from != 0 {
        skin.from = colors;
    } else {
        skin.to = colors;
    }
}

/// PET_PALETTE: half of another player's pet colors, as read_skin reads a skin.
fn read_pet_skin(net: &mut NetClient, packet: &mut Packet) {
    let (Some(is_from), Some(id), Some(size)) = (packet.read_u8(), packet.read_u16(), packet.read_u8()) else { return };
    let channels: Option<Vec<u8>> = (0..size).map(|_| packet.read_u8()).collect();
    let Some(channels) = channels else { return };
    let colors: crate::client::palette::Colors = channels.chunks_exact(4).map(|color| [color[0], color[1], color[2]]).collect();
    let pet = net.pet_skins.entry(id).or_insert_with(|| crate::client::unlockables::PetPalette { from: Vec::new(), to: Vec::new() });
    if is_from != 0 {
        pet.from = colors;
    } else {
        pet.to = colors;
    }
}

/// SERVER_RESULTS, in the waiting room and in the round: on to room_results.
fn show_results(net: &mut NetClient, packet: &mut Packet, notices: &mut Vec<Notice>) -> bool {
    // The server's ?gui window shows no results: it goes back to picking players.
    if net.observer_key.is_some() {
        net.is_spectating = false;
        net.state = NetState::Pending;
        notices.push(Notice::GoTo(RoomId::Lobby));
        return true;
    }
    net.level = packet.read_u8().map(u16::from);
    net.is_ready = true;
    net.game_ends = true;
    net.state = NetState::Results;
    notices.push(Notice::GoTo(RoomId::Results));
    true
}

/// net_state_game and the game packets of net_udpprocess.
// ponytail: the round start, clock, pings and death timers so far; endings, rings and map objects come with the round rules
fn game_state(context: &mut Context, packet_type: PacketType, passthrough: bool, packet: &mut Packet, notices: &mut Vec<Notice>) -> bool {
    let net = &mut context.net;
    match packet_type {
        // Relayed from another client, as the original did.
        PacketType::PLAYER_PALETTE => read_skin(net, packet),
        PacketType::PET_PALETTE => read_pet_skin(net, packet),
        // The in-round chat of the pause menu.
        PacketType::CHAT_MESSAGE => chat_message(net, packet, notices),
        _ if passthrough => {}
        PacketType::SERVER_GAME_PLAYERS_READY => net.is_server_ready = true,
        PacketType::SERVER_GAME_TIME_SYNC => notices.push(Notice::RoundTime(packet.read_u16().unwrap_or(0))),
        PacketType::SERVER_GAME_SUDDEN_DEATH => notices.push(Notice::SuddenDeath),
        PacketType::PONG => net.ping = packet.read_u16().unwrap_or(0),
        PacketType::SERVER_GAME_PING => {
            let (id, milliseconds) = (packet.read_u16().unwrap_or(0), packet.read_u16().unwrap_or(0));
            net.pings.insert(id, milliseconds);
        }
        PacketType::SERVER_GAME_EXE_WINS | PacketType::SERVER_GAME_SURVIVOR_WIN | PacketType::SERVER_GAME_TIME_OVER => {
            net.game_ends = true;
            let ending = match packet_type {
                PacketType::SERVER_GAME_EXE_WINS => crate::core::game::RoundEnding::ExeWon,
                PacketType::SERVER_GAME_SURVIVOR_WIN => crate::core::game::RoundEnding::SurvivorsEscaped,
                _ => crate::core::game::RoundEnding::TimeOver,
            };
            let counts = packet.read_u8().unwrap_or(0) != 0;
            notices.push(Notice::RoundEnded(ending, counts));
        }
        PacketType::SERVER_RESULTS => return show_results(net, packet, notices),
        PacketType::SERVER_GAME_SPAWN_RING if !net.game_ends => {
            let ready = packet.read_u8().unwrap_or(0) != 0;
            notices.push(Notice::BigRing { ready, spawn: packet.read_u8().unwrap_or(0) });
        }
        PacketType::SERVER_PLAYER_ESCAPED => notices.push(Notice::Escaped),
        PacketType::SERVER_REVIVAL_RINGSUB => notices.push(Notice::RevivedTeammate),
        PacketType::SERVER_GAME_PLAYER_ESCAPED => notices.push(Notice::OtherEscaped(packet.read_u16().unwrap_or(0))),
        PacketType::SERVER_REVIVAL_STATUS => {
            let shown = packet.read_u8().unwrap_or(0) != 0;
            let id = packet.read_u16().unwrap_or(0);
            net.revivals.entry(id).or_insert((false, 0.0)).0 = shown;
        }
        PacketType::SERVER_REVIVAL_PROGRESS => {
            let id = packet.read_u16().unwrap_or(0);
            let progress = packet.read_f64().unwrap_or(0.0);
            net.revivals.entry(id).or_insert((false, 0.0)).1 = progress;
        }
        PacketType::SERVER_GAME_DEATHTIMER_TICK => {
            let exe_near = packet.read_u8().unwrap_or(0) != 0;
            let id = packet.read_u16().unwrap_or(0);
            let seconds = packet.read_u8().unwrap_or(0);
            net.death_timers.insert(id, (seconds, exe_near));
        }
        _ => {}
    }
    false
}

/// net_state_lobby: players, readiness, chat, and the start of the vote.
fn lobby_state(context: &mut Context, packet_type: PacketType, passthrough: bool, packet: &mut Packet, notices: &mut Vec<Notice>) -> bool {
    if passthrough && !matches!(packet_type, PacketType::SERVER_LOBBY_CORRECT | PacketType::LOBBY_CHOOSE_PLAYER) {
        return false;
    }
    let net = &mut context.net;
    match packet_type {
        PacketType::SERVER_LOBBY_EXE => exe_chosen(context, packet, notices),
        PacketType::SERVER_PLAYER_JOINED => {
            let id = packet.read_u16().unwrap_or(0);
            let nickname = packet.read_str().unwrap_or_default();
            let icon = packet.read_u8().unwrap_or(0);
            let pet = packet.read_u8().unwrap_or(crate::client::unlockables::NO_PET);
            net.players.insert(id, Player { pet, ..Player::new(nickname, icon, false) });
            notices.push(Notice::PlayerJoined { id, play_sound: true });
        }
        PacketType::LOBBY_READY_STATE => {
            let id = packet.read_u16().unwrap_or(0);
            let ready = packet.read_u8().unwrap_or(0) != 0;
            let preferences = read_preferences(packet);
            if let Some(player) = net.players.get_mut(&id) {
                player.is_ready = ready;
                (player.preferred_exe, player.preferred_survivor) = preferences;
                context.audio.play(sound::SND_READY, false);
            }
        }
        PacketType::LOBBY_CHARACTER => {
            let survivor = packet.read_u8().unwrap_or(0);
            if packet.read_u8().unwrap_or(0) == 0 {
                notices.push(Notice::SurvivorTaken(survivor));
            }
        }
        PacketType::LOBBY_EXECHARACTER => {
            let exe = packet.read_u8().unwrap_or(0);
            if packet.read_u8() == Some(0) {
                notices.push(Notice::ExeTaken(exe + 1));
            }
        }
        PacketType::SERVER_LOBBY_CORRECT => notices.push(Notice::JoinedLobby),
        PacketType::LOBBY_PLAYER => {
            let id = packet.read_u16().unwrap_or(0);
            let ready = packet.read_u8().unwrap_or(0) != 0;
            let nickname = packet.read_str().unwrap_or_default();
            let icon = packet.read_u8().unwrap_or(0);
            let pet = packet.read_u8().unwrap_or(crate::client::unlockables::NO_PET);
            let mut player = Player { pet, ..Player::new(nickname, icon, ready) };
            (player.preferred_exe, player.preferred_survivor) = read_preferences(packet);
            net.players.insert(id, player);
            notices.push(Notice::PlayerJoined { id, play_sound: false });
        }
        PacketType::CHAT_MESSAGE => chat_message(net, packet, notices),
        PacketType::SERVER_VOTE_MAPS => return vote_started(net, packet, notices),
        PacketType::LOBBY_CHOOSE_PLAYER => notices.push(Notice::ChoosePlayer(packet.read_u8().unwrap_or(0))),
        _ => {}
    }
    false
}

/// The results of a round, and, between the rounds of a tournament
/// (states.gameplay.tournament_mode), straight on to the next one's vote or, without
/// a vote, its character select stage, both on the lobby screen.
fn results_state(context: &mut Context, packet_type: PacketType, passthrough: bool, packet: &mut Packet, notices: &mut Vec<Notice>) -> bool {
    if passthrough {
        return false;
    }
    match packet_type {
        PacketType::RESULTS_DATA => {
            if let Some(row) = crate::client::screens::ResultsRow::read(packet) {
                notices.push(Notice::ResultsRow(row));
            }
        }
        PacketType::SERVER_VOTE_MAPS => return vote_started(&mut context.net, packet, notices),
        PacketType::SERVER_LOBBY_EXE => {
            exe_chosen(context, packet, notices);
            notices.push(Notice::GoTo(RoomId::Lobby));
            return true;
        }
        _ => {}
    }
    false
}

/// SERVER_VOTE_MAPS: the map vote starts. A client not on the lobby screen (it
/// comes in while the vote runs, or shows the results of a tournament's round)
/// goes there, and the rest of the vote's packets wait for it.
fn vote_started(net: &mut NetClient, packet: &mut Packet, notices: &mut Vec<Notice>) -> bool {
    net.vote_maps = [0; 3].map(|_| packet.read_u8().unwrap_or(0));
    let came_from = net.state;
    net.state = NetState::Vote;
    net.game_ends = false;
    notices.push(Notice::VoteMaps);
    if came_from == NetState::Lobby {
        return false;
    }
    // Only a client that watches the round is shown its vote while waiting.
    if came_from == NetState::Pending {
        net.is_spectating = true;
    }
    notices.push(Notice::GoTo(RoomId::Lobby));
    true
}

/// net_state_vote
fn vote_state(context: &mut Context, packet_type: PacketType, passthrough: bool, packet: &mut Packet, notices: &mut Vec<Notice>) -> bool {
    if passthrough {
        return false;
    }
    match packet_type {
        PacketType::SERVER_LOBBY_EXE => exe_chosen(context, packet, notices),
        PacketType::SERVER_VOTE_TIME_SYNC => notices.push(Notice::VoteTime(packet.read_u8().unwrap_or(0))),
        PacketType::VOTE => notices.push(Notice::VoteCounts([0; 3].map(|_| packet.read_u8().unwrap_or(0)))),
        _ => {}
    }
    false
}

/// net_state_charselect: everyone picks their character, or the server hands them
/// out, and the round starts.
fn character_state(context: &mut Context, packet_type: PacketType, passthrough: bool, packet: &mut Packet, notices: &mut Vec<Notice>) -> bool {
    if passthrough {
        return false;
    }
    let net = &mut context.net;
    match packet_type {
        PacketType::LOBBY_CHARACTER | PacketType::LOBBY_EXECHARACTER => {
            let character = packet.read_u8().unwrap_or(NO_CHARACTER);
            if packet.read_u8().unwrap_or(0) != 0 {
                net.character = character;
                net.is_exe = packet_type == PacketType::LOBBY_EXECHARACTER;
                context.audio.play(sound::SND_CLOCK, false);
            } else {
                context.audio.play(sound::SND_NONO, false);
            }
        }
        PacketType::SERVER_CHAR_TIME_SYNC => {
            net.character_stage = true;
            notices.push(Notice::CharacterTime(packet.read_u8().unwrap_or(0)));
        }
        PacketType::SERVER_LOBBY_CHARACTER_CHANGE => {
            let id = packet.read_u16().unwrap_or(0);
            let character = packet.read_u8().unwrap_or(NO_CHARACTER);
            let Some(player) = net.players.get_mut(&id) else { return false };
            player.character = character;
            // Only on the stage: without it every character arrives at once, and one
            // clock per player would stack into noise.
            if net.character_stage {
                context.audio.play(sound::SND_CLOCK, false);
            }
        }
        PacketType::SERVER_LOBBY_GAME_START => {
            net.state = NetState::Game;
            net.is_server_ready = false;
            net.round_snapshot = None;
            net.round_events.clear();
            net.ping = 0;
            net.death_timers.clear();
            net.player_skins.clear();
            net.revivals.clear();
            let exe_character = if net.is_exe {
                net.character
            } else {
                net.exe_ids.first().and_then(|exe| net.players.get(exe)).map_or(NO_CHARACTER, |player| player.character)
            };
            let laugh = match exe_character {
                1 => Some(sound::SND_EXE_LAUGH),
                2 => Some(sound::SND_CHAOS_LAUGH),
                3 => Some(sound::SND_EXETIOR_LAUGH),
                4 => Some(sound::SND_EXELLER_LAUGH),
                _ => None,
            };
            if let Some(laugh) = laugh {
                context.audio.play(laugh, false);
            }
            notices.push(Notice::RoundStarts);
        }
        _ => {}
    }
    false
}

/// The Sandbox page's random pick, (character, is_exe, demonized): any survivor,
/// as is or demonized, or any EXE.
pub fn random_alone_pick() -> (u8, bool, bool) {
    let picks: Vec<(u8, bool, bool)> = (1..=SURVIVOR_COUNT)
        .flat_map(|survivor| [(survivor, false, false), (survivor, false, true)])
        .chain((1..=EXE_COUNT).map(|exe| (exe, true, false)))
        .collect();
    picks[macroquad::rand::gen_range(0, picks.len())]
}

/// A player's skin as PLAYER_PALETTE told it: name and colors.
#[derive(Clone, Debug, PartialEq)]
pub struct PlayerSkin {
    pub name: String,
    pub from: crate::client::palette::Colors,
    pub to: crate::client::palette::Colors,
}

/// The preference cards appended to a lobby player's packets.
fn read_preferences(packet: &mut Packet) -> (u8, u8) {
    (packet.read_u8().unwrap_or(0), packet.read_u8().unwrap_or(0))
}

/// SERVER_LOBBY_EXE, the same in the lobby and the vote: who is EXE and which map.
fn exe_chosen(context: &mut Context, packet: &mut Packet, notices: &mut Vec<Notice>) {
    let net = &mut context.net;
    let main_exe = packet.read_u16().unwrap_or(0);
    let map = packet.read_u16().unwrap_or(0);
    let other_exe_count = packet.read_u8().unwrap_or(0);
    net.exe_ids = std::iter::once(main_exe).chain((0..other_exe_count).filter_map(|_| packet.read_u16())).collect();
    net.level = Some(map);
    net.character = NO_CHARACTER;
    net.character_stage = false;
    net.is_exe = net.id.is_some_and(|id| net.exe_ids.contains(&id));
    // The characters of the last round are no one's in this one.
    for player in net.players.values_mut() {
        player.character = NO_CHARACTER;
    }
    net.state = NetState::CharSelect;
    notices.push(Notice::ExeChosen { map });
}

fn chat_message(net: &NetClient, packet: &mut Packet, notices: &mut Vec<Notice>) {
    let id = packet.read_u16().unwrap_or(0);
    let text = packet.read_str().unwrap_or_default();
    let sender = if id == 0 {
        "(server)".to_string()
    } else if let Some(player) = net.players.get(&id) {
        player.nickname.clone()
    } else {
        return;
    };
    notices.push(Notice::ChatMessage { sender, text });
}

impl Player {
    pub fn new(nickname: String, icon: u8, is_ready: bool) -> Player {
        Player { nickname, icon, is_ready, preferred_exe: ANY_CHARACTER, preferred_survivor: ANY_CHARACTER, character: NO_CHARACTER, pet: crate::client::unlockables::NO_PET }
    }
}

/// "host:port" or "[ipv6]:port" as (host, port); without a port, the default one.
pub fn split_port(address: &str) -> (&str, u16) {
    // "[2001:db8::1]:7606": an IPv6 address with a port is written in brackets.
    if let Some((host, rest)) = address.strip_prefix('[').and_then(|inner| inner.split_once(']')) {
        return (host, rest.strip_prefix(':').and_then(|port| port.parse().ok()).unwrap_or(DEFAULT_PORT));
    }
    // More than one colon is a bare IPv6 address, whose last group is no port.
    if address.matches(':').count() > 1 {
        return (address, DEFAULT_PORT);
    }
    match address.rsplit_once(':').map(|(host, port)| (host, port.parse::<u16>())) {
        Some((host, Ok(port))) => (host, port),
        _ => (address, DEFAULT_PORT),
    }
}

/// net_identity: who this client is.
fn send_identity(context: &mut Context) {
    let mut packet = Packet::new(PacketType::IDENTITY);
    let _ = packet.write_char(VERSION);
    let _ = packet.write_u32(context.net.wanted_lobby as u32);
    let _ = packet.write_str(&context.options.nickname);
    let _ = packet.write_str(&crate::client::device::device_id());
    let _ = packet.write_u8(context.unlockables.lobby_icon);
    let _ = packet.write_u8(context.unlockables.pet);
    if let Some(key) = &context.net.observer_key {
        let _ = packet.write_str(key);
    }
    context.net.send_reliable(&packet);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_port_is_split_off_ipv4_names_and_bracketed_ipv6_but_not_bare_ipv6() {
        assert_eq!(split_port("1.2.3.4:7000"), ("1.2.3.4", 7000));
        assert_eq!(split_port("example.org"), ("example.org", DEFAULT_PORT));
        assert_eq!(split_port("[2001:db8::1]:7000"), ("2001:db8::1", 7000));
        assert_eq!(split_port("[2001:db8::1]"), ("2001:db8::1", DEFAULT_PORT));
        assert_eq!(split_port("2001:db8::1"), ("2001:db8::1", DEFAULT_PORT), "its last group is no port");
    }

    #[test]
    fn another_players_pet_colors_are_kept_by_their_id() {
        let mut net = NetClient::new();
        for (is_from, color) in [(1u8, [10u8, 20, 30]), (0, [40, 50, 60])] {
            let mut packet = Packet::passthrough(PacketType::PET_PALETTE);
            let _ = packet.write_u8(is_from);
            let _ = packet.write_u16(7);
            let _ = packet.write_u8(4);
            for channel in [color[0], color[1], color[2], u8::MAX] {
                let _ = packet.write_u8(channel);
            }
            packet.pos = 2;
            read_pet_skin(&mut net, &mut packet);
        }
        let pet = &net.pet_skins[&7];
        assert_eq!((pet.from.as_slice(), pet.to.as_slice()), (&[[10, 20, 30]][..], &[[40, 50, 60]][..]));
    }

    #[test]
    fn an_address_may_carry_its_port() {
        let mut net = NetClient::new();
        net.join("example.org:7607", None);
        assert_eq!((net.ip.as_str(), net.port, net.wanted_lobby), ("example.org", 7607, -1));
        net.join("example.org", None);
        assert_eq!((net.ip.as_str(), net.port), ("example.org", DEFAULT_PORT));
        // A sibling lobby's port wins, and is the lobby asked for.
        net.join("example.org:7607", Some(7700));
        assert_eq!((net.port, net.wanted_lobby), (7700, 7700));
    }
}
