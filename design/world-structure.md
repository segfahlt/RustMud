# World Structure

Status: **Active** — authoritative reference for the three-tier world model. All other design docs defer to this one for structural definitions.

---

## Overview

The world is organized in three tiers. Each tier has a distinct role, a distinct creation model, and a distinct relationship to the game's evolution system.

```
Zone  (hex geographic unit, ~500m flat-to-flat)
  └── Areas  (navigable outdoor spaces, AI-generated, traffic-driven)
        └── Fixtures  (freestanding interactive objects, some with Room exits)

Rooms  (globally registered, builder-authored interior spaces)
  └── Fixtures  (furnishings, equipment, and exits back to Areas)
```

Zones own Areas. Rooms are **not** owned by any Zone or Area — they are globally registered at the World level. The connection between the outdoor world and a Room cluster is a **Permanent Fixture** exit: a building entrance, a cave mouth, a gate. Exiting a Room cluster returns the player to an Area via an explicit exit.

**Settlements are 100% Room clusters.** Firstfall and all other built settlements exist entirely as Room nodes. Their descriptions may evoke open streets and plazas — that is a writing choice, not a structural one. The N/S/E/W exits from a city gate are the transition point from Room navigation to Area navigation.

---

## Zone

A Zone is a hex-shaped geographic unit derived from the Azgaar world map. It is the top-level administrative and environmental unit of the world.

**Geographic properties:**
- Fixed hex shape, approximately 500m flat-to-flat
- Identified by axial coordinates `(q, r)` on the world hex grid
- Six neighbors at offsets `(±1, 0)`, `(0, ±1)`, `(1, −1)`, `(−1, 1)` — computed on demand, not stored
- `map_cells` records which Azgaar cells this zone covers

**What zones own directly:**
- `biome_origin` — the biome type at world generation (from map symbol). Permanent baseline.
- `coherence` — the live Coherence level (0–100). This is the one zone-level value that is NOT derived from Areas — it propagates from the planetary network, is influenced by human infrastructure and faction activity, and bleeds between adjacent zones over time.
- `areas` — the Zone's Area registry. Zones own Areas; they do not own Rooms.

**What zones derive from their Areas:**
- `human_pressure` — weighted average of Area evolution stages
- `most_evolved` — highest evolution stage present in any Area in the zone
- `dominant_terrain` — modal biome/terrain type across Areas
- `faction_footprint` — which factions have evidence of presence, and how strong
- `ecological_health` — inverse of human pressure, weighted by untouched Areas

Zone state is a cached summary computed from Areas, refreshed on a tick. It is not independently maintained state that Areas consult — it is a read-only view for reporting, cross-zone comparison, and AI context for first-generation of new Areas in previously unvisited zones.

**Coherence is the exception.** It is zone-level, live, and not derived from Areas. It is the primary non-traffic environmental driver of Area and fixture evolution.

---

## Area

An Area is a navigable outdoor space. It is what traditional MUDs call a "room" — a described location with exits. In this world, Areas are the middle tier: too large and amorphous to be buildings, too small and specific to be zones.

**Key properties:**
- `zone_id` — which zone this Area belongs to
- `offset (i16, i16)` — logical position within the zone from the zone's origin point, used for zone boundary detection
- `stride: Stride` — geographic increment per step. Set at creation based on evolution stage at the time of generation. Immutable after creation.
- `visit_count: u64` — total visits ever
- `recent_visits: u32` — visits in a rolling time window (used for evolution/devolution)
- `last_visited: Timestamp`
- `evolution_stage: EvolutionStage` — current human-impact level (see below)
- `refactor_pending: bool` — true when evolution has crossed a stride boundary and the graph needs restructuring
- `generated: bool` — true if AI-generated, false if hand-authored

Areas are not fixed geometry. Their exits connect only to other Areas. Entry into a Room cluster is handled exclusively by Permanent Fixtures — the Area's exit map never references a Room directly. The graph is non-Euclidean — you can go north five times and south five times and not return to origin. This is intentional and is a property of all text MUDs.

### Stride

Stride is the geographic distance one step represents. It is set at creation based on the Area's initial evolution stage. When evolution crosses a **stride boundary**, the Area is marked for refactoring — its stride and the surrounding graph need restructuring.

| Evolution Stage | Stride |
|---|---|
| Pristine / Marked | 50m |
| Path / Footpath | 25m |
| Trail | 15m |
| Road | 10m |

