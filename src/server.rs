use std::net::UdpSocket;
use std::sync::RwLock;
use std::time::Instant;

use rusty_enet as enet;

use crate::local_link::{ServerEnd, ToGame, HOST_SLOT};
use crate::packet::{Packet as GamePacket, PacketType};
use crate::player::Player;
use crate::vote::Vote;

/// Why the server ended a connection: the client shows its words (screens/message.rs).
/// It travels in ENet's disconnect data, a u32, of which only the low byte is used.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisconnectReason {
    Other           = 255,
    KickedByHost    = 1,
    BannedByHost    = 2,
    VersionMismatch = 3,
    ServerTimeout   = 4,
    PacketsNotRecv  = 5,
    AfkTimeout      = 7,
    LobbyFull       = 8,
    RateLimited     = 9,
    Shutdown        = 10,
    IpInUse         = 11,
}

pub enum OutboxMsg {
    Broadcast(Vec<u8>, bool),
    BroadcastEx(Vec<u8>, bool, u16),
    SendTo(u16, Vec<u8>, bool),
    Disconnect(u16, DisconnectReason),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum BigRingState {
    None        = 0,
    Deactivated = 1,
    Activated   = 2,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ending {
    ExeWin  = 0,
    SurvWin = 1,
    TimeOver = 2,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameState {
    Lobby,
    MapVote,
    CharSelect,
    Game,
    Results,
}

/// How lobby `id` (counted from 0 inside the server) is named in the logs and the
/// console: "[Lobby 1]" for the first, as the terminal and the .lobby command count.
pub fn lobby_tag(id: u16) -> String {
    format!("[Lobby {}]", id + 1)
}

pub struct PeerData {
    pub id: u16,
    pub ip: String,
    pub plr: Player,
    pub nickname: String,
    pub udid: String,
    pub lobby_icon: u8,
    /// Their pet's number (from 1), 0 for none: the server only passes it on.
    pub pet: u8,
    pub verified: bool,
    pub in_game: bool,
    pub op: u8,
    pub ready: bool,
    pub can_vote: bool,
    pub voted: bool,
    pub disconnecting: bool,
    /// The character they play this round, by their side (packet::NO_CHARACTER and on).
    pub character: u8,
    /// The characters this player's preference cards asked for when getting ready
    /// (packet::ANY_CHARACTER for no wish). The round gives them these instead of a
    /// character select stage.
    pub preferred_exe: u8,
    pub preferred_survivor: u8,
    /// The EXE card says "not me": drawn as EXE only when nobody else is left.
    pub refuses_exe: bool,
    pub should_timeout: bool,
    pub exe_chance: u8,
    pub timeout: f64,
    pub vote_cooldown: f64,
    pub rtt: u16,
    /// Whether this peer is watching the round instead of playing it: from the
    /// waiting room (.spectate), or from the lobby with the freecam card or without
    /// getting ready. See states::spectate.
    pub is_spectating: bool,
    // Per-peer chat token bucket (anti-flood). Refilled on demand from
    // wall-clock time, so no per-tick bookkeeping is needed.
    pub chat_tokens: f64,
    pub chat_last: Instant,
}

impl PeerData {
    /// Consume one chat token. `burst`/`refill_per_sec` come from config
    /// (`states.lobby_misc.chat_rate_limit`). Returns true if the message is allowed,
    /// false if the peer is currently rate-limited and the message should be dropped.
    pub fn chat_token_take(&mut self, burst: f64, refill_per_sec: f64) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.chat_last).as_secs_f64();
        self.chat_last = now;
        self.chat_tokens = (self.chat_tokens + elapsed * refill_per_sec).min(burst);
        if self.chat_tokens >= 1.0 {
            self.chat_tokens -= 1.0;
            true
        } else {
            false
        }
    }

    /// Whether this peer's client is following the round: playing it, or
    /// watching it from the waiting room. Every stage that addresses "the
    /// round" rather than "everyone connected" asks this, so a spectator is
    /// shown the same screens the round's players are shown.
    pub fn follows_round(&self) -> bool {
        self.in_game || self.is_spectating
    }

    /// Ready with the survivor card saying "freecam": this peer will watch the
    /// round the lobby starts, not play it (states::spectate).
    pub fn chose_freecam(&self) -> bool {
        self.ready && self.preferred_survivor == crate::packet::NO_CHARACTER
    }

    pub fn new(id: u16, ip: String) -> Self {
        Self {
            id,
            ip,
            plr: Player::default(),
            nickname: String::new(),
            udid: String::new(),
            lobby_icon: 0,
            pet: 0,
            verified: false,
            in_game: false,
            op: 0,
            ready: false,
            can_vote: true,
            voted: false,
            disconnecting: false,
            character: crate::packet::NO_CHARACTER,
            preferred_exe: crate::packet::ANY_CHARACTER,
            preferred_survivor: crate::packet::ANY_CHARACTER,
            refuses_exe: false,
            should_timeout: false,
            exe_chance: 1,
            timeout: 0.0,
            vote_cooldown: 0.0,
            rtt: 0,
            is_spectating: false,
            // Start with a full bucket so a fresh peer can chat immediately; the cap is
            // re-applied from config on the first take.
            chat_tokens: crate::config::cfg().states.lobby_misc.chat_rate_limit.burst,
            chat_last: Instant::now(),
        }
    }
}

pub struct LobbyData {
    pub countdown: f64,
    pub prac_countdown: f64,
    pub countdown_sec: u8,
    pub vote: Vote,
    pub kick_target: Option<PeerData>,
    /// The map a vote to change it is about, while one runs.
    pub voting_map: Option<u8>,
    pub maps: [u8; 3],
    pub votes: [u8; 3],
    /// The map the next round is on, once chosen.
    pub map: Option<u8>,
    /// The round's EXE players, in the order they were drawn.
    pub exes: Vec<u16>,
    // legacy (v1.0.0 C#) votekick state, used only when
    // states.lobby_misc.votekick.use_legacy_voting is enabled
    pub legacy_votekick_ongoing: bool,
    pub legacy_votekick_target: Option<u16>,
    pub legacy_votekick_timer: f64,
    pub legacy_votekick_votes: Vec<u16>,
    // legacy (v1.0.0 C#) votepractice state
    pub legacy_practice_ongoing: bool,
    pub legacy_practice_votes: Vec<u16>,
}

impl LobbyData {
    /// The EXE that the round rules (states::game) know:
    /// they were written for one EXE per round.
    pub fn main_exe(&self) -> u16 {
        self.exes.first().copied().unwrap_or(0)
    }

