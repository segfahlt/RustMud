use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

use rustmud::commands::{parse, Command};
use rustmud::game::{describe_location, execute, GameState};
use rustmud::persist::{
    character_name_taken, has_perm, hash_password, load_account, load_character, load_world_save,
    verify_password, write_account, write_character, write_world_save,
    AccountFile, CharacterFile, CharacterRef, CharacterSave, Permission, WorldSave,
};
use rustmud::proto::{GameMsg, GatewayMsg};
use rustmud::world::loader::{flush_room_id_sequence, load_world};
use rustmud::world::{AreaRef, HexCoord, PlayerLocation, World};

const SOCKET_PATH:  &str = "/tmp/rustmud.sock";
const ACCOUNTS_DIR: &str = "data/accounts";
const CHARS_DIR:    &str = "data/characters";
const SAVE_PATH:    &str = "data/save/state.json";

// Room 1 is the Cryo-Bay aboard the Perihelion — the new player start.
fn start_loc() -> PlayerLocation {
    PlayerLocation::room(1)
}

// ---------------------------------------------------------------------------
// Admin signal — sent from a command handler back to the main loop.
// ---------------------------------------------------------------------------

enum Signal {
    Reboot,         // save + exit; gateway holds connections
    RebootRefresh,  // save at home + disconnect all + exit
    Shutdown,       // save + tell gateway to shut down + exit
}

// ---------------------------------------------------------------------------
// Session state machine — one entry per connected client.
// ---------------------------------------------------------------------------

enum SessionState {
    NeedUsername,
    NeedPassword        { username: String },
    NeedNewPassword     { username: String },
    NeedPasswordConfirm { username: String, hash: String },
    CharacterSelect     { account_id: String },
    NeedCharName        { account_id: String },
    Playing             { account_id: String, character_id: String, permissions: HashSet<Permission> },
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    let world = load_world(Path::new("data")).unwrap_or_else(|e| {
        eprintln!("Error loading world: {e}");
        std::process::exit(1);
    });

    let mut state    = GameState::new(world);
    let mut sessions: HashMap<u32, SessionState> = HashMap::new();
    let mut save     = load_world_save(Path::new(SAVE_PATH));

    eprintln!("Game loop started. Connecting to gateway at {SOCKET_PATH}...");

    let ctrl_c = tokio::signal::ctrl_c();
    tokio::pin!(ctrl_c);

    let (signal_tx, mut signal_rx) = tokio::sync::mpsc::channel::<Signal>(4);

    'outer: loop {
        let stream = connect_with_retry().await;
        eprintln!("Connected to gateway.");

        let (reader, mut writer) = stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        loop {
            tokio::select! {
                biased;

                _ = &mut ctrl_c => {
                    do_save(&state, &sessions, &mut save, false).await;
                    eprintln!("State saved. Shutting down.");
                    break 'outer;
                }

                Some(signal) = signal_rx.recv() => {
                    match signal {
                        Signal::Reboot => {
                            do_save(&state, &sessions, &mut save, false).await;
                            eprintln!("Rebooting game...");
                            std::process::exit(0);
                        }
                        Signal::RebootRefresh => {
                            do_save(&state, &sessions, &mut save, true).await;
                            send(&mut writer, GatewayMsg::DisconnectAll {
                                message: "Game is rebooting. Reconnect to continue.".to_string(),
                            }).await;
                            eprintln!("Rebooting with player reset...");
                            std::process::exit(0);
                        }
                        Signal::Shutdown => {
                            do_save(&state, &sessions, &mut save, false).await;
                            send(&mut writer, GatewayMsg::Shutdown).await;
                            eprintln!("Shutting down.");
                            std::process::exit(0);
                        }
                    }
                }

                result = lines.next_line() => {
                    match result {
                        Ok(Some(line)) => {
                            let msg: GameMsg = match serde_json::from_str(&line) {
                                Ok(m)  => m,
                                Err(e) => { eprintln!("Bad message: {e}"); continue; }
                            };
                            handle_msg(msg, &mut state, &mut sessions, &save, &mut writer, &signal_tx).await;
                        }
                        _ => break,
                    }
                }
            }
        }

        eprintln!("Gateway disconnected. Reconnecting...");
        state.players.clear();
        sessions.clear();
    }
}

async fn connect_with_retry() -> UnixStream {
    loop {
        match UnixStream::connect(SOCKET_PATH).await {
            Ok(s)  => return s,
            Err(_) => tokio::time::sleep(Duration::from_secs(1)).await,
        }
    }
}