Stride boundaries sit between Marked→Path, Footpath→Trail, and Trail→Road (and their reverse for devolution). Crossing a boundary sets `refactor_pending: true` on the Area. The refactor does not happen immediately — it is processed by a background task.

### Area Refactoring

When an Area's evolution crosses a stride boundary, its stride needs to change. Since stride affects how much geographic distance each step represents, a stride change requires restructuring the surrounding graph — existing exits may need intermediate Areas inserted between them (on stride decrease) or collapsed (on stride increase).

**Stride decrease (evolution, finer granularity):**

A 50m Area evolves to Path stage (25m stride). The area now represents a finer space than its exits currently express. Intermediate Areas need to be inserted between this Area and each of its neighbors so the graph reflects the new granularity.

```
Before:  [A: 50m] --north--> [B: 50m]
After:   [A: 25m] --north--> [new: 25m] --north--> [B: 50m]
```

The number of intermediates depends on the stride ratio and the distance to the neighbor. This calculation is non-trivial when neighbors have different strides or when the direction is not a clean cardinal axis. **The exact math for intermediate count, placement, and description generation is deferred — this is a known complexity.**

**Stride increase (devolution, coarser granularity):**

A 25m Road devolves to Footpath stage. Intermediate Areas that were inserted during a prior densification may now be candidates for collapse — merging back into their neighbors and removing the intermediate nodes.

Collapsing is more destructive than inserting: it removes existing Areas with their descriptions, fixtures, and potentially player-placed objects. Collapse must be conservative — only Areas that are truly empty of permanent content (no Rooms attached, no player-placed fixtures, no objects) are eligible.

**`refactor_pending: bool`** on the Area struct signals that this Area has crossed a stride boundary and is awaiting refactoring. The game loop processes pending refactors during low-traffic ticks. Players in a pending Area are not refactored around — the refactor waits until the Area is empty.

**What refactoring does NOT do:**
- It does not change the description of the original Area (the AI will update the description separately based on the new evolution stage)
- It does not affect Areas that have not crossed a stride boundary themselves
- It does not cascade automatically to neighbors — each Area's refactor is independent

**Current status:** Refactoring is a known requirement. The stride boundary detection and `refactor_pending` flag will be implemented with the Area struct. The actual graph restructuring logic (intermediate insertion, exit re-linking, collapse) is deferred pending a dedicated design pass on the math.

### Zone Boundary Detection

When generating a new Area from an existing one, the new Area's zone is determined by:

1. Compute the new offset: `new_offset = current_offset + direction_vector`
2. If `|new_offset| <= zone.radius_steps`: new Area belongs to the same zone
3. If `|new_offset| > zone.radius_steps`: new Area belongs to the adjacent zone in that direction

`zone.radius_steps` is a configurable integer per zone (not a meter measurement). A typical wilderness zone has radius 5–8 steps from its origin. The zone crossing is a logical boundary, not a geometric one.

### Area Evolution

Areas evolve forward with traffic and devolve backward without it. Evolution and devolution are separate rates.

```
Pristine → Marked → Path → Footpath → Trail → Road
```

| Stage | Human impact | Typical contents |
|---|---|---|
| Pristine | None | No evidence of human passage |
| Marked | Traces | Footprints, broken branches, a fire ring |
| Path | Consistent passage | A clear line through vegetation |
| Footpath | Regular use | Beaten earth, occasional debris |
| Trail | Established route | Wider, possibly marked, waypoints |
| Road | Maintained infrastructure | Graded surface, structures nearby |

**Evolution drivers (forward):**
- `recent_visits` crossing a threshold for the current stage

**Devolution drivers (backward):**
- `recent_visits` falling below a threshold
- Rate accelerated by: high ecological health (living systems reclaim quickly), high Coherence (network reasserts), wet environment, time since last visit
- Rate decelerated by: Road/Trail stage (infrastructure resists reclamation), dry environment, active faction presence

Zone ecology and Coherence affect devolution rate but not directly the stage — they are multipliers on the time constants, not independent agents of change.

**Devolution floor — permanent fixtures:**

An Area cannot devolve below the minimum evolution stage required by its most demanding permanent fixture. If a building exists in an Area, that Area is anchored at the stage that supports a building — it will not devolve beneath it regardless of traffic.

