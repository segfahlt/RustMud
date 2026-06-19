# RustMud

A science-fiction MUD set on **Kepler-442b ("The Eye")** — an alien world where humanity's first interstellar colony is struggling to survive. Written in Rust with a focus on a clean, extensible engine that can support a full persistent world with real game depth.

## Setting

The colony ship *Perihelion* made landfall at the site now called **Firstfall**. Players wake from cryosleep into a world where corporate authority, alien coherence fields, and the demands of survival shape every interaction. The world is divided between outdoor hex zones — each with its own biome, evolution stage, and alien coherence level — and building interiors accessed through fixture gateways.

## Architecture

RustMud runs as two separate binaries:

- **Gateway** (`src/bin/gateway.rs`) — holds all player TCP connections. Never needs to restart. Survives game reboots without dropping sockets.
- **Game** (`src/main.rs`) — runs the world, session state machine, and command logic. Communicates with the gateway over a Unix domain socket using newline-delimited JSON.

This split means the game can be rebooted for code changes or world reloads while players stay connected — no re-login required, no dropped connections.

## World Model

### Outdoor zones (hex grid)
The overworld uses a hex coordinate system. Each **zone** (one hex cell) contains one or more **areas** that players move between. Zones have a biome origin, coherence level, and evolution stage that drive ambient descriptions and alien effects.

### Building interiors (room networks)
Buildings are networks of **rooms** loaded from separate JSON files. Fixture objects in areas act as gateways — entering them moves the player into the room cluster. Rooms connect to each other and back to area fixtures via directional exits.

### World continuity
Floor objects persist across reboots. Items dropped in an area or room are saved on shutdown and restored on startup, so the world state survives restarts.

## Object System

Objects are defined as **templates** (blueprints in JSON data files) and instantiated as **instances** at runtime. The object registry holds all templates; instances carry per-copy state.

### Categories

| Category | Description |
|----------|-------------|
| `weapon` | Wieldable in main or off hand |
| `armor` | Worn in a body slot |
| `tool` | Utility item |
| `consumable` | Eaten, drunk, or used — restores health or triggers effects |
| `component` | Crafting material, typically stackable |
| `container` | Carried storage with a slot capacity |
| `data` | Books, notes, data chips — readable |
| `currency` | Stackable credits and trade tokens |
| `trade_good` | Barter items |
| `quest` | Key story items — cannot be dropped or sold |
| `bonded` | Soul-bound to the first character who picks them up |
| `structural` | Walls, gates, doors |
| `crafting_station` | Forges, workbenches, terminals |
| `environmental` | Plants, terrain features, ambient scenery |
| `toggle` | Switches, levers, control panels |
| `commerce` | Vendor terminals |
| `coherence` | Alien growths and coherence emitters |

### Flags

`STACKABLE` · `NO_DROP` · `NO_SELL` · `NO_GIVE` · `NO_TRADE` · `BONDED` · `TWO_HANDED` · `LIGHT_SOURCE` · `PERISHABLE` · `RESTRICTED` · `HIDDEN` · `QUEST` · `EARTH_ORIGIN` · `CORPORATE_ISSUE` · `SETTLER_MADE` · `ALIEN_MADE` · `SALVAGED`

### Equipment slots

`main_hand` · `off_hand` · `body` · `head` · `hands` · `feet`

### Item behaviours

- **Stacking** — stackable items merge automatically on pickup and split on drop; quantity is shown in inventory
- **Containers** — items can be put into and retrieved from carried bags and cases
- **Bonded items** — bind to the first character who picks them up; no other character can take them
- **Quest / NoDrop** — enforced at drop and sell; cannot be discarded

## Commands

### Movement and navigation
`look` · `look <dir>` · `examine <thing>` · `go <dir>` · `enter <dir>` · `wmap`

### Items
`get <item>` · `drop <item>` · `get <n> <item>` · `drop <n> <item>`
`put <item> in <container>` · `get <item> from <container>` · `look in <container>`
`read <item>` · `eat <item>` · `drink <item>` · `use <item>`

### Equipment
`wield <weapon>` · `wear <armor>` · `remove <item>` · `equipment` · `inventory`

### Information
`help` · `help <command>` · `ohelp` · `ohelp -list` · `ohelp <name>` · `ohelp -desc <text>`

### Admin / Builder
`ofind <template_id>` — locate every instance of an object across floors, online players, and offline character saves
`teleport <room_id>` · `teleport <q> <r> <area_id>`
`shutdown` · `reboot` · `reboot refresh`

## Building and Running

### Build

```bash
cargo build
```

### Start the gateway

```bash
cargo run --bin gateway
```

### Start the game (second terminal)

```bash
cargo run
```

### Connect (third terminal)

```bash
telnet localhost 4000
```

Type `help` for available commands.

## Data Layout

```
data/
  zones/           # Outdoor hex world — committed to git
  buildings/       # Interior room clusters — committed to git
  world/           # World map and global config
  state/           # Runtime — gitignored (room ID sequence)
  accounts/        # Runtime — gitignored
  characters/      # Runtime — gitignored
  save/            # Runtime — gitignored (world state)
    state.json
```

Zone and building files follow the schema printed by:

```bash
cargo run --bin schema
```

## Permissions

Permissions live on the character file, not the account. One account can have an admin character and a regular player character.

| Permission | Description |
|------------|-------------|
| `player`   | Baseline — all characters |
| `remort`   | Has completed a remort cycle |
| `builder`  | Can use builder tools and edit world content in-game |
| `monitor`  | Can observe other players |
| `dev`      | Developer access — can `reboot` |
| `admin`    | Full access — satisfies any permission check |

The first character ever created on a fresh server is automatically granted Admin.

To grant permissions manually, edit the character JSON:

```json
{ "permissions": ["player", "builder"] }
```

## Running Tests

```bash
cargo test
```

All tests are inline — parse tests cover every command variant, behaviour tests exercise commands through the full `execute()` path.
