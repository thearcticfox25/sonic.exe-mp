use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use rusty_enet as enet;

use crate::connection::{handle_identity, PeerRegistry};
use crate::terminal::ConsoleCmd;
use crate::packet::{Packet as GamePacket, PacketType};
use crate::local_link::{ServerEnd, ToGame, ToServer, HOST_SLOT};
use crate::server::{
    apply_outbox, broadcast_chat, DisconnectReason, OutboxMsg, PeerSummary,
    PendingPeer, Server, ServerShared, Transport,
};
use crate::states::lobby::{handle_console_cmd, lobby_broadcast_init, lobby_init};

fn print_network_setup_tutorial(base_port: u16, server_count: u16) {
    log::info!("==========================================================");
    log::info!(" HINT: No one connected in the last 3 minutes.");
    log::info!(" Your ports may not be open. See the setup guide below.");
    log::info!("==========================================================");
    log::info!("");
    log::info!("--- NFTABLES (Linux firewall) ---");
    log::info!("Edit /etc/nftables.conf and add to your inet filter table:");
    log::info!("");
    log::info!("  table inet filter {{");
    log::info!("    chain input {{");
    log::info!("      type filter hook input priority 0; policy drop;");
    log::info!("      ct state established,related accept");
    log::info!("      iif lo accept");
    if server_count > 1 {
        log::info!("      udp dport {}-{} accept   # game ports", base_port, base_port + server_count - 1);
    } else {
        log::info!("      udp dport {} accept   # game port", base_port);
    }
    log::info!("    }}");
    log::info!("  }}");
    log::info!("");
    log::info!("After editing the file, apply the changes (run as root):");
    log::info!("  systemctl restart nftables");
    log::info!("");
    log::info!("--- IF YOUR COMPUTER IS BEHIND A ROUTER ---");
    log::info!("1. Find your local IP address:");
    log::info!("     ifconfig");
    log::info!("   Look for your network interface (e.g. eth0 or enp3s0) and note");
    log::info!("   the inet address, e.g.: 192.168.0.17");
    log::info!("");
    log::info!("2. Open the MikroTik management panel:");
    log::info!("   - Launch WinBox or open a browser -> http://192.168.0.1 (or your gateway)");
    log::info!("   - Assign a static IP to this computer by MAC address:");
    log::info!("       IP -> DHCP Server -> Leases -> find the entry -> make it Static");
    log::info!("");
    log::info!("3. Set up port forwarding via NAT:");
    log::info!("       IP -> Firewall -> NAT -> Add (+)");
    log::info!("       Chain:      dstnat");
    log::info!("       Protocol:   UDP");
    if server_count > 1 {
        log::info!("       Dst. Port:  {}-{}", base_port, base_port + server_count - 1);
    } else {
        log::info!("       Dst. Port:  {}", base_port);
    }
    log::info!("       Action:     dst-nat");
    log::info!("       To Address: <your local IP, e.g. 192.168.0.17>");
    log::info!("       OK -> Apply");
    log::info!("");
    log::info!("4. Share your connection address with players:");
    log::info!("   - Find your PUBLIC IP at https://2ip.ru");
    log::info!("     You can share this IP directly, but if you own a domain name,");
    log::info!("     it is better to use it: a domain is easier to remember, and");
    log::info!("     if your server's IP ever changes you only need to update the DNS");
    log::info!("     record -- players keep connecting to the same address.");
    log::info!("   - To use a domain: add an A record pointing to your public IP");
    log::info!("     at your domain registrar (e.g. reg.ru: Personal account ->");
    log::info!("     Domains -> DNS -> Add record -> Type A).");
    log::info!("     NOTE: the game does NOT support IPv6 -- do NOT add an AAAA record");
    log::info!("     unless you have a specific reason to do so.");
    log::info!("");
    log::info!("--- PLAYING OVER A VPN? ---");
    log::info!("  ZeroTier-One / RadminVPN / Hamachi / Tailscale:");
    log::info!("    Make sure ALL players are connected to the same VPN network,");
    log::info!("    and that they join using your IP address INSIDE that VPN network");
    log::info!("    (not your LAN or public IP).");
    log::info!("");
    log::info!("  Self-hosted OpenVPN / WireGuard:");
    log::info!("    Make sure peer-to-peer traffic is allowed (clients can reach each other).");
    log::info!("    OpenVPN: add 'client-to-client' to your server config.");
    log::info!("    WireGuard: enable IP forwarding and set AllowedIPs to the full VPN subnet.");
    log::info!("    Players must join using the VPN IP, not the LAN or public IP.");
    log::info!("==========================================================");
}

