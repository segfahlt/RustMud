use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, UnixListener};
use tokio::sync::mpsc;

use rustmud::color::colorize;
use rustmud::proto::{GameMsg, GatewayMsg};

const TCP_ADDR:    &str = "0.0.0.0:4000";
const SOCKET_PATH: &str = "/tmp/rustmud.sock";

type ClientId = u32;
type TcpWriter  = tokio::net::tcp::OwnedWriteHalf;
type UnixWriter = tokio::net::unix::OwnedWriteHalf;

// Per-connection state the gateway retains across game reboots.
struct ClientInfo {
    writer:       TcpWriter,
    character_id: Option<String>,  // set once the game authenticates the client
    // TODO: auto-detect via TTYPE/MTTS telnet negotiation; allow player override with `color on/off`
    color:        bool,
}

// All events funnel to the central handler task via this channel.
enum Event {
    ClientConnected(ClientId, String, TcpWriter),   // id, peer addr, write half
    ClientInput(ClientId, String),
    ClientDropped(ClientId),
    GameConnected(UnixWriter),
    GameMsg(GatewayMsg),
    GameDropped,
}

#[tokio::main]
async fn main() {
    // Remove stale socket so the new listener binds cleanly.
    let _ = std::fs::remove_file(SOCKET_PATH);

    let (tx, mut rx) = mpsc::channel::<Event>(512);
    let next_id = Arc::new(AtomicU32::new(1));

    let tx2 = tx.clone();
    let id_counter = Arc::clone(&next_id);
    tokio::spawn(async move { tcp_accept_loop(tx2, id_counter).await });

    let tx3 = tx.clone();
    tokio::spawn(async move { unix_accept_loop(tx3).await });

    eprintln!("Gateway listening on {TCP_ADDR}");
    eprintln!("Waiting for game on {SOCKET_PATH}");

    // Central state — only this task touches it, so no locking needed.
    let mut clients: HashMap<ClientId, ClientInfo> = HashMap::new();
    let mut game: Option<UnixWriter> = None;

    while let Some(event) = rx.recv().await {
        match event {
            Event::ClientConnected(id, addr, writer) => {
                eprintln!("Client {id} connected from {addr}");
                clients.insert(id, ClientInfo { writer, character_id: None, color: true });
                match &mut game {
                    Some(gw) => send_game(gw, GameMsg::Connect {
                        client_id: id, addr, character_id: None,
                    }).await,
                    None => write_client(&mut clients, id,
                        "[Game is starting up. Please wait...]\n\n").await,
                }
            }

            Event::ClientInput(id, line) => {
                match &mut game {
                    Some(gw) => send_game(gw, GameMsg::Input { client_id: id, line }).await,
                    None     => write_client(&mut clients, id,
                        "[Game is rebooting. Please wait...]\n").await,
                }
            }

            Event::ClientDropped(id) => {
                eprintln!("Client {id} dropped");
                clients.remove(&id);
                if let Some(gw) = &mut game {
                    send_game(gw, GameMsg::Disconnect { client_id: id }).await;
                }
            }

            Event::GameConnected(writer) => {
                eprintln!("Game connected.");
                game = Some(writer);
                // Re-announce every connected client, including their character_id if known.
                // The game uses character_id to restore state without prompting the player.
                let ids: Vec<ClientId> = clients.keys().copied().collect();
                for id in ids {
                    let character_id = clients[&id].character_id.clone();
                    if let Some(gw) = &mut game {
                        send_game(gw, GameMsg::Connect {
                            client_id: id,
                            addr: "(reconnect)".to_string(),
                            character_id,
                        }).await;
                    }
                }
            }

            Event::GameMsg(msg) => match msg {
                GatewayMsg::Output { client_id, text } => {
                    write_client(&mut clients, client_id, &text).await;
                }
                GatewayMsg::Broadcast { text } => {
                    let ids: Vec<ClientId> = clients.keys().copied().collect();
                    for id in ids {
                        write_client(&mut clients, id, &text).await;
                    }
                }
                GatewayMsg::Disconnect { client_id } => {
                    if let Some(info) = clients.get_mut(&client_id) {
                        let _ = info.writer.shutdown().await;
                    }
                    clients.remove(&client_id);
                    eprintln!("Client {client_id} disconnected by game.");
                }
                GatewayMsg::Authenticated { client_id, character_id } => {
                    if let Some(info) = clients.get_mut(&client_id) {
                        eprintln!("Client {client_id} authenticated as '{character_id}'");
                        info.character_id = Some(character_id);
                    }
                }
                GatewayMsg::DisconnectAll { message } => {
                    eprintln!("Disconnecting all clients: {message}");
                    let ids: Vec<ClientId> = clients.keys().copied().collect();
                    for id in ids {
                        write_client(&mut clients, id, &format!("\n{message}\n")).await;
                        if let Some(info) = clients.get_mut(&id) {
                            let _ = info.writer.shutdown().await;
                        }
                    }
                    clients.clear();
                }
                GatewayMsg::Shutdown => {
                    eprintln!("Shutdown requested by game. Goodbye.");
                    let ids: Vec<ClientId> = clients.keys().copied().collect();
                    for id in ids {
                        write_client(&mut clients, id,
                            "\nThe game is shutting down. Goodbye!\n").await;
                        if let Some(info) = clients.get_mut(&id) {
                            let _ = info.writer.shutdown().await;
                        }
                    }
                    std::process::exit(0);
                }
            },

            Event::GameDropped => {
                eprintln!("Game disconnected.");
                game = None;
                let ids: Vec<ClientId> = clients.keys().copied().collect();
                for id in ids {
                    write_client(&mut clients, id,
                        "\n[Game is rebooting. Please wait...]\n").await;
                }
            }
        }
    }
}

