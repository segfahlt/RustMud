# RustMud — Project Rules & Conventions

## Architecture

### World Model
- Hierarchy: `World` → `Zone` → `Room`
- Room exits use `RoomRef { zone_id, room_id }` — supports cross-zone exits
- Room IDs are scoped per zone, not globally unique
- World integrity checked via `World::validate()` — call after building a world

### Mob System
- Shared data: `MobCore` (id, name, health, max_health, location)
- Concrete types: `Player`, `Npc`, `Monster` — each embeds a `MobCore`
- Trait: `Mobile` — defines shared interface; only `core()` + `core_mut()` required, rest default
- Enum: `Mob` — wraps concrete types for mixed collections; implements `Mobile` by delegation
- To add a new mob type: create struct with `MobCore`, `impl Mobile`, add variant to `Mob` enum

### Module Layout
```
src/
  lib.rs              shared library — declares all modules (imported by both binaries)
  proto.rs            IPC message types: GameMsg (gateway→game), GatewayMsg (game→gateway)
  main.rs             game binary — async loop, connects to gateway via Unix socket
  bin/
    gateway.rs        durable connection manager — never restarts; holds all TCP clients
    schema.rs         schema binary — prints JSON Schema for all data file formats
  commands/mod.rs     Command enum, ParseError, parse()
  world/
    mod.rs            World struct, validate(), re-exports
    zone.rs           Zone struct
    room.rs           Direction (+ FromStr), RoomRef (+ Deserialize + JsonSchema), Room
    loader.rs         load_world(), LoadError, pub ZoneFile/RoomFile (+ JsonSchema)
  mob/
    mod.rs            Mobile trait, Mob enum, re-exports
    core.rs           MobCore
    player.rs         Player
    npc.rs            Npc
    monster.rs        Monster

data/
  zones/              One JSON file per zone (e.g. 001_town.json)
```

### Networking Architecture
Two separate binaries communicate over a Unix domain socket at `/tmp/rustmud.sock`:

- **`gateway`** (port 4000) — durable; holds all player TCP connections. Never restarts unless absolutely necessary. When the game is down it buffers input and tells players to wait.
- **`rustmud`** (game loop) — connects TO the gateway (not the other way). Can be freely killed and restarted. Players stay connected through a reboot.

**Start order**: start `gateway` first, then `rustmud`. The game retries the Unix socket connection automatically.

**IPC protocol**: newline-delimited JSON. Messages are tagged with `"type"`:
- Gateway → Game (`GameMsg`): `connect`, `input`, `disconnect`
- Game → Gateway (`GatewayMsg`): `output`, `broadcast`, `disconnect`

**Reboot procedure**: kill `rustmud`, restart it. Players see `[Game is rebooting. Please wait...]` and then get their welcome message when the game reconnects.

**`GameState`**: holds `players: HashMap<u32, Player>` keyed by `client_id`. `execute()` takes a `client_id` and returns `(String, bool)` — the output text and whether the player should stay connected.

### Schema Tool
- Run `cargo run --bin schema` to print JSON Schema for all data file formats
- Pipe to a file: `cargo run --bin schema > schema/zone.json`
- When adding a new data file format: derive `JsonSchema` on its file structs, add `schema_for!(...)` to `src/bin/schema.rs`
- Data file structs must be `pub` in their module to be reachable from the schema binary
- `#[serde(default)]` on optional fields correctly marks them non-required in the schema

### Data Files
- Zone files live in `data/zones/`, named `NNN_slug.json` — numeric prefix controls load order
- JSON format: `{ id, name, description, rooms: [{ id, name, description, exits: { "north": { zone_id, room_id } } }] }`
- Direction keys in exits are lowercase strings: `"north"`, `"south"`, `"east"`, `"west"`, `"up"`, `"down"`
- Rooms without exits may omit the `"exits"` key entirely
- After loading, `World::validate()` is called automatically — a bad `RoomRef` in any exit is a hard error at startup

### Command System
- Commands are defined in `src/commands/registry.rs` as `CommandDef` structs registered in `Registry::build()`
- `CommandDef` fields: `name`, `priority: u32`, `aliases`, `category`, `usage`, `description`, `parse: fn(&str) -> Result<Command, ParseError>`
- **Priority convention**: lower number = higher priority (1 beats 2). Used only to break ties when multiple command names share a prefix.
- **Prefix matching**: users can abbreviate any command to any unambiguous prefix — no explicit abbreviation aliases needed.
  - `l` → `look`, `n` → `north`, `q` → `quit`, etc. (all unique in the current command set)
  - When a prefix matches multiple commands, the lowest-priority-number wins.
  - If two matches share the same `priority`, `ParseError::AmbiguousCommand` is returned — this is a configuration error, not user error.
- **Alias rules**: only use `aliases` for things that can't be expressed as name prefixes:
  - Special characters: `"?"` → `help`
  - Full alternative words: `"exit"` → `quit`, `"move"` → `go`
  - Never put single-letter abbreviations in `aliases` — let prefix matching handle them.
- **Argument abbreviation**: direction arguments to `go` and `look` are also prefix-matched (handled by `prefix_match_direction()`).
- **Categories** (`Category` enum in `registry.rs`): `Movement`, `Info`, `Communication` — drives the self-generating help system.
- Lookup order in `Registry::find()`: exact name → exact alias → prefix-sorted-by-priority.

## Rust Conventions
- Private fields, public methods — callers use the API, never raw HashMap access
- `pub use` re-exports in `mod.rs` — flatten the public API; internal structure is an impl detail
- `impl Into<String>` for string params in constructors — accept `&str` or `String` without coercion at callsite
- `#[derive(Debug)]` on all data types
- `Copy` on small, heap-free structs (e.g. `RoomRef`)

## Testing Rules
- All pure logic must be tested: parsers, lookups, calculations, data integrity
- Skip testing: I/O, `describe()` / output formatting (fragile, breaks on wording changes)
- World integrity: always test via `World::validate()` — catches dead exit references
- Tests live in `#[cfg(test)] mod tests` at the bottom of the relevant source file
- Use `assert!(matches!(...))` for enum variant matching
- Test helper functions (e.g. `make_world()`, `loc()`) live inside the `tests` module