// ---------------------------------------------------------------------------
// Top-level message handler
// ---------------------------------------------------------------------------

async fn handle_msg(
    msg:       GameMsg,
    state:     &mut GameState,
    sessions:  &mut HashMap<u32, SessionState>,
    save:      &WorldSave,
    writer:    &mut tokio::net::unix::OwnedWriteHalf,
    signal_tx: &tokio::sync::mpsc::Sender<Signal>,
) {
    match msg {
        GameMsg::Connect { client_id, character_id: Some(cid), .. } => {
            let s = restore_character(client_id, &cid, state, save, writer, false).await;
            sessions.insert(client_id, s);
        }
        GameMsg::Connect { client_id, .. } => {
            sessions.insert(client_id, SessionState::NeedUsername);
            send(writer, GatewayMsg::Output {
                client_id,
                text: "Username: ".to_string(),
            }).await;
        }
        GameMsg::Input { client_id, line } => {
            if let Some(current) = sessions.remove(&client_id) {
                let next = dispatch(
                    client_id, line.trim().to_string(),
                    current, state, save, writer, signal_tx,
                ).await;
                if let Some(s) = next {
                    sessions.insert(client_id, s);
                }
            }
        }
        GameMsg::Disconnect { client_id } => {
            eprintln!("Client {client_id} disconnected");
            state.remove_player(client_id);
            sessions.remove(&client_id);
        }
    }
}

// ---------------------------------------------------------------------------
// State machine dispatch
// ---------------------------------------------------------------------------

async fn dispatch(
    client_id:  u32,
    input:      String,
    current:    SessionState,
    state:      &mut GameState,
    save:       &WorldSave,
    writer:     &mut tokio::net::unix::OwnedWriteHalf,
    signal_tx:  &tokio::sync::mpsc::Sender<Signal>,
) -> Option<SessionState> {
    match current {
        SessionState::NeedUsername =>
            on_username(client_id, input, writer).await,
        SessionState::NeedPassword { username } =>
            on_password(client_id, input, username, state, save, writer).await,
        SessionState::NeedNewPassword { username } =>
            on_new_password(client_id, input, username, writer).await,
        SessionState::NeedPasswordConfirm { username, hash } =>
            on_confirm_password(client_id, input, username, hash, state, save, writer).await,
        SessionState::CharacterSelect { account_id } =>
            on_char_select(client_id, input, account_id, state, save, writer).await,
        SessionState::NeedCharName { account_id } =>
            on_char_name(client_id, input, account_id, state, save, writer).await,
        SessionState::Playing { account_id, character_id, permissions } =>
            on_command(client_id, input, account_id, character_id, permissions, state, writer, signal_tx).await,
    }
}

// ---------------------------------------------------------------------------
// Individual state handlers
// ---------------------------------------------------------------------------

async fn on_username(
    client_id: u32,
    input:     String,
    writer:    &mut tokio::net::unix::OwnedWriteHalf,
) -> Option<SessionState> {
    if !is_valid_username(&input) {
        send(writer, GatewayMsg::Output {
            client_id,
            text: "Username must be 3–24 letters or numbers. Try again: ".to_string(),
        }).await;
        return Some(SessionState::NeedUsername);
    }
    let id = input.to_lowercase();
    if load_account(Path::new(ACCOUNTS_DIR), &id).is_some() {
        send(writer, GatewayMsg::Output { client_id, text: "Password: ".to_string() }).await;
        Some(SessionState::NeedPassword { username: id })
    } else {
        send(writer, GatewayMsg::Output {
            client_id,
            text: format!("Creating new account '{input}'.\nChoose a password (min 8 chars): "),
        }).await;
        Some(SessionState::NeedNewPassword { username: id })
    }
}

async fn on_password(
    client_id: u32,
    input:     String,
    username:  String,
    state:     &mut GameState,
    save:      &WorldSave,
    writer:    &mut tokio::net::unix::OwnedWriteHalf,
) -> Option<SessionState> {
    let account = load_account(Path::new(ACCOUNTS_DIR), &username)?;
    if verify_password(&input, &account.password_hash) {
        let text = char_select_screen(&account, save, state);
        send(writer, GatewayMsg::Output { client_id, text }).await;
        Some(SessionState::CharacterSelect { account_id: username })
    } else {
        send(writer, GatewayMsg::Output {
            client_id,
            text: "Wrong password. Try again: ".to_string(),
        }).await;
        Some(SessionState::NeedPassword { username })
    }
}