/// A new socket showed up. Send it an authentication ticket and wait for its
/// IDENTITY packet; until then it is only "pending", not a player.
fn on_peer_connect(idx: usize, ip: String, peers: &mut PeerRegistry, server_id: u16) {
    log::debug!("{} New connection from {} (idx={})", crate::server::lobby_tag(server_id), ip, idx);

    let pending = PendingPeer {
        ip,
        timeout: 5.0 * 60.0,
    };
    peers.pending.insert(idx, pending);
}

/// A socket went away. If it belonged to an identified player, tell the current
/// state about it, start their reconnect timeout, and forget them.
fn on_peer_disconnect(
    idx: usize,
    net: &mut Transport,
    server: &mut Server,
    peers: &mut PeerRegistry,
) {
    peers.pending.remove(&idx);

    let game_id = match peers.game_id_of_slot.remove(&idx) {
        Some(id) => id,
        None     => return,
    };
    peers.slot_of_game_id.remove(&game_id);

    if crate::states::host_gui::leave(game_id, server) {
        return;
    }

    let peer = match server.find_peer(game_id) {
        Some(peer) => peer,
        None     => return,
    };
    let nick           = peer.nickname.clone();
    let udid           = peer.udid.clone();
    let ip             = peer.ip.clone();
    let op             = peer.op;
    let should_timeout = peer.should_timeout;

    peers.taken_addresses.remove(&ip);
    peers.taken_addresses.remove(&udid);

    log::info!(
        "{} Player '{}' (id={}) disconnected",
        crate::server::lobby_tag(server.id), crate::colors::colorize(&nick), game_id
    );

    let mut outbox: Vec<OutboxMsg> = Vec::new();
    crate::states::state_left(game_id, server, &mut outbox);

    // Leaving mid-round costs a short reconnect cooldown, so quitting is not a
    // free way to dodge a bad round. Operators are exempt.
    if op < 2 && should_timeout && crate::moderation::timeout_check(&udid, &ip).is_none() {
        let window  = crate::config::cfg().server_config.pairing.rate_limit_window as u64;
        let expires = crate::moderation::now_unix() + window;
        crate::moderation::leave_cooldown_set(&nick, &udid, &ip, expires);
    }

    server.peers.retain(|peer| peer.id != game_id);
    apply_outbox(net, peers, &mut outbox);
}

/// A packet arrived. From an identified player it goes to the current state;
/// from a pending connection only an IDENTITY packet is listened to.
fn on_peer_receive(
    idx: usize,
    data: &[u8],
    net: &mut Transport,
    server: &mut Server,
    peers: &mut PeerRegistry,
    all_shared: &[Arc<ServerShared>],
    base_port: u16,
) {
    if let Some(game_id) = peers.game_id_of_slot.get(&idx).copied() {
        let rtt_ms = net.round_trip_ms(idx);
        if let Some(peer) = server.find_peer_mut(game_id) {
            peer.rtt = rtt_ms;
        }

        let mut pkt = GamePacket::from_data(data);
        pkt.pos = 2;

        let mut outbox: Vec<OutboxMsg> = Vec::new();
        crate::states::state_handle(game_id, &mut pkt, server, &mut outbox);
        apply_outbox(net, peers, &mut outbox);
        return;
    }

    if !peers.pending.contains_key(&idx) { return; }

    let pkt = GamePacket::from_data(data);
    if pkt.len < 2 { return; }
    if pkt.packet_type() == Some(PacketType::IDENTITY) {
        handle_identity(idx, data, net, server, peers, all_shared, base_port);
    }
}

