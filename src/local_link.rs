//! IServer: the game that started a server in its own process is a player of it
//! without a network in between. Its packets go both ways through channels in memory,
//! byte for byte the packets an ENet peer would send and get, so the lobby, the vote
//! and the round treat it as any other player. It is player 0, which no ENet
//! connection is given (connection.rs PeerRegistry::new_id).

use std::sync::mpsc::{self, Receiver, Sender};

/// The host's slot, past every ENet slot; its player is HOST_PLAYER.
pub const HOST_SLOT: usize = usize::MAX;
pub const HOST_PLAYER: u16 = 0;

/// What the game says to its server.
pub enum ToServer {
    /// The game starts connecting, as an ENet connect does.
    Connect,
    Packet(Vec<u8>),
    /// The game leaves, as ENet's disconnect does.
    Leave,
}

/// What the server says to its game.
pub enum ToGame {
    Connected,
    Packet(Vec<u8>),
    /// The connection is over, for this reason (DisconnectReason).
    Disconnected(u32),
}

/// The game's half.
pub struct GameEnd {
    pub to_server: Sender<ToServer>,
    pub from_server: Receiver<ToGame>,
}

/// The server's half, held by the first lobby.
pub struct ServerEnd {
    pub from_game: Receiver<ToServer>,
    pub to_game: Sender<ToGame>,
}

pub fn pair() -> (GameEnd, ServerEnd) {
    let (to_server, from_game) = mpsc::channel();
    let (to_game, from_server) = mpsc::channel();
    (GameEnd { to_server, from_server }, ServerEnd { from_game, to_game })
}