async fn on_new_password(
    client_id: u32,
    input:     String,
    username:  String,
    writer:    &mut tokio::net::unix::OwnedWriteHalf,
) -> Option<SessionState> {
    if input.len() < 8 {
        send(writer, GatewayMsg::Output {
            client_id,
            text: "Password must be at least 8 characters. Try again: ".to_string(),
        }).await;
        return Some(SessionState::NeedNewPassword { username });
    }
    let hash = hash_password(&input);
    send(writer, GatewayMsg::Output { client_id, text: "Confirm password: ".to_string() }).await;
    Some(SessionState::NeedPasswordConfirm { username, hash })
}

async fn on_confirm_password(
    client_id: u32,
    input:     String,
    username:  String,
    hash:      String,
    state:     &mut GameState,
    save:      &WorldSave,
    writer:    &mut tokio::net::unix::OwnedWriteHalf,
) -> Option<SessionState> {
    if !verify_password(&input, &hash) {
        send(writer, GatewayMsg::Output {
            client_id,
            text: "Passwords don't match. Choose a password: ".to_string(),
        }).await;
        return Some(SessionState::NeedNewPassword { username });
    }
    let account = AccountFile {
        username:      username.clone(),
        password_hash: hash,
        characters:    vec![],
    };
    write_account(Path::new(ACCOUNTS_DIR), &account)
        .unwrap_or_else(|e| eprintln!("Could not write account: {e}"));
    let text = char_select_screen(&account, save, state);
    send(writer, GatewayMsg::Output { client_id, text }).await;
    Some(SessionState::CharacterSelect { account_id: username })
}

async fn on_char_select(
    client_id:  u32,
    input:      String,
    account_id: String,
    state:      &mut GameState,
    save:       &WorldSave,
    writer:     &mut tokio::net::unix::OwnedWriteHalf,
) -> Option<SessionState> {
    let account = load_account(Path::new(ACCOUNTS_DIR), &account_id)?;
    let lower = input.to_lowercase();

    if lower == "n" {
        send(writer, GatewayMsg::Output {
            client_id,
            text: "Character name (letters only, 3–24 chars): ".to_string(),
        }).await;
        return Some(SessionState::NeedCharName { account_id });
    }

    if let Ok(num) = lower.parse::<usize>() {
        if num >= 1 && num <= account.characters.len() {
            let char_ref = account.characters[num - 1].clone();
            let s = restore_character(client_id, &char_ref.id, state, save, writer, false).await;
            send(writer, GatewayMsg::Authenticated {
                client_id,
                character_id: char_ref.id,
            }).await;
            return Some(s);
        }
    }

    let text = format!("Invalid choice.\n\n{}", char_select_screen(&account, save, state));
    send(writer, GatewayMsg::Output { client_id, text }).await;
    Some(SessionState::CharacterSelect { account_id })
}

async fn on_char_name(
    client_id:  u32,
    input:      String,
    account_id: String,
    state:      &mut GameState,
    save:       &WorldSave,
    writer:     &mut tokio::net::unix::OwnedWriteHalf,
) -> Option<SessionState> {
    if !is_valid_char_name(&input) {
        send(writer, GatewayMsg::Output {
            client_id,
            text: "Name must be 3–24 letters only. Try again: ".to_string(),
        }).await;
        return Some(SessionState::NeedCharName { account_id });
    }
    let char_id = input.to_lowercase();
    if character_name_taken(Path::new(CHARS_DIR), &char_id) {
        send(writer, GatewayMsg::Output {
            client_id,
            text: "That name is taken. Try again: ".to_string(),
        }).await;
        return Some(SessionState::NeedCharName { account_id });
    }

    let is_first_character = std::fs::read_dir(Path::new(CHARS_DIR))
        .map(|mut d| d.next().is_none())
        .unwrap_or(true);
    let permissions: HashSet<Permission> = if is_first_character {
        [Permission::Player, Permission::Admin].into()
    } else {
        [Permission::Player].into()
    };

    write_character(Path::new(CHARS_DIR), &CharacterFile {
        id:            char_id.clone(),
        account_id:    account_id.clone(),
        name:          input.clone(),
        home_location: None,
        permissions,
    }).unwrap_or_else(|e| eprintln!("Could not write character: {e}"));

    if let Some(mut account) = load_account(Path::new(ACCOUNTS_DIR), &account_id) {
        account.characters.push(CharacterRef { id: char_id.clone(), name: input.clone() });
        write_account(Path::new(ACCOUNTS_DIR), &account)
            .unwrap_or_else(|e| eprintln!("Could not update account: {e}"));
    }

    let welcome_suffix = if is_first_character {
        " (Admin access granted — you are the first player.)"
    } else {
        ""
    };
    send(writer, GatewayMsg::Output {
        client_id,
        text: format!("Welcome to RustMud, {input}!{welcome_suffix} Type 'help' for commands.\n\n"),
    }).await;

    let s = restore_character(client_id, &char_id, state, save, writer, true).await;
    send(writer, GatewayMsg::Authenticated { client_id, character_id: char_id }).await;
    Some(s)
}