    pub fn is_exe(&self, player_id: u16) -> bool {
        self.exes.contains(&player_id)
    }
}

impl Default for LobbyData {
    fn default() -> Self {
        Self {
            countdown: 0.0,
            prac_countdown: 0.0,
            countdown_sec: 0,
            vote: Vote::default(),
            kick_target: None,
            voting_map: None,
            maps: [0; 3],
            votes: [0; 3],
            map: None,
            exes: Vec::new(),
            legacy_votekick_ongoing: false,
            legacy_votekick_target: None,
            legacy_votekick_timer: 0.0,
            legacy_votekick_votes: Vec::new(),
            legacy_practice_ongoing: false,
            legacy_practice_votes: Vec::new(),
        }
    }
}

pub struct GameData {
    pub exe: i32,
    pub map: u8,
    pub started: bool,
    pub sudden_death: bool,
    pub start_timeout: f64,
    pub time: f64,
    pub elapsed: f64,
    pub time_sec: u16,
    pub end: f64,
    pub ending: Ending,
    pub bring_state: BigRingState,
    pub bring_loc: u8,
    /// Ravine Mist: shards the simulation says are no longer on the map, which the map's
    /// rules open the big ring by (states::round::sync_level_to_rules).
    pub shards_found: u8,
    pub next_entity_id: u16,
    pub ring_coff: u8,
    pub left: Vec<PeerData>,
    /// The simulation of the authoritative round being played.
    pub round: Option<crate::states::round::Round>,
}

impl Default for GameData {
    fn default() -> Self {
        Self {
            exe: -1,
            map: 0,
            started: false,
            sudden_death: false,
            start_timeout: 0.0,
            time: 0.0,
            elapsed: 0.0,
            time_sec: 0,
            end: 0.0,
            ending: Ending::TimeOver,
            bring_state: BigRingState::None,
            shards_found: 0,
            bring_loc: 0,
            next_entity_id: 0,
            ring_coff: 5,
            left: Vec::new(),
            round: None,
        }
    }
}

pub struct ResultsData {
    pub countdown: f64,
}

impl Default for ResultsData {
    fn default() -> Self {
        Self { countdown: 0.0 }
    }
}

pub struct Server {
    pub id: u16,
    pub running: bool,
    pub state: GameState,
    pub lobby: LobbyData,
    pub game: GameData,
    pub results: ResultsData,
    pub last_map: Option<u8>,
    pub map_pickrates: [i16; 30],
    pub delta: f64,
    pub peers: Vec<PeerData>,
    /// The host's ?gui windows connected to this lobby (states::host_gui): they watch
    /// everything and are no player at all, so they are not among `peers`.
    pub observers: Vec<u16>,
}

impl Server {
    pub fn new(id: u16) -> Self {
        Self {
            id,
            running: true,
            state: GameState::Lobby,
            lobby: LobbyData::default(),
            game: GameData::default(),
            results: ResultsData::default(),
            last_map: None,
            map_pickrates: [255i16; 30],
            delta: 1.0 / crate::core::config::ticks_per_second(),
            peers: Vec::new(),
            observers: Vec::new(),
        }
    }