This is enforced by the fixture itself, not by the Area. Before applying a devolution step, the system checks: does any permanent fixture in this Area require a minimum stage higher than the proposed new stage? If yes, devolution stops there.

Fixtures are either **permanent** or **degradable**:

| Permanence | Behavior on devolution | Examples |
|---|---|---|
| Permanent | Defines a devolution floor. Area cannot devolve below `fixture.minimum_stage`. | Building, well, constructed road surface, permanent marker |
| Degradable | No floor. Devolves or disappears with the area naturally. | Fire pit, lean-to, cache, worn trail marker |

Degradable fixtures below the area's new evolution stage are either removed or transitioned to a ruined/reclaimed description appropriate to the lower stage. This is narratively coherent — a fire pit gets reclaimed by vegetation, a lean-to rots and collapses. A building disappearing because nobody walked past it for a month is not.

A permanent fixture also acts as a **refactoring floor**: an Area with a permanent fixture will not have its stride increased through devolution refactoring while that fixture exists. The fixture must be explicitly demolished before the Area can devolve past its floor.

### Area Generation Context

When the world needs to generate a new Area (player steps into unvisited territory):

**Primary context:** The Area the player just came from — its full description, evolution stage, terrain, visible directions.

**Secondary context:** 2–3 prior Areas in the direction of travel — abbreviated, for narrative thread.

**Zone context:** Zone biome (authoritative for unvisited zones), zone Coherence level, zone trajectory.

**Boundary context** (only when near zone edge): Adjacent zone's biome, Coherence, faction. Instruction to blend transitionally.

**Exploration state:** `visit_count` of this general territory. First-ever visit generates differently from a well-worn path.

The zone state does not *drive* generation. The adjacent Area is the real driver. The zone provides backdrop constraints.

---

## Room

A Room is a globally registered interior space. Rooms are **not** owned by any Zone or Area — they live in a World-level registry alongside (not inside) Zones. This allows Room clusters to span multiple Areas or Zones, and allows structures of any size without geographic constraint.

**Key properties:**
- `id: u32` — globally unique across all Rooms in the world
- `breadcrumb_zone: String` — builder-set display label for the Zone segment of the breadcrumb
- `breadcrumb_building: String` — builder-set display label for the Building segment
- `name: String` — the Room's own name (rightmost breadcrumb segment)
- `coherence_level: Option<u8>` — builder-set. Overrides zone Coherence for this Room. `None` applies a default suppressed level (buildings dampen the network). Set explicitly for caves, shrines, Coherence-dead zones, etc.
- `environment: RoomEnvironment` — builder-set weather exposure, atmosphere, lighting baseline, other zone-style mechanics. Decoupled from any Zone's live state.
- `building_id: Option<u32>` — groups Room nodes belonging to the same structure for automapper and building-level operations.
- `created_by: CreationSource` — `HumanBuilder | Player | AI`
- `owner: Option<CharacterId>` — for player-built structures

**Rooms are off the zone grid.** Zone mechanics (Coherence propagation, weather, faction pressure) do not bleed into Rooms automatically. Every environmental attribute that would otherwise come from a Zone must be set explicitly on the Room by its builder. This is intentional: it allows total environmental control — a dungeon can feel like deep wilderness, a shrine can amplify Coherence, a Corporate lab can suppress it entirely.

**Settlements are Room clusters.** Firstfall and all built settlements are entirely Room nodes. Their descriptions may read as outdoor spaces (streets, plazas, market squares) — this is a writing technique, not a structural property. The city *feels* outdoor; it is structurally a Room cluster. Players know they have crossed into Area mode when the valid exits change and the breadcrumb shifts from `Zone > Building > Room` to `Zone > Area`.

**Who creates Rooms:**

| Source | Method | Examples |
|---|---|---|
| Human builder | Room file authorship, loaded at startup | All of Firstfall, waystations, dungeons |
| AI | Settlement generation on zone evolution | Homesteads, trading posts as zones urbanize |
| Player | `build <structure_type>` command | Personal homes, workshops, storage |

**How player building works:**

A player with sufficient building skill and materials executes `build house` (or similar) in a sufficiently evolved Area. This:
1. Consumes resources from inventory
2. Creates a Permanent Fixture (the building entrance) on the current Area
3. Generates a new Room node in the global registry with AI-generated initial description
4. Sets the fixture's exit destination to the new Room
5. Adds a return exit in the Room (`ExitDestination::Fixture`) pointing back to the entrance fixture
6. Sets `owner`, `building_id`, `created_by: Player` on the new Room
7. Broadcasts construction event to the Area

