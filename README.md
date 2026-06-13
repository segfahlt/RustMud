# RustMud

A MUD (Multi-User Dungeon) server written in Rust, built as a learning project with a focus on clean architecture and real async networking.

## Architecture

RustMud runs as two separate binaries:

- **Gateway** (`src/bin/gateway.rs`) — holds all player TCP connections. Never needs to restart. Survives game reboots without dropping sockets.
- **Game** (`src/main.rs`) — runs the world, session state machine, and command logic. Communicates with the gateway over a Unix domain socket (`/tmp/rustmud.sock`) using newline-delimited JSON.

This split means the game loop can be rebooted freely (for code changes, world reloads, or admin commands) while players stay connected with no interruption beyond a brief pause.

## Features

- Multi-player async server (tokio)
- Account system with argon2id password hashing — one account, multiple characters
- Per-character permissions: `player`, `remort`, `builder`, `monitor`, `dev`, `admin`
- First character ever created is automatically granted Admin
- Session state machine: login → character select → playing
- Seamless reconnect after game reboot — no re-login required
- Game state save/restore on shutdown (room positions, health)
- Command registry with prefix matching and priority-based disambiguation
- Zone/room world model loaded from JSON data files
- Admin commands: `shutdown`, `reboot`, `reboot refresh`

## Building and Running

### 1. Build

```bash
cargo build
```

### 2. Start the gateway

```bash
cargo run --bin gateway
```

### 3. Start the game (second terminal)

```bash
cargo run
```

### 4. Connect (third terminal)

```bash
telnet localhost 4000
# or
nc localhost 4000
```

Type `help` for available commands.

## Data Layout

```
data/
  zones/        # World data — committed to git
    001_town.json
    002_forest.json
  accounts/     # Runtime — gitignored
  characters/   # Runtime — gitignored
  save/         # Runtime — gitignored
    state.json
```

Zone files follow the schema printed by:

```bash
cargo run --bin schema
```

## Permissions

Permissions live on the character file (`data/characters/<name>.json`), not the account. This lets one account have an admin character and a regular player character for testing.

| Permission | Description |
|------------|-------------|
| `player`   | Baseline — all characters |
| `remort`   | Has completed a remort cycle |
| `builder`  | Can edit world content in-game |
| `monitor`  | Can observe other players |
| `dev`      | Developer access — can `reboot` |
| `admin`    | Full access — satisfies any permission check |

To grant permissions, edit the character's JSON directly until in-game grant commands are added:

```json
{ "permissions": ["player", "admin"] }
```

## Admin Commands

| Command | Permission | Effect |
|---------|------------|--------|
| `shutdown` | Admin | Saves state, disconnects all players, exits gateway |
| `reboot` | Admin or Dev | Saves state, restarts game; players reconnect automatically |
| `reboot refresh` | Admin | Saves all players at their home room, disconnects all, restarts game |

## Running Tests

```bash
cargo test
```