/// IServer: what the game that started this server said through its channel, handled
/// as ENet's events are for a slot (local_link HOST_SLOT, player 0). A game that went
/// away without a word (it closed) leaves as if it had said so.
fn serve_host_game(
    net: &mut Transport,
    server: &mut Server,
    peers: &mut PeerRegistry,
    all_shared: &[Arc<ServerShared>],
    base_port: u16,
    server_id: u16,
) {
    let Some(local) = net.local.as_ref() else { return };
    let mut messages: Vec<ToServer> = local.from_game.try_iter().collect();
    if net.local_connected && matches!(local.from_game.try_recv(), Err(mpsc::TryRecvError::Disconnected)) {
        messages.push(ToServer::Leave);
    }
    for message in messages {
        match message {
            ToServer::Connect => {
                net.local_connected = true;
                if let Some(local) = &net.local {
                    let _ = local.to_game.send(ToGame::Connected);
                }
                on_peer_connect(HOST_SLOT, "127.0.0.1".to_string(), peers, server_id);
            }
            ToServer::Packet(data) => on_peer_receive(HOST_SLOT, &data, net, server, peers, all_shared, base_port),
            ToServer::Leave => {
                net.local_connected = false;
                on_peer_disconnect(HOST_SLOT, net, server, peers);
            }
        }
    }
}