A building can be expanded: `build room` from inside an existing Room creates an adjacent Room node and connects them. The building grows as a cluster of connected Room nodes with no zone or area constraints.

**Room geometry:**

Rooms are fixed and mappable. The building cluster has a consistent floor plan that automappers can render. Exits connect Room to Room or Room to Area — never Room to Area mid-cluster unless it is an explicit opening (a balcony, a cellar hatch leading to outdoor terrain). Multi-story structures use Up/Down exits freely. Structures that span zones or areas simply have Room-to-Area exits at both ends.

---

## Navigation Graph

Players move through the world as a flat node graph — Areas and Rooms are both nodes with exits. The graph has two distinct **navigation modes** determined by the current node type.

### Navigation Modes

**Area mode** — player is in an Area (outdoor, AI-generated):
- Valid exit directions: `N, S, NE, NW, SE, SW`
- No `E`, `W`, `Up`, or `Down` exits on Areas
- Enforced at load time: Area exits outside this set are a validation error

**Room mode** — player is in a Room (interior, builder-authored):
- Valid exit directions: `N, S, E, W, NE, NW, SE, SW, Up, Down`
- All ten directions available
- The player knows they are indoors (or in a structured interior) from the full direction set and the three-segment breadcrumb

Mode switches on every move that crosses the boundary:
```
Area → Room   (entering a building, cave, or settlement gate)
Room → Area   (exiting to the outdoor world)
```

Zone transitions in Area mode are transparent — the player types `north` and moves, whether or not they cross a zone boundary. The breadcrumb zone segment updates; nothing else changes visibly.

### Exit Destinations

**Area exits** always point to other Areas:
```
Area.exits: HashMap<Direction, AreaRef>
AreaRef = { zone: HexCoord, area_id: u32 }
```

**Room exits** point to either another Room or back to a Permanent Fixture:
```
Room.exits: HashMap<Direction, ExitDestination>

ExitDestination::Room(u32)          — to another Room by global sequence ID
ExitDestination::Fixture(FixtureRef) — return to outdoor Area via the fixture
FixtureRef = { zone: HexCoord, area_id: u32, fixture_id: String }
```

**Area → Room entry** is handled entirely by fixtures, not by Area exits. A Permanent Fixture in an Area carries `connects_to_room: Option<u32>`. When a player moves in a direction where a fixture has a room connection (and no Area exit exists in that direction), they enter the Room.

**Room → Area exit** is handled by `ExitDestination::Fixture`. The Room exit points back to the specific fixture on the specific Area that is the building entrance. When taken, the game looks up the Area that owns that fixture and places the player there. The fixture is the canonical connection point for both directions — entering and exiting pass through the same fixture reference.

A Room with four exits to four different buildings (or four different Areas) simply has four `ExitDestination::Fixture` entries pointing to four different fixtures in four different Areas. No AreaRef or zone coordinate needs to be hardcoded in the Room itself.

### Exit Priority Rule

When a player is in an Area and types a direction:

1. **Area exit only** in that direction → player moves to the Area.
2. **Fixture with `connects_to_room` only** in that direction → player enters the Room.
3. **Both** an Area exit and a fixture with a room connection in the same direction → **Area exit is the default**. Player must type `enter <direction>` to enter the fixture instead.

This prevents accidental Room entry when a path and a building entrance share the same direction.

### Room ID Sequence

Rooms are assigned globally unique `u32` IDs by the server. The current sequence is stored in `data/state/last_room_id` as a single integer. On startup the server loads this into an in-memory atomic counter. OLC room creation and player `build` commands both increment the counter and claim the next ID. The file is flushed on graceful shutdown.

**OLC is the sole room-creation path.** No hand-crafted room JSON files are loaded from disk (outside of the initial migration of existing Firstfall data). This makes the server the single authority on ID assignment — collisions are impossible.

### Location Breadcrumb (display)

Every move event prints a breadcrumb as the first output line:

```
In an Area:   Zone Name > Area Name
In a Room:    Zone Name > Building Name > Room Name
```

For Areas, the Zone Name comes from the Zone that owns the Area. For Rooms, both the Zone Name and Building Name are **builder-set string labels** on the Room — they are not derived from graph position. This allows a building that spans two zones to display whichever zone label makes narrative sense.

