# RustMud Architecture

## Overview

RustMud is split into two long-running binaries that communicate over a Unix domain socket using newline-delimited JSON.

```
Players (TCP)
    │
    ▼
┌─────────────────────────────────────────────┐
│  Gateway  (src/bin/gateway.rs)              │
│  - Holds all TCP sockets                    │
│  - Stores character_id per connection       │
│  - Survives game reboots                    │
└───────────────┬─────────────────────────────┘
                │ Unix socket (/tmp/rustmud.sock)
                │ Newline-delimited JSON (GameMsg / GatewayMsg)
                ▼
┌─────────────────────────────────────────────┐
│  Game Loop  (src/main.rs)                   │
│  - Session state machine per client         │
│  - World model + command execution          │
│  - Saves state on shutdown/reboot           │
└─────────────────────────────────────────────┘
```

The gateway is the durable half. The game loop can be killed and restarted freely — when it reconnects, the gateway re-announces every connected client (with their stored `character_id`), and the game restores their session without prompting.

---

## IPC Protocol (`src/proto.rs`)

### Game → Gateway (`GatewayMsg`)

| Variant | Fields | Effect |
|---------|--------|--------|
| `Output` | `client_id`, `text` | Send text to one client |
| `Broadcast` | `text` | Send text to all clients |
| `Disconnect` | `client_id` | Close one client's TCP socket |
| `Authenticated` | `client_id`, `character_id` | Gateway stores character_id for this socket |
| `DisconnectAll` | `message` | Broadcast message, close all sockets, clear map |
| `Shutdown` | — | Disconnect all, `process::exit(0)` in gateway |

### Gateway → Game (`GameMsg`)

| Variant | Fields | When |
|---------|--------|------|
| `Connect` | `client_id`, `addr`, `character_id?` | New TCP connection, or re-announce on game reconnect |
| `Input` | `client_id`, `line` | A line of text from a player |
| `Disconnect` | `client_id` | Player's TCP socket closed |

---

## Session State Machine (`src/main.rs`)

Each connected client has a `SessionState` entry in a `HashMap<u32, SessionState>`.

```
NeedUsername
    │ valid username
    ├── account exists ──► NeedPassword ──► verified ──► CharacterSelect
    └── new account   ──► NeedNewPassword ──► NeedPasswordConfirm ──► CharacterSelect
                                                                           │
                                                         N (new char) ──► NeedCharName
                                                         1..n (pick)  ──► Playing
                                                                           │
                                                                      Playing (game loop)
```

`SessionState::Playing` carries `{ account_id, character_id, permissions }`. Permissions are loaded from the character file at login and cached in the session — they don't refresh mid-session.

On `GameMsg::Connect { character_id: Some(id) }` (reboot reconnect), the game skips straight to `Playing` by calling `restore_character()`.

---

## Persistence

### Runtime files (gitignored)

| Path | Content |
|------|---------|
| `data/accounts/<username>.json` | Account: id, username, password_hash (argon2id), character list |
| `data/characters/<name>.json` | Character: id, account_id, display name, home_room, permissions |
| `data/save/state.json` | World save: character positions + health (written on shutdown/reboot) |

### Static data (committed)

| Path | Content |
|------|---------|
| `data/zones/*.json` | Zone definitions: rooms, exits, descriptions |

### Key invariants

- Character ID = `name.to_lowercase()`. Globally unique. Stable across reboots.
- Permissions live on the character file, not the account. One account can hold an admin character and a regular player character for testing.
- `write_world_save` is called on every clean shutdown and reboot. Dirty exits lose position data but nothing permanent.

---

## Module Layout

```
src/
  lib.rs              — pub mod declarations
  main.rs             — game loop, session state machine, save/restore
  proto.rs            — GameMsg / GatewayMsg IPC types
  persist.rs          — all file I/O: accounts, characters, world save, permissions
  commands/
    mod.rs            — Command enum, parse(), help_text()
    registry.rs       — CommandDef, Registry, priority-based prefix matching
  game.rs             — GameState, execute(), movement logic
  world/
    mod.rs            — World (zone map), load helpers
    zone.rs           — Zone struct
    room.rs           — Room, RoomRef, Direction, exits map
    loader.rs         — Load zones from data/zones/*.json
  mob/
    mod.rs
    core.rs           — MobCore: id, name, location, health
    player.rs         — Player: MobCore + character_id
  bin/
    gateway.rs        — Durable TCP+Unix gateway
    schema.rs         — Prints JSON Schema for zone files
```

---

## Reboot Mechanics

| Event | What happens |
|-------|-------------|
| `Ctrl+C` on game | `do_save()` at current positions, game exits, gateway waits |
| `reboot` command | Same as Ctrl+C — gateway re-announces clients, game restores sessions |
| `reboot refresh` | `do_save(use_home=true)` — saves everyone at `home_room`, gateway sends `DisconnectAll`, game exits |
| `shutdown` command | `do_save()`, gateway receives `Shutdown` → disconnects all → `process::exit(0)` |

---

## Adding Features: Key Extension Points

- **New command** → `src/commands/registry.rs` (new `CommandDef`) + `src/commands/mod.rs` (new `Command` variant) + `src/game.rs` (`execute()` match arm)
- **New world data** → `src/world/` structs + zone JSON schema
- **New character data** → `CharacterFile` in `persist.rs` + migration note in agent actions
- **New IPC message** → `proto.rs` + handler in gateway.rs `Event::GameMsg` match + sender in main.rs
- **New permission** → `Permission` enum in `persist.rs` + check with `has_perm()` at the call site