pub fn server_worker(
    server_id: u16,
    port: u16,
    running: Arc<AtomicBool>,
    cmd_rx: &mpsc::Receiver<ConsoleCmd>,
    shared: Arc<ServerShared>,
    all_shared: Vec<Arc<ServerShared>>,
    local: Option<ServerEnd>,
) {
    if server_id > 0 {
        std::thread::sleep(std::time::Duration::from_millis(server_id as u64 * 50));
    }

    let mut server = Server::new(server_id);

    let listen_address = crate::config::cfg().server_config.networking.listen_address;
    let socket = match UdpSocket::bind(SocketAddr::new(listen_address, port)) {
        Ok(bound) => bound,
        Err(error) => {
            log::error!(
                "{} Failed to bind UDP socket on {} port {}: {}",
                crate::server::lobby_tag(server_id), listen_address, port, error
            );
            return;
        }
    };

    let host = match enet::Host::new(
        socket,
        enet::HostSettings {
            peer_limit: 50,
            channel_limit: 2,
            ..Default::default()
        },
    ) {
        Ok(created) => created,
        Err(error) => {
            log::error!("{} Failed to create ENet host: {:?}", crate::server::lobby_tag(server_id), error);
            return;
        }
    };

    crate::terminal::print_always(module_path!(), &format!("{} Listening on port {}", crate::server::lobby_tag(server_id), port));
    let mut net = Transport::new(host, local);
    let mut peers = PeerRegistry::new();

    {
        let mut outbox: Vec<OutboxMsg> = Vec::new();
        lobby_init(&mut server, &mut outbox);
        lobby_broadcast_init(&server, &mut outbox);
        apply_outbox(&mut net, &peers, &mut outbox);
    }

    let target_tick = Duration::from_nanos((1_000_000_000.0 / crate::core::config::ticks_per_second()) as u64);
    let mut next_tick = Instant::now() + target_tick;

    let cfg = crate::config::cfg();
    let base_port = cfg.server_config.networking.port;
    let server_count = cfg.server_config.networking.server_count;

    let mut empty_ticks: u64 = 0;
    let mut tutorial_shown = false;
    let tutorial_ticks = crate::core::config::ticks(3.0 * 60.0) as u64;
    let mut ever_had_player = false;

    const SUMMARY_REFRESH_TICKS: u64 = 10;
    let mut summary_tick: u64 = 0;
    let mut last_peer_count: usize = usize::MAX;

    // Flood cap: bounds how many ENet events a tick drains, and how many receives
    // from any single peer it processes, so one flooding/buggy client can't stall
    // ticking for everyone else. Both reset each tick, not each pass of the loop,
    // which comes round every millisecond.
    let mut serviced_events: u32 = 0;
    let mut peer_drained: HashMap<usize, u16> = HashMap::new();

    log::debug!("Entering main loop...");
    while running.load(Ordering::Relaxed) && server.running {
        while serviced_events < cfg.server_config.networking.vinny.max_events_per_tick {
            match net.host.service() {
                Ok(Some(event)) => {
                    serviced_events += 1;
                    let event_kind = event.no_ref();
                    match event_kind {
                        enet::EventNoRef::Connect { peer: peer_id, .. } => {
                            let ip = net
                                .host
                                .get_peer(peer_id)
                                .and_then(|enet_peer| enet_peer.address())
                                // A server listening on :: takes IPv4 players too, as ::ffff:a.b.c.d: written the
                                // IPv4 way they stay the players the moderation lists and ?gui's loopback check know.
                                .map(|address| address.ip().to_canonical().to_string())
                                .unwrap_or_default();
                            on_peer_connect(peer_id.0, ip, &mut peers, server_id);
                        }

                        enet::EventNoRef::Disconnect { peer: peer_id, .. } => {
                            on_peer_disconnect(peer_id.0, &mut net, &mut server, &mut peers);
                        }

                        enet::EventNoRef::Receive { peer: peer_id, packet, .. } => {
                            let idx = peer_id.0;
                            // Flood cap, per peer and per tick: a client that keeps
                            // shouting gets its extra packets dropped rather than
                            // stalling the tick for everyone else.
                            let drained = peer_drained.entry(idx).or_insert(0);
                            *drained += 1;
                            if *drained > cfg.server_config.networking.vinny.max_packets_per_peer_per_tick {
                                continue;
                            }
                            on_peer_receive(
                                idx,
                                &packet.data().to_vec(),
                                &mut net,
                                &mut server,
                                &mut peers,
                                &all_shared,
                                base_port,
                            );
                        }
                    }
                }
                Ok(None) => break,
                Err(error) => {
                    log::warn!("{} ENet service error: {:?}", crate::server::lobby_tag(server_id), error);
                    break;
                }
            }
        }

        serve_host_game(&mut net, &mut server, &mut peers, &all_shared, base_port, server_id);

        while let Ok(cmd) = cmd_rx.try_recv() {
            let mut outbox: Vec<OutboxMsg> = Vec::new();
            match cmd {
                ConsoleCmd::Chat(msg) => {
                    broadcast_chat(&mut outbox, &msg);
                }
                ConsoleCmd::ExecAsServer(line) => {
                    handle_console_cmd(&line, &mut server, &mut outbox);
                }
            }
            apply_outbox(&mut net, &peers, &mut outbox);
        }

        let now = Instant::now();
        if now >= next_tick {
            next_tick += target_tick;
            serviced_events = 0;
            peer_drained.clear();

            let delta = server.delta;
            peers.pending.retain(|idx, pp| {
                pp.timeout -= delta * 60.0;
                if pp.timeout <= 0.0 {
                    net.disconnect_now(*idx, DisconnectReason::ServerTimeout as u32);
                    false
                } else {
                    true
                }
            });

            if server_id == 0 && !tutorial_shown && cfg.miscellaneous.other.instructor_enabled {
                if !server.peers.is_empty() {
                    ever_had_player = true;
                }
                if ever_had_player {
                    empty_ticks = 0;
                } else {
                    empty_ticks += 1;
                    if empty_ticks >= tutorial_ticks {
                        print_network_setup_tutorial(base_port, server_count);
                        tutorial_shown = true;
                    }
                }
            }

            if empty_ticks % crate::core::config::ticks(60.0) as u64 == 0 || empty_ticks == 1 {
                crate::moderation::cleanup_expired_timeouts();
            }

            let mut outbox: Vec<OutboxMsg> = Vec::new();
            crate::states::state_tick(&mut server, &mut outbox);
            apply_outbox(&mut net, &peers, &mut outbox);

            let to_disconnect: Vec<u16> = server
                .peers
                .iter()
                .filter(|peer| peer.disconnecting)
                .map(|peer| peer.id)
                .collect();
            for id in to_disconnect {
                if let Some(idx) = peers.slot_of_game_id.get(&id).copied() {
                    net.disconnect(idx, DisconnectReason::KickedByHost as u32);
                }
            }

            summary_tick += 1;
            let peer_count = server.peers.len();
            if peer_count != last_peer_count || summary_tick % SUMMARY_REFRESH_TICKS == 0 {
                last_peer_count = peer_count;
                let summary: Vec<PeerSummary> = server
                    .peers
                    .iter()
                    .map(|peer| PeerSummary {
                        id: peer.id,
                        ip: peer.ip.clone(),
                        udid: peer.udid.clone(),
                        nickname: peer.nickname.clone(),
                        op: peer.op,
                        in_game: peer.in_game,
                    })
                    .collect();
                if let Ok(mut guard) = shared.peers.write() {
                    *guard = summary;
                }
            }
        }

        // A flooded tick services ENet no more until the next one: what was sent since
        // still goes out now, not a tick late.
        if serviced_events >= cfg.server_config.networking.vinny.max_events_per_tick {
            net.host.flush();
        }
        std::thread::sleep(Duration::from_millis(1));
    }

    let remaining_ids: Vec<(u16, usize)> = server.peers.iter()
        .filter_map(|peer| peers.slot_of_game_id.get(&peer.id).map(|&idx| (peer.id, idx)))
        .collect();
    for (_id, idx) in remaining_ids {
        net.disconnect_now(idx, DisconnectReason::Shutdown as u32);
    }

    log::info!("{} Shutting down.", crate::server::lobby_tag(server_id));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_link::{self, HOST_PLAYER};

    /// IServer: the game that started the server joins its first lobby through memory,
    /// is answered as any player is, and is player 0.
    #[test]
    fn the_host_game_joins_as_player_zero_through_memory() {
        crate::config::set_test_config();
        crate::status::init_empty_status();
        crate::moderation::init_new_server_moderation();
        let (game, server_end) = local_link::pair();
        let running = Arc::new(AtomicBool::new(true));
        let shared = Arc::new(ServerShared::new());
        let worker = {
            let (running, shared) = (running.clone(), shared.clone());
            std::thread::spawn(move || {
                let (_commands, command_rx) = mpsc::sync_channel(1);
                // Port 0: any free port, so the test needs none of its own.
                server_worker(0, 0, running, &command_rx, shared.clone(), vec![shared], Some(server_end));
            })
        };
        let wait = |what: &str| game.from_server.recv_timeout(Duration::from_secs(4)).unwrap_or_else(|_| panic!("no {what}"));

        game.to_server.send(ToServer::Connect).unwrap();
        assert!(matches!(wait("connect"), ToGame::Connected));

        let mut identity = GamePacket::new(PacketType::IDENTITY);
        let _ = identity.write_char(crate::packet::VERSION);
        let _ = identity.write_u32(u32::MAX);
        let _ = identity.write_str("fox");
        let _ = identity.write_str("test-device");
        let _ = identity.write_u8(0);
        let _ = identity.write_u8(0);
        game.to_server.send(ToServer::Packet(identity.data().to_vec())).unwrap();
        let player_id = loop {
            let ToGame::Packet(data) = wait("identity response") else { panic!("disconnected") };
            if PacketType::from_u8(data[1]) == Some(PacketType::SERVER_IDENTITY_RESPONSE) {
                let mut response = GamePacket::from_data(&data);
                response.pos = 3;
                break response.read_u16().unwrap();
            }
        };
        assert_eq!(player_id, HOST_PLAYER);

        running.store(false, Ordering::Relaxed);
        worker.join().unwrap();
    }
}