async fn on_command(
    client_id:    u32,
    input:        String,
    account_id:   String,
    character_id: String,
    permissions:  HashSet<Permission>,
    state:        &mut GameState,
    writer:       &mut tokio::net::unix::OwnedWriteHalf,
    signal_tx:    &tokio::sync::mpsc::Sender<Signal>,
) -> Option<SessionState> {
    let text = match parse(&input) {
        Ok(Command::Shutdown) => {
            if !has_perm(&permissions, Permission::Admin) {
                "Permission denied.\n\n> ".to_string()
            } else {
                send(writer, GatewayMsg::Broadcast {
                    text: "\n*** The game is shutting down. Goodbye! ***\n".to_string(),
                }).await;
                let _ = signal_tx.send(Signal::Shutdown).await;
                "Shutdown signal sent.\n\n> ".to_string()
            }
        }
        Ok(Command::Reboot) => {
            if !has_perm(&permissions, Permission::Dev) {
                "Permission denied.\n\n> ".to_string()
            } else {
                send(writer, GatewayMsg::Broadcast {
                    text: "\n*** The game is rebooting. Hold tight... ***\n".to_string(),
                }).await;
                let _ = signal_tx.send(Signal::Reboot).await;
                "Reboot signal sent.\n\n> ".to_string()
            }
        }
        Ok(Command::RebootRefresh) => {
            if !has_perm(&permissions, Permission::Admin) {
                "Permission denied.\n\n> ".to_string()
            } else {
                send(writer, GatewayMsg::Broadcast {
                    text: "\n*** The game is rebooting with a player reset. ***\n".to_string(),
                }).await;
                let _ = signal_tx.send(Signal::RebootRefresh).await;
                "Reboot refresh signal sent.\n\n> ".to_string()
            }
        }
        Ok(Command::Quit) => {
            let (output, _) = execute(Command::Quit, client_id, state);
            send(writer, GatewayMsg::Output { client_id, text: output }).await;
            send(writer, GatewayMsg::Disconnect { client_id }).await;
            state.remove_player(client_id);
            return None;
        }
        Ok(cmd) => {
            let (output, keep_playing) = execute(cmd, client_id, state);
            if !keep_playing {
                send(writer, GatewayMsg::Output { client_id, text: output }).await;
                send(writer, GatewayMsg::Disconnect { client_id }).await;
                state.remove_player(client_id);
                return None;
            }
            format!("{output}\n> ")
        }
        Err(e) => format!("{e}\n\n> "),
    };
    send(writer, GatewayMsg::Output { client_id, text }).await;
    Some(SessionState::Playing { account_id, character_id, permissions })
}

// ---------------------------------------------------------------------------
// Character restore
// ---------------------------------------------------------------------------

async fn restore_character(
    client_id:    u32,
    character_id: &str,
    state:        &mut GameState,
    save:         &WorldSave,
    writer:       &mut tokio::net::unix::OwnedWriteHalf,
    is_new:       bool,
) -> SessionState {
    let char_file    = load_character(Path::new(CHARS_DIR), character_id);
    let display_name = char_file.as_ref().map(|f| f.name.as_str()).unwrap_or(character_id);
    let account_id   = char_file.as_ref().map(|f| f.account_id.clone()).unwrap_or_default();

    let (location, health, max_health) = match save.characters.get(character_id) {
        Some(cs) => {
            let loc = if location_exists(cs.location, &state.world) { cs.location } else { start_loc() };
            (loc, cs.health, cs.max_health)
        }
        None => (start_loc(), 100, 100),
    };

    state.add_player(client_id, character_id, display_name, location);
    if let Some(p) = state.players.get_mut(&client_id) {
        p.core.health     = health;
        p.core.max_health = max_health;
        if let Some(cs) = save.characters.get(character_id) {
            p.inventory  = cs.inventory.clone();
            p.last_area  = cs.last_area;
        }
    }

    let loc_desc = describe_location(client_id, state);
    let text = if is_new {
        format!("{loc_desc}\n> ")
    } else {
        format!("Welcome back, {display_name}!\n\n{loc_desc}\n> ")
    };
    send(writer, GatewayMsg::Output { client_id, text }).await;

    let permissions = char_file.as_ref()
        .map(|f| f.permissions.clone())
        .unwrap_or_else(|| [Permission::Player].into());

    SessionState::Playing { account_id, character_id: character_id.to_string(), permissions }
}

