# Agent Actions Log

Single append-only log. All timestamps are UTC. Most recent entries at the bottom.
Supersedes the dated file `2026-06-12.md` (safe to delete).

Format per entry:
```
## YYYY-MM-DDTHH:MM:SSZ | Category
**Files:** ...
**Description:** ...
```

Categories: `Code` `Architecture` `Refactor` `Fix` `Config` `Docs` `Design` `Planning`

---

## 2026-06-12T01:00:00Z | Architecture, Code
**Files:** `src/world/`, `src/lib.rs`, `Cargo.toml`
**Description:** Initial world model. `Zone`, `Room`, `RoomRef`, `Direction` structs. Rooms have exits as `HashMap<Direction, RoomRef>`. Zones loaded from `data/zones/*.json`. `schemars` added for JSON Schema generation. `src/bin/schema.rs` prints the zone schema.

## 2026-06-12T02:00:00Z | Architecture, Code
**Files:** `src/commands/mod.rs`, `src/commands/registry.rs`
**Description:** Command registry with `CommandDef` structs and priority-based prefix matching. Lower number = higher priority. Resolution order: exact name → exact alias → prefix match sorted by priority. `AmbiguousCommand` error on same-priority ties. Direction commands at priority 5, regular at 10, admin at 20 — this resolved `s` matching both `south` and `shutdown`.

## 2026-06-12T03:00:00Z | Architecture, Code
**Files:** `src/bin/gateway.rs`, `src/proto.rs`, `src/main.rs`
**Description:** Two-binary architecture. Gateway holds TCP connections on port 4000, communicates with game over Unix socket `/tmp/rustmud.sock` via newline-delimited JSON. Game loop is freely restartable — gateway re-announces all clients on reconnect. `ClientInfo` stores `character_id` per socket for seamless reboot restore.

## 2026-06-12T04:00:00Z | Code
**Files:** `src/persist.rs`, `src/main.rs`
**Description:** State save/restore. `WorldSave` persisted to `data/save/state.json` on every clean shutdown or reboot. Stores `CharacterSave` (zone_id, room_id, health, max_health) keyed by character_id. Loaded on game startup to restore player positions without re-prompting.

## 2026-06-12T05:00:00Z | Code
**Files:** `src/persist.rs`, `src/main.rs`
**Description:** Account system with argon2id password hashing. One account, multiple characters. `CharacterFile` with stable id = `name.to_lowercase()`. Character select screen on login. Session state machine: NeedUsername → NeedPassword/NeedNewPassword → CharacterSelect → NeedCharName → Playing. Seamless reboot reconnect via gateway-stored `character_id`.

## 2026-06-12T06:00:00Z | Architecture, Code
**Files:** `src/persist.rs`, `src/commands/mod.rs`, `src/commands/registry.rs`, `src/main.rs`, `src/bin/gateway.rs`
**Description:** Non-exclusive `Permission` enum on `CharacterFile`: player, remort, builder, monitor, dev, admin. `has_perm()` where Admin satisfies any check. `SessionState::Playing` caches permissions loaded at login. Admin commands: `shutdown`, `reboot`, `reboot refresh`. `Signal` enum + `mpsc::channel` for admin→main loop coordination. `GatewayMsg::DisconnectAll` and `GatewayMsg::Shutdown` added to IPC.

## 2026-06-12T18:00:00Z | Architecture, Refactor
**Files:** `src/persist.rs`, `src/main.rs`
**Description:** Moved `permissions` from `AccountFile` to `CharacterFile`. Rationale: one account can hold an admin character and a regular player for testing — permissions should be per-character. `restore_character()` now reads permissions from char file. `#[serde(default = "default_permissions")]` added to `CharacterFile.permissions` for backward compat with existing files.

## 2026-06-12T18:15:00Z | Code
**Files:** `src/main.rs`
**Description:** Bootstrap admin. First character ever created (detected by `data/characters/` being empty at creation time) receives `[player, admin]` permissions automatically. Subsequent characters get `[player]`. Welcome message notes the grant. Eliminates the need to manually edit JSON to get initial Admin access.

## 2026-06-12T18:30:00Z | Docs
**Files:** `README.md`
**Description:** Project README covering architecture overview, features list, build/run instructions, data layout table, permissions table, admin commands table, and test instructions.

## 2026-06-12T18:35:00Z | Config
**Files:** `.gitignore`
**Description:** Added `data/accounts/`, `data/characters/`, `data/save/` to gitignore. Runtime state, not source. Zone files in `data/zones/` remain tracked.

## 2026-06-12T19:00:00Z | Docs, Planning, Design
**Files:** `docs/architecture.md`, `docs/adding-commands.md`, `planning/roadmap.md`, `design/economy.md`, `design/llm-integration.md`, `agent_actions/log.md`
**Description:** Full agentic helper file structure. Architecture doc covers IPC protocol, session state machine, persistence, module layout, reboot mechanics, and extension points. Adding-commands guide covers all three layers (Command variant, CommandDef, execute handler). Roadmap captures 7-phase vision through LLM integration. Economy design covers currency, shops, player vendor stands, auctions. LLM integration design covers 5 use cases, model selection, technical architecture, and open questions. Agent actions log consolidated from dated files into single append-only log with UTC timestamps.

## 2026-06-12T19:30:00Z | Design, Planning
**Files:** `design/objects.md`, `design/fixtures.md`, `design/socials.md`, `design/olc.md`, `design/world-map.md`, `design/player-stats.md`, `design/races.md`, `design/classes.md`, `design/crafting.md`, `design/environment-manipulation.md`, `planning/theme-lore.md`
**Description:** Placeholder design files created for all major upcoming systems: objects/items, fixtures, socials, OLC, world map, player stats/attributes, races, classes, crafting, and environment manipulation (spells/psionics). Theme and lore discussion document created in planning/ to anchor all content decisions before implementation begins.