    pub fn ingame_count(&self) -> usize {
        self.peers.iter().filter(|peer| peer.in_game).count()
    }

    pub fn total_count(&self) -> usize {
        self.peers.len()
    }

    pub fn find_peer(&self, id: u16) -> Option<&PeerData> {
        self.peers.iter().find(|peer| peer.id == id)
    }

    pub fn find_peer_mut(&mut self, id: u16) -> Option<&mut PeerData> {
        self.peers.iter_mut().find(|peer| peer.id == id)
    }

    /// Whether `id` is allowed to send a chat message right now. Operators at or
    /// above the configured `exempt_op_level` bypass the limit so moderation messaging is
    /// never throttled. When the limit is disabled this is a no-op (always allows).
    pub fn chat_rate_allow(&mut self, id: u16) -> bool {
        let rate_limit = &crate::config::cfg().states.lobby_misc.chat_rate_limit;
        if !rate_limit.enable {
            return true;
        }
        let (burst, refill, exempt) = (rate_limit.burst, rate_limit.messages_per_second, rate_limit.exempt_op_level);
        match self.find_peer_mut(id) {
            Some(peer) => peer.op >= exempt || peer.chat_token_take(burst, refill),
            None => false,
        }
    }
}

pub struct PendingPeer {
    pub ip: String,
    pub timeout: f64,
}

pub struct PeerSummary {
    pub id: u16,
    pub ip: String,
    pub udid: String,
    pub nickname: String,
    pub op: u8,
    pub in_game: bool,
}

pub struct ServerShared {
    pub peers: RwLock<Vec<PeerSummary>>,
}

impl ServerShared {
    pub fn new() -> Self {
        Self {
            peers: RwLock::new(Vec::new()),
        }
    }
}

pub fn send_chat(outbox: &mut Vec<OutboxMsg>, target: u16, msg: &str) {
    let mut pkt = GamePacket::new(PacketType::CHAT_MESSAGE);
    let _ = pkt.write_u16(0);
    let _ = pkt.write_str(&msg.to_lowercase());
    outbox.push(OutboxMsg::SendTo(target, pkt.data().to_vec(), true));
}

/// Sends `packet` to everyone shown the round's screens (PeerData::follows_round):
/// its players and its spectators, but not the waiting screen, which shows none.
pub fn send_to_round(server: &Server, outbox: &mut Vec<OutboxMsg>, packet: &GamePacket) {
    for peer in server.peers.iter().filter(|peer| peer.follows_round()) {
        outbox.push(OutboxMsg::SendTo(peer.id, packet.data().to_vec(), true));
    }
}

/// A player's chat message to everyone else, under the id the server knows them by: the
/// sender id in the client's packet is not taken, or anyone could speak as the server
/// (id 0) or as another player.
pub fn relay_chat(outbox: &mut Vec<OutboxMsg>, sender: u16, msg: &str) {
    let mut pkt = GamePacket::new(PacketType::CHAT_MESSAGE);
    let _ = pkt.write_u16(sender);
    let _ = pkt.write_str(msg);
    outbox.push(OutboxMsg::BroadcastEx(pkt.data().to_vec(), true, sender));
}

pub fn broadcast_chat(outbox: &mut Vec<OutboxMsg>, msg: &str) {
    let mut pkt = GamePacket::new(PacketType::CHAT_MESSAGE);
    let _ = pkt.write_u16(0);
    let _ = pkt.write_str(&msg.to_lowercase());
    outbox.push(OutboxMsg::Broadcast(pkt.data().to_vec(), true));
}

/// Where the server's packets go: ENet for the players on the network and, in a game
/// that hosts (IServer), the channel to that game's own player (local_link).
pub struct Transport {
    pub host: enet::Host<UdpSocket>,
    pub local: Option<ServerEnd>,
    /// The game connected through `local` and has not left.
    pub local_connected: bool,
}

impl Transport {
    pub fn new(host: enet::Host<UdpSocket>, local: Option<ServerEnd>) -> Transport {
        Transport { host, local, local_connected: false }
    }