// --- TCP accept loop ---

async fn tcp_accept_loop(tx: mpsc::Sender<Event>, next_id: Arc<AtomicU32>) {
    let listener = TcpListener::bind(TCP_ADDR).await
        .unwrap_or_else(|e| { eprintln!("Cannot bind {TCP_ADDR}: {e}"); std::process::exit(1) });

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                let id  = next_id.fetch_add(1, Ordering::Relaxed);
                let (reader, writer) = stream.into_split();
                let addr_str = addr.to_string();
                let _ = tx.send(Event::ClientConnected(id, addr_str, writer)).await;

                let tx2 = tx.clone();
                tokio::spawn(async move {
                    let mut lines = BufReader::new(reader).lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        if tx2.send(Event::ClientInput(id, line)).await.is_err() { break; }
                    }
                    let _ = tx2.send(Event::ClientDropped(id)).await;
                });
            }
            Err(e) => eprintln!("TCP accept error: {e}"),
        }
    }
}

// --- Unix socket accept loop (one game connection at a time) ---

async fn unix_accept_loop(tx: mpsc::Sender<Event>) {
    let listener = UnixListener::bind(SOCKET_PATH)
        .unwrap_or_else(|e| { eprintln!("Cannot bind {SOCKET_PATH}: {e}"); std::process::exit(1) });

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let (reader, writer) = stream.into_split();
                let _ = tx.send(Event::GameConnected(writer)).await;

                let tx2 = tx.clone();
                let mut lines = BufReader::new(reader).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    match serde_json::from_str::<GatewayMsg>(&line) {
                        Ok(msg) => { if tx2.send(Event::GameMsg(msg)).await.is_err() { break; } }
                        Err(e)  => eprintln!("Bad message from game: {e}"),
                    }
                }
                let _ = tx2.send(Event::GameDropped).await;
            }
            Err(e) => eprintln!("Unix accept error: {e}"),
        }
    }
}

// --- Helpers ---

async fn write_client(clients: &mut HashMap<ClientId, ClientInfo>, id: ClientId, text: &str) {
    if let Some(info) = clients.get_mut(&id) {
        let processed = colorize(text, info.color);
        let _ = info.writer.write_all(processed.as_bytes()).await;
    }
}

async fn send_game(writer: &mut UnixWriter, msg: GameMsg) {
    let mut line = serde_json::to_string(&msg).unwrap();
    line.push('\n');
    let _ = writer.write_all(line.as_bytes()).await;
}