The breadcrumb is the complete location title. There is no separate bare name line. See `design/world-building.md` for full formatting rules and examples.

---

## Fixtures in Both Tiers

Fixtures are interactive objects that cannot be picked up. They exist in both Areas and Rooms, with different typical roles:

**In an Area:**
- Freestanding objects without interior space: fire ring, notice board, well, ruined marker, symbiont pod, ancient stone
- **Building entrance fixtures** (Permanent): the gateway into a Room cluster. The fixture owns the connection — it carries `connects_to_room: u32` pointing to the entry Room. The fixture describes the entrance (a heavy door, a cave mouth, a gate); the fixture's room connection is what moves the player. See Exit Priority Rule above.

**In a Room:**
- Interior furnishings and equipment: forge, counter, bed, medical scanner, communications terminal
- **Exit fixtures** (Permanent): doors, hatches, gates that lead back to Areas or to other Room clusters. The fixture is the description; the Room's exit map is the mechanic.

The `Fixture` data model is identical in both contexts. The semantic difference is implied by the Area or Room's purpose, not enforced structurally.

Coherence fixtures (`coherence_driven: true`) can exist in either tier. In Areas they respond to the Zone's live Coherence level. In Rooms they respond to the Room's builder-set `coherence_level`.

---

## How the Tiers Interact at World Generation

**First player entry into an unvisited zone:**
1. Zone exists in the world map with biome and Coherence values
2. No Areas exist yet within it
3. When a player steps in from an adjacent zone, the generator creates the first Area
4. Context: adjacent Area (from origin zone) + new zone's biome + Coherence

**Area densification over time:**
1. An Area evolves from Path to Trail (high traffic)
2. Adjacent unexplored directions get generated as new Path-stage Areas
3. New Areas inherit the Trail's evolved character along the route, Pristine character off it

**Building emergence:**
1. A Trail Area crosses a threshold — "settled enough" for structures
2. AI generates a waystation Room cluster (Rooms registered globally, IDs from sequence)
3. AI creates a Permanent Fixture on the Area with `connects_to_room` pointing to the cluster's entry Room
4. Area description evolves to acknowledge the structure's presence
5. Future Area generation in this zone reflects increased human presence

**Zone state updating:**
1. On a tick, each Zone recomputes derived state from its Areas
2. Coherence propagates: high-Coherence adjacent zones bleed in, heavy human infrastructure suppresses
3. Changed zone state affects future generation context for any new Areas in the zone

---

## Open Questions

- **Last-area tracking**: On death or `recall` from inside a Room cluster, the game places the player back in an Area. Normal Room→Area exits resolve via FixtureRef automatically. Death (no exit taken) needs a `last_area: AreaRef` field on the character save, updated every time a player enters a Room from an Area. Implementation is straightforward; the field just needs to be added to `CharacterSave`.
- **Zone radius_steps**: Should this vary by biome (dense forest = smaller zones, open plains = larger) or be uniform?
- **Room demolition**: Can player-built Rooms be demolished? What happens to the entrance fixture on the Area when they are?
- **Building ownership transfer**: How does a player-built structure transfer to another player or to the settlement?
- **Multi-entrance buildings**: A building with two street-level entrances — two Area nodes both have fixture exits into the same Room cluster. Legal in the graph, needs automapper testing.

### Deferred: Area Refactoring Math

The following questions are explicitly deferred. They are known to be complex and will require a dedicated design pass before implementation:

- **Intermediate count on stride decrease**: When a 50m Area densifies to 25m, how many intermediate Areas are inserted per exit? The naive answer is `old_stride / new_stride - 1` per direction, but exits are not always aligned with stride multiples.
- **Non-cardinal exit handling**: Diagonal exits (NE, SW, etc.) complicate intermediate placement — the geometric relationship between two Areas connected diagonally is not the same as two connected cardinally.
- **Neighbor stride mismatch**: What happens when Area A refactors to 25m but its neighbor B is still 50m? The intermediate sits between them at 25m, but B doesn't refactor just because A did. How does the boundary feel to a player walking from fine to coarse?
- **Collapse eligibility**: On devolution, which intermediates are safe to collapse? What happens to player descriptions, fixtures, and objects in a collapsed Area?
- **Player displacement**: If a player is in an Area being collapsed, where do they go?
- **Automapper reconciliation**: When a refactor inserts or removes nodes, how do clients reconcile their cached maps?