    pub fn send(&mut self, slot: usize, data: &[u8], reliable: bool) {
        if slot == HOST_SLOT {
            self.send_local(ToGame::Packet(data.to_vec()));
        } else if let Some(peer) = self.host.get_peer_mut(enet::PeerID(slot)) {
            let _ = peer.send(0, &make_enet_packet(data, reliable));
        }
    }

    /// To every connection, but the one in `except`.
    pub fn broadcast(&mut self, data: &[u8], reliable: bool, except: Option<usize>) {
        let packet = make_enet_packet(data, reliable);
        for peer in self.host.connected_peers_mut() {
            if Some(peer.id().0) != except {
                let _ = peer.send(0, &packet);
            }
        }
        if except != Some(HOST_SLOT) {
            self.send_local(ToGame::Packet(data.to_vec()));
        }
    }

    /// Ends a connection once what was sent to it went out (ENet's disconnect).
    pub fn disconnect(&mut self, slot: usize, reason: u32) {
        if slot == HOST_SLOT {
            self.end_local(reason);
        } else if let Some(peer) = self.host.get_peer_mut(enet::PeerID(slot)) {
            peer.disconnect(reason);
        }
    }

    /// Ends a connection at once (ENet's disconnect_now).
    pub fn disconnect_now(&mut self, slot: usize, reason: u32) {
        if slot == HOST_SLOT {
            self.end_local(reason);
        } else if let Some(peer) = self.host.get_peer_mut(enet::PeerID(slot)) {
            peer.disconnect_now(reason);
        }
    }

    /// The connection's round trip; the host's own game has none.
    pub fn round_trip_ms(&self, slot: usize) -> u16 {
        self.host.get_peer(enet::PeerID(slot)).map_or(0, |peer| peer.round_trip_time().as_millis() as u16)
    }

    fn end_local(&mut self, reason: u32) {
        self.send_local(ToGame::Disconnected(reason));
        self.local_connected = false;
    }

    fn send_local(&mut self, message: ToGame) {
        if let (true, Some(local)) = (self.local_connected, &self.local) {
            // The game went away without leaving: it closed, and nobody reads this.
            let _ = local.to_game.send(message);
        }
    }
}

/// Sends what the states said, to the slots their player ids are connected at now. A
/// message to a player who already left goes nowhere.
pub fn apply_outbox(
    net: &mut Transport,
    peers: &crate::connection::PeerRegistry,
    outbox: &mut Vec<OutboxMsg>,
) {
    let slot = |id: u16| peers.slot_of_game_id.get(&id).copied();
    for msg in outbox.drain(..) {
        match msg {
            OutboxMsg::Broadcast(data, reliable) => net.broadcast(&data, reliable, None),
            OutboxMsg::BroadcastEx(data, reliable, exclude_id) => net.broadcast(&data, reliable, slot(exclude_id)),
            OutboxMsg::SendTo(id, data, reliable) => if let Some(slot) = slot(id) { net.send(slot, &data, reliable) },
            OutboxMsg::Disconnect(id, reason) => if let Some(slot) = slot(id) { net.disconnect(slot, reason as u32) },
        }
    }
}

fn make_enet_packet(data: &[u8], reliable: bool) -> enet::Packet {
    if reliable {
        enet::Packet::reliable(data)
    } else {
        enet::Packet::unreliable(data)
    }
}
