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

## 2026-06-12T20:30:00Z | Planning
**Files:** `planning/theme-lore.md`
**Description:** World bible established for The Eye. Core decisions locked: MUD name is The Eye (also the black hole's settler name). Setting is a sci-fi frontier colony on a hostile alien world orbiting a stellar-mass black hole. The Coherence is the quantum biotic network connecting all planetary life, amplified by the black hole's gravitational gradient. The Shaper is the singular (intentionally ambiguous) extra-dimensional intelligence that cultivated the Coherence over geological timescales — its singularity seeds religion, factions, and castes. Humans are not enemies of the planet but irritants at the wrong quantum frequency; symbiont bonding shifts a character's signature toward compatibility over time. Five factions established: Corporate, Settlers, The Adapted (passive generational change), The Synchronized (deliberate bonding), The Orthodox, The Seekers. Caste structure documented. World events system (Coherence threat level + quantum ship arrivals) and dual power systems (tech vs. symbiont bonding) documented. Open questions listed for next session.

## 2026-06-12T21:30:00Z | Planning, Design
**Files:** `planning/theme-lore.md`, `design/character-creation.md`, `design/firstfall.md`
**Description:** World bible updated with arrival narrative, faction details, and naming. The first colony ship named The Perihelion (settler idiom: "pulling a perihelion" = getting lucky when you had no right to). Landing base: Corporate name Port Meridian, settler name Firstfall. Newbie zone: The Receiving. Character creation design doc created: 3-step flow (name, profession, personal items), 8 professions with starting skills and locker contents, personal items list of ~25 Earth goods across weapons/tools/medical/tech/provisions/sentimental categories. Firstfall zone design created: 12 facility areas (cryo-bay, The Receiving, Corporate Admin, Medical, Engineering Bay, Supply Depot, Canteen, Barracks, Settler Quarter, Research Station, Orthodox Gathering, Perimeter/Gates), each with key rooms and NPCs. Zone progression table from Firstfall outward documented.

## 2026-06-12T22:15:00Z | Planning, Design
**Files:** `planning/theme-lore.md`, `design/firstfall.md`
**Description:** Firstfall geography and age locked in. Colony is 1 year old, ~700 residents. Firstfall sits on a coastal bluff overlooking Perihelion Bay — a natural deepwater harbor formed from the flooded southern rim of an ancient impact crater (~50,000 years ago). Corporate chose the site for mineral-rich ejecta deposits (surface mining base for year one) plus the sheltered port. Serious deposits at the impact center, several miles north, are future content. The impact zone center is Coherence-dead — ecosystem that regrew is younger and strange. Gates updated to cardinal N/E/W/S: South (shuttle pad + harbor, primary entry), North (debris field, resource runs, most-used), East (coastal scrubland, lower Coherence), West (restricted, continental interior). Developed area extends ~1 mile north and east. Zone progression table updated. Mira corrected — not Groundborn, arrived as a child on The Perihelion. Bay ecosystem noted as open question.

## 2026-06-13T00:00:00Z | Planning, Design
**Files:** `planning/theme-lore.md`, `design/firstfall.md`
**Description:** Population revised down to ~100. Original landing was far larger — the planet killed the rest through ecosystem aggression, alien pathogens, equipment failure, and disappearances. The survivors are the careful, the lucky, and the ones who started listening. Empty bunks and unused prefabs tell the story.

## 2026-06-13T01:00:00Z | Design, Zone Data
**Files:** `data/zones/002_forest.json`, `data/zones/001_town.json`
**Description:** Built out Zone 2 (Firstfall) — 37 rooms across all 12 facility areas: The Receiving (6-room intake sequence), Main Street (3 rooms + North Gate), Supply Depot, Corporate Administration (5 rooms), The Canteen, Engineering Bay (4 rooms), Medical Facility (3 rooms + East Gate), Barracks, Settler Quarter (4 rooms), Orthodox Gathering (2 rooms), Research Station (2 rooms), Wall Walk NW. Zone 1 (Arrival) connections verified and intact. All exits validated.

## 2026-06-13T01:30:00Z | Zone Data, Config
**Files:** `data/zones/003_debris.json`, `data/zones/004_coast.json`, `data/zones/005_interior.json`
**Description:** Stub zones for all three outer-gate exits — Debris Field (North Gate), Coastal Scrubland (East Gate), Continental Interior (West Gate). Each has one room with a return exit to the appropriate Firstfall gate. Required for world validation. Zone 5 stub added after initial omission caused test failure.

## 2026-06-13T02:00:00Z | Design
**Files:** `design/world-building.md`
**Description:** Room writing style guide created. Covers the core rule (each room independently intelligible, no adjacent-room context assumed), exit description rules (sensory cues not room names), room anatomy, sensory priority, Corporate vs settler aesthetic, The Coherence (never named, only observed), The Eye (always visible outdoors, never dramatized after first view), environmental accumulation, and ML/LLM integration instructions. Intended to serve as a system prompt context document for agentic world evolution.

## 2026-06-13T02:15:00Z | Zone Data
**Files:** `data/zones/001_town.json`, `data/zones/002_forest.json`
**Description:** Full audit and rewrite of all room descriptions in both zone files per world-building.md core rule. Primary violation pattern: closing sentences that named adjacent rooms ("The cryo-bay is to the east", "Admin lobby is south", "The supply depot entrance is east"). Fixed across all 44 rooms — replaced named-room directional hints with sensory/physical descriptions, or removed them where exits are self-evident. Loader test updated: "Town Square"/"Dark Clearing" → "Cryo-Bay (zone 1, room 1)"/"Intake Lobby (zone 2, room 2)". All 84 tests pass.

## 2026-06-13T03:30:00Z | Design
**Files:** `design/fixtures.md`, `design/world-building.md`
**Description:** Fixture display model finalized. Two-layer approach: (1) state_lines — a map of state keys to one-liner strings, rendered at room enter/look time, no events needed; (2) transition broadcasts — when a handler changes fixture state it broadcasts a message to the room, then state_line picks up automatically for all subsequent enters. Coherence fixtures use global threat level as the state key instead of stored state (coherence_driven: true). Room output order documented: name → description → fixture state lines → object look lines → exits. Rust implementation shape added to fixtures.md. world-building.md updated with room output order section.

## 2026-06-13T03:00:00Z | Design
**Files:** `design/objects.md`, `design/fixtures.md`
**Description:** Full design of both the object system and fixture system. Objects: template/instance model, categories (weapon/armor/tool/consumable/component/container/data/trade/bonded), flags (EARTH_ORIGIN/NO_DROP/BONDED/STACKABLE/etc.), weight category system (tiny/light/medium/heavy/bulky), equipment slots, condition/durability (pristine→broken), special types (Earth items, corporate gear, settler-made, symbiont-bonded), persistence model, template storage layout, cargo locker integration with character creation. Fixtures: 7 categories (structural/container/crafting_station/environmental/toggle/commerce/coherence), full JSON schema with examples for each type, verb dispatch model, requirements system (skill/item/permission/faction/bonding_level), crafting station integration with separate recipe registry, state persistence via FixtureRef, zone file integration, Firstfall fixture inventory (23 fixtures planned across 16 rooms), LLM integration guidance, OLC implications. This makes every NPC significant and the colony's desperation for new arrivals narratively earned.

## 2026-06-13T05:00:00Z | Code, Zone Data
**Files:** `src/world/object.rs` (new), `src/world/fixture.rs` (new), `src/world/room.rs`, `src/world/zone.rs`, `src/world/mod.rs`, `src/world/loader.rs`, `src/commands/mod.rs`, `src/commands/registry.rs`, `src/game.rs`, `src/mob/player.rs`, `src/persist.rs`, `src/main.rs`, `data/zones/001_town.json`, `data/zones/002_forest.json`, `Cargo.toml`
**Description:** Full implementation of the object and fixture systems. New types: ObjectTemplate, ObjectInstance, ObjectCategory, ObjectFlag, Weight, Condition, ObjectRegistry (HashMap<String, ObjectTemplate>), Fixture, FixtureState, FixtureCategory. Room now holds fixtures: Vec<Fixture> and objects: Vec<ObjectInstance>; render() takes &ObjectRegistry and outputs fixture state lines + room-look lines between description and exits. World gains object_registry field built during load. Loader extended: ZoneFile.object_templates registers templates globally; RoomFile.objects are ObjectSpawnFile references that become instances at load time. New commands: Examine (look at <thing> routes here too), Get, Drop, Inventory — all wired through registry with correct priority (movement beats item commands on single chars). Player gains inventory: Vec<ObjectInstance>. CharacterSave gains inventory with #[serde(default)]. do_save and restore_character handle inventory persistence. Zone 1 adds perihelion_manifest template and spawns one in Cargo Bay 3. Zone 2 adds 4 Earth-origin templates (handwritten_note, personal_photograph, old_paperback, earth_whiskey) and 7 fixtures across 6 rooms: orientation_screen (Briefing Room), landing_roster (Records Room), coffee_urn + canteen_notice_board (Canteen), forge (Forge), wall_garden + devotional_focus (coherence-driven, Wall Garden and Orthodox Hall). uuid = "1" added to Cargo.toml. All 106 tests pass.