fn location_exists(loc: PlayerLocation, world: &World) -> bool {
    match loc {
        PlayerLocation::Area { zone_q, zone_r, area_id } => {
            world.get_area(AreaRef { zone: HexCoord::new(zone_q, zone_r), area_id }).is_some()
        }
        PlayerLocation::Room { room_id } => world.get_room(room_id).is_some(),
    }
}

// ---------------------------------------------------------------------------
// Character-select screen
// ---------------------------------------------------------------------------

fn char_select_screen(account: &AccountFile, save: &WorldSave, state: &GameState) -> String {
    let mut out = format!("\nWelcome, {}!\n\n-- Characters --\n", account.username);
    if account.characters.is_empty() {
        out.push_str("  (no characters yet)\n");
    } else {
        for (i, cr) in account.characters.iter().enumerate() {
            let location_name = save.characters.get(&cr.id)
                .map(|cs| location_display_name(cs.location, &state.world))
                .unwrap_or_else(|| "The Beginning".to_string());
            out.push_str(&format!("  {}. {} — {}\n", i + 1, cr.name, location_name));
        }
    }
    out.push_str("\n  N. New character\n\nChoose: ");
    out
}

fn location_display_name(loc: PlayerLocation, world: &World) -> String {
    match loc {
        PlayerLocation::Area { zone_q, zone_r, area_id } => {
            world.get_area(AreaRef { zone: HexCoord::new(zone_q, zone_r), area_id })
                .map(|a| a.name.clone())
                .unwrap_or_else(|| "The Beginning".to_string())
        }
        PlayerLocation::Room { room_id } => {
            world.get_room(room_id)
                .map(|r| r.name.clone())
                .unwrap_or_else(|| "The Beginning".to_string())
        }
    }
}

// ---------------------------------------------------------------------------
// Save helpers
// ---------------------------------------------------------------------------

async fn do_save(
    state:    &GameState,
    sessions: &HashMap<u32, SessionState>,
    save:     &mut WorldSave,
    use_home: bool,
) {
    for (client_id, session) in sessions {
        if let SessionState::Playing { character_id, .. } = session {
            let snapshot = if use_home {
                let home = home_loc_for(character_id);
                let (health, inventory) = state.players.get(client_id)
                    .map(|p| (p.core.max_health, p.inventory.clone()))
                    .unwrap_or((100, vec![]));
                Some(CharacterSave {
                    location: home, health, max_health: health, inventory, last_area: None,
                })
            } else {
                state.snapshot_character(*client_id)
            };
            if let Some(s) = snapshot {
                save.characters.insert(character_id.clone(), s);
            }
        }
    }
    write_world_save(save, Path::new(SAVE_PATH))
        .unwrap_or_else(|e| eprintln!("Save failed: {e}"));
    flush_room_id_sequence(Path::new("data"), &state.world)
        .unwrap_or_else(|e| eprintln!("Room ID flush failed: {e}"));
}

fn home_loc_for(character_id: &str) -> PlayerLocation {
    load_character(Path::new(CHARS_DIR), character_id)
        .and_then(|c| c.home_location)
        .unwrap_or_else(start_loc)
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

fn is_valid_username(s: &str) -> bool {
    let len = s.len();
    len >= 3 && len <= 24 && s.chars().all(|c| c.is_alphanumeric())
}

fn is_valid_char_name(s: &str) -> bool {
    let len = s.len();
    len >= 3 && len <= 24 && s.chars().all(|c| c.is_alphabetic())
}

// ---------------------------------------------------------------------------
// Wire helper
// ---------------------------------------------------------------------------

async fn send(writer: &mut tokio::net::unix::OwnedWriteHalf, msg: GatewayMsg) {
    let mut line = serde_json::to_string(&msg).unwrap();
    line.push('\n');
    let _ = writer.write_all(line.as_bytes()).await;
}
