# Fixture System — The Eye

Status: **Draft** — design complete, ready for implementation planning

---

## What Fixtures Are

Fixtures are interactive objects fixed to a location. They cannot be picked up or moved. They exist in Areas and Rooms the way furniture exists in a space — they define it and enable action within it, but they do not travel with the player.

Fixtures appear in both tiers of the navigable world:
- **In an Area** (outdoor space): freestanding objects without interior — a fire ring, notice board, well, ancient marker, symbiont pod
- **In a Room** (building interior): furnishings and equipment — a forge, counter, bed, medical scanner, terminal

The data model is identical in both contexts. The semantic role differs by where they appear.

Fixtures can:
- Have extended descriptions separate from the room description
- Be interacted with through a verb system (use, examine, read, open, pull, etc.)
- Hold state that persists across sessions (open/closed, on/off, contents)
- Contain objects (as container fixtures)
- Function as crafting stations
- Act as commerce nodes (vendor stands, vending machines)
- Trigger room and world events
- Carry Coherence interactions (for the planet's responsive ecosystem)

Fixtures are **not** NPCs. An NPC that tends a garden is a mob. The garden itself is a fixture. The interaction is with the fixture; the NPC's presence may modify what's available.

---

## Fixture Categories

### Structural
Permanent features that enrich description and reward examination. No meaningful state.

| Example | Interactions |
|---------|-------------|
| Carved doorframe (Orthodox Threshold) | look, examine |
| Missing persons board (Security Office) | look, examine, read, post (if permitted) |
| Orientation screen (Briefing Room) | look, examine, read |
| The Eye (visible through windows) | look, examine |
| Mineral vein (bluff rock face) | look, examine, search |
| The original landing roster (Records Room) | look, read, search |

### Container Fixtures
Hold objects. Most have lock state. Contents persist.

| Example | Interactions |
|---------|-------------|
| Player locker (Cargo Bay 3) | open, close, lock, unlock, look in, get, put |
| Pharmacy cabinet | open, close, search (requires medical skill or lock skill) |
| Supply crate | open, look in, get |
| Evidence lockbox (Security) | look, examine (open requires clearance) |
| Parts storage (Engineering) | open (requires Engineer profession or skill) |

### Crafting Stations
Convert inputs (from player inventory or attached containers) to outputs. Each station type can process specific recipe categories.

| Station Type | Example Location | Processes |
|-------------|-----------------|-----------|
| `workbench` | Engineering Main Workshop | General fabrication, repairs, assembly |
| `forge` | Engineering — The Forge | Metalworking, smelting, casting |
| `electronics_bench` | Engineering — Electronics Bench | Circuit assembly, comm repair, sensor work |
| `chemistry_station` | Research Station (future) | Compounds, medicines, reactive materials |
| `kitchen_range` | Canteen Kitchen (future) | Food preparation, preservation |
| `smelter` | Impact Center (future, far zone) | Heavy ore processing |
| `press` | Settler Quarter (future) | Textiles, composite materials |

### Environmental Fixtures
Ambient or functional parts of the environment. State may matter.

| Example | Interactions | State |
|---------|-------------|-------|
| Fountain / water source | drink, fill [container] | water level, quality |
| Campfire | light, extinguish, cook [item] | lit/unlit, fuel level |
| Comm terminal | use, access [channel] | on/off, access level |
| Medical scanner | use | on/off (requires power) |
| Colony map display | look, examine | static |
| Emergency beacon | activate | armed/disarmed |

### Toggle Fixtures
Interact to trigger a single effect. Classic lever-and-door pattern, but the effect system is general.

| Example | Interaction | Effect |
|---------|-------------|--------|
| Lever | pull, push | Opens/closes a hidden exit in this or adjacent room |
| Valve | turn | Modifies room atmosphere description, enables/disables hazard |
| Emergency lock | activate | Seals a door exit (requires clearance) |
| Power switch | flip | Toggles on/off state of power-dependent fixtures in room |

### Commerce Fixtures
Interface for the economy system. Not functional until economy is implemented, but the fixture type is defined now.

| Example | Interactions |
|---------|-------------|
| Corporate vending machine | look, buy [item], list |
| Settler vendor stand | look, buy, sell, list, haggle |
| Bulletin board (trade) | look, read, post, remove |
| Auction terminal | look, bid, list, post |
| Supply requisition terminal | look, request (Corporate employees only) |

### Information Fixtures
Provide lore, quests, or world state. Reward examination.

| Example | Interactions |
|---------|-------------|
| Assignment board (North Gate) | look, read, sign up (for resource runs) |
| Research display (Lab) | look, examine, read, access (skill check) |
| Corporate employee database | access (clearance required) |
| Settler handwritten map | look, examine, read (partial — torn) |
| Quantum comms relay (Comms Center) | access, send (registered users), receive |

### Coherence Fixtures
Special to The Eye. Interact with the planet's biotic network in ways that are felt rather than explained.

| Example | Location | Behavior |
|---------|---------|---------|
| Wall Garden | Settler Quarter | Living, responsive to Coherence state. Grows differently when threat level is high. Bonded characters perceive things here. |
| Orthodox focus | Orthodox Hall | Amplifies Coherence signal. Bonding-related interactions that deepen with character's bonding level. |
| Symbiont pod | Future (wilderness) | A naturally occurring Coherence node. Used for deliberate bonding advancement. |
| Coherence-dead zone marker | Impact Center (future) | The inverse — suppresses signal, useful for certain Corporate research. |

Coherence fixtures are not scripted trigger-and-effect systems. They are living fixtures that the game loop updates based on global state (threat level, character bonding depth, faction actions). Their descriptions change. Their available interactions change. They respond to the world.

---

## Fixture Permanence

Every fixture is either **permanent** or **degradable**. This classification drives the devolution floor system described in `design/world-structure.md`.

| Permanence | Devolution behavior | Examples |
|---|---|---|
| **Permanent** | Defines a devolution floor. The Area cannot devolve below `fixture.minimum_stage`. | Building entrance, well, constructed road marker, forge (bolted), permanent signage |
| **Degradable** | No floor. Removed or transitioned to a ruined state when the Area devolves past threshold. | Fire pit, lean-to, cache, rough trail marker, field-built camp structures |

**The floor rule:** Before applying a devolution step, the game loop checks every permanent fixture in the Area. The highest `minimum_stage` among them is the floor. Devolution stops there regardless of traffic or time.

**Degradable removal:** When an Area devolves and a degradable fixture falls below its implicit plausibility threshold, the fixture is removed from the Area and its state is discarded. This is narratively coherent — a fire pit gets reclaimed, a lean-to rots. A building disappearing because nobody walked past it for a month is not coherent, and the permanent classification prevents that.

**Demolition:** A permanent fixture can be deliberately demolished by a player or builder with appropriate permissions. Demolishing the last permanent fixture that anchors the floor removes that floor constraint — the Area can then devolve freely again.

### Minimum Stage by Fixture Type

| Fixture type | Permanence | Minimum stage | Reasoning |
|---|---|---|---|
| Building entrance / structure | Permanent | Trail | Established traffic route required to justify construction |
| Well | Permanent | Footpath | Regular use needed to justify excavation |
| Constructed road surface | Permanent | Road | Is the infrastructure |
| Permanent signage / marker | Permanent | Path | Consistent enough passage to justify marker |
| Forge (bolted, built structure) | Permanent | Trail | Infrastructure implies settlement |
| Field-built camp | Degradable | — | No floor; devolves naturally |
| Fire pit / fire ring | Degradable | — | Reclaimed by vegetation |
| Lean-to / field shelter | Degradable | — | Rots, collapses, vanishes |
| Cache (hidden) | Degradable | — | Looted, buried, forgotten |
| Worn trail marker | Degradable | — | Disappears with devolution to Pristine |

`minimum_stage` for permanent fixtures is defined per fixture instance, not per category — a rough wooden signpost is Degradable, but a metal plaque bolted to a stone post is Permanent with a minimum stage of `Path`.

---

## Room Display Model

This is how fixtures appear to players. There are three distinct text layers:

| Layer | When shown | Field |
|-------|-----------|-------|
| **State line** | On room entry and `look` — one line appended to room output | `state_lines[current_state]` |
| **Look text** | On `look <fixture>` — medium detail | `look` |
| **Examine text** | On `examine <fixture>` — full detail, always contains something look does not | `examine` |

### Room Render Order

When a player enters a room or types `look`:

```
[Room name]
[Room description]

[Fixture state lines — one per visible fixture]
[Object room-look lines — one per object on the floor]

Exits: [directions]
```

Example output for Engineering Bay — Main Workshop:

```
Engineering Bay — Main Workshop
The largest interior space in the settlement and the loudest...

A stone forge stands cold against the north wall.
The fabrication bench runs the length of the east wall, surface scarred by use.
A rack of hanging tools covers the south wall. One slot is conspicuously empty.

A worn leather boot lies here.

Exits: east, north, south, west
```

### State Lines

Every fixture has a `state_lines` map. The renderer picks the line matching the fixture's current state. A fixture with only one state has one entry under `"default"`.

State lines follow the same rules as room descriptions: concrete sensory detail, no interpretation, third-person present. They are one sentence. They do not name adjacent rooms. They do not explain what the fixture does — they describe what it looks like right now.

**Good:** `"The forge blazes here, waves of heat rolling from its mouth."`
**Bad:** `"The forge is active — you can use it to smelt ore."`

### State Transitions and Room Broadcast

When a player action changes fixture state:

1. **Handler broadcasts** a transition message to everyone in the room: `"The forge ignites with a deep whomp as [PlayerA] works the bellows."`
2. **Fixture state is updated**: `state = "active"`
3. **Subsequent enters** automatically see the new state line without needing a broadcast.

Players already in the room when the state changes see the transition broadcast. Players who arrive after see the correct state line. Neither requires an event bus or async injection — state-at-render-time handles it.

For autonomous state changes (timer firing — crafting completes, fire burns out):
1. The game loop tick handler fires the timer
2. Updates fixture state
3. Optionally broadcasts to the room if any players are present: `"The forge settles to a low glow as the heat cycle completes."`

---

## Fixture Data Model

Fixtures are defined in zone JSON under an optional `fixtures` array on each room.

### Base Fixture Schema

```json
{
  "id": "fabrication_bench",
  "names": ["bench", "fabrication bench", "workbench"],
  "category": "crafting_station",
  "permanence": "permanent",
  "minimum_stage": "Trail",
  "connects_to_room": null,
  "state_lines": {
    "default": "The fabrication bench runs the length of the east wall, surface scarred by use."
  },
  "look": "A wide fabrication bench — vise on one end, precision measurement tools racked above it, mounted multi-tool with interchangeable heads at the center. Every tool has a labeled spot. Most of the spots are occupied.",
  "examine": "The bench is bolted to the floor. Someone has scratched a calendar into the surface near the vise — twenty-three marks. The last seven are different, made with a different tool, pressed harder.",
  "read": null,
  "state": {
    "powered": true
  },
  "interactions": {
    "use": { "type": "open_crafting", "station": "workbench" },
    "examine": { "type": "text" }
  },
  "requirements": {
    "use": { "skill": "fabrication", "minimum": 1 }
  },
  "persist_state": false
}
```

### Container Fixture Schema

```json
{
  "id": "player_locker_7delta",
  "names": ["locker", "7-delta", "pod 7-delta"],
  "category": "container",
  "state_lines": {
    "closed": "A numbered metal locker stands here, sealed.",
    "open":   "A numbered metal locker stands here, door hanging open."
  },
  "look": "A numbered metal locker, dented by the hands and hopes of whoever packed it. Yours.",
  "examine": "Stenciled in fading yellow: POD 7-DELTA. Your colony ID has been registered to this locker.",
  "state": {
    "open": false,
    "locked": true,
    "lock_type": "colony_id",
    "owner_character": "character_id_here"
  },
  "interactions": {
    "open": { "type": "container_open", "requires_owner": true },
    "close": { "type": "container_close" },
    "lock": { "type": "container_lock" },
    "unlock": { "type": "container_unlock", "requires_owner": true }
  },
  "persist_state": true
}
```

### Toggle Fixture Schema

```json
{
  "id": "north_gate_lever",
  "names": ["lever", "gate lever", "mechanism"],
  "category": "toggle",
  "state_lines": {
    "closed": "A heavy gate mechanism protrudes from the wall, lever in the down position.",
    "open":   "A heavy gate mechanism protrudes from the wall, lever thrown — the gate is open."
  },
  "look": "A heavy manual gate mechanism — a lever-and-counterweight system that operates the gate independently of power.",
  "examine": "The mechanism is built to last. Whoever designed it did not trust the power grid.",
  "state": {
    "position": "closed"
  },
  "interactions": {
    "pull": {
      "type": "toggle",
      "states": ["open", "closed"],
      "effects": {
        "open": {
          "room_exit": { "direction": "north", "action": "open" },
          "message_room": "The gate mechanism engages with a deep clunk. The north gate swings open.",
          "message_actor": "You pull the lever. The gate swings open."
        },
        "closed": {
          "room_exit": { "direction": "north", "action": "close" },
          "message_room": "The gate mechanism engages. The north gate swings shut.",
          "message_actor": "You push the lever back. The gate closes."
        }
      }
    }
  },
  "requirements": {
    "pull": { "permission": "builder" }
  },
  "persist_state": true
}
```

---

## Interaction Verb Dispatch

When a player types a verb that targets a fixture:

1. Parser identifies the verb and target string
2. Room is checked for fixtures matching the target name
3. If matched: check the fixture's `interactions` map for the verb
4. If the verb exists: check requirements (skill, item, permission, state)
5. If requirements met: execute the interaction handler
6. Broadcast messages: actor message, room message (excluding actor), optional area message

**Supported core verbs:**
`look at`, `examine`, `read`, `use`, `open`, `close`, `lock`, `unlock`, `pull`, `push`, `turn`, `activate`, `deactivate`, `fill`, `drink`, `light`, `extinguish`, `access`, `search`, `get [from]`, `put [in]`

**Commerce verbs** (when economy is implemented):
`buy`, `sell`, `list`, `post`, `remove`, `bid`, `haggle`

Unknown verbs on a fixture return a sensible failure: `"That doesn't seem to do anything."` or `"You don't know how to do that with this."` — never break the fiction with a system error message.

---

## Requirements System

Fixture interactions can require:

| Requirement Type | Example |
|-----------------|---------|
| `skill` | `{ "skill": "fabrication", "minimum": 2 }` |
| `item` | `{ "item": "colony_id_chip", "consume": false }` |
| `permission` | `{ "permission": "builder" }` |
| `faction` | `{ "faction": "corporate", "standing": "employed" }` |
| `bonding_level` | `{ "minimum": 2 }` — for Coherence fixtures |
| `state` | `{ "fixture_state": "powered", "value": true }` — must be powered |
| `owner` | `{ "requires_owner": true }` — player must own this fixture |

Multiple requirements are AND by default. `"any_of"` wrapper supports OR logic.

---

## Crafting Station Integration

A crafting station fixture opens a crafting interface when `use`d. The interface:

1. Shows available recipes for this station type that the player knows
2. Shows what materials the player is currently carrying (or in an attached container)
3. Player selects a recipe
4. System checks: has materials? meets skill minimum?
5. Consumes materials from inventory, begins crafting timer
6. On timer completion: spawns output object in player inventory (or on bench surface)

Recipes are defined separately in `data/recipes/` and are not embedded in fixture definitions. The station fixture only specifies its `station_type`. The recipe system looks up all recipes tagged for that station type.

**Crafting timer:** Defined per recipe. During crafting, the player is not locked — they can move and interact — but leaving the room cancels crafting. Timer is tracked by the game loop, not the session.

**Attached containers:** A crafting station may reference a `storage_fixture_id` — an adjacent container fixture whose contents are available as crafting inputs without requiring the player to move them to inventory first. Example: the forge and an adjacent materials bin.

---

## Fixture State Persistence

Stateful fixtures (`persist_state: true`) write to `WorldSave`:

```
WorldSave.fixture_states: HashMap<FixtureRef, serde_json::Value>
```

`FixtureRef = { zone_id, location_type, location_id, fixture_id }` where `location_type` is `Area` or `Room`. The state value is the fixture's current `state` object, serialized. On startup, fixture states are overlaid onto zone file definitions.

Fixtures with `persist_state: false` reset to their zone file definition on every reboot. Most structural and informational fixtures are not persisted.

**Container contents** are always persisted regardless of `persist_state`, because objects inside a fixture container are real object instances with their own persistence track.

---

## Zone File Integration

Fixtures are an optional array on each room in the zone JSON:

```json
{
  "id": 35,
  "name": "Engineering Bay — Main Workshop",
  "description": "...",
  "exits": { "..." },
  "fixtures": [
    {
      "id": "fabrication_bench",
      "names": ["bench", "workbench", "fabrication bench"],
      "category": "crafting_station",
      "state_lines": {
        "idle":    "The fabrication bench runs the length of the east wall, surface scarred by use.",
        "active":  "The fabrication bench hums with activity, tools in motion.",
        "waiting": "The fabrication bench sits mid-project, components laid out in order."
      },
      "look": "A wide fabrication bench — vise, precision tools, mounted multi-head unit. Every tool has a labeled spot.",
      "examine": "Every tool has a labeled spot. Most of the spots are occupied. Someone has scratched a tally into the surface near the vise.",
      "interactions": { "use": { "type": "open_crafting", "station": "workbench" } },
      "requirements": { "use": { "skill": "fabrication", "minimum": 1 } },
      "persist_state": false
    },
    {
      "id": "tool_rack",
      "names": ["rack", "tool rack", "tools"],
      "category": "structural",
      "state_lines": {
        "default": "A rack of hanging tools covers the south wall — wrenches, clamps, specialty fabrication heads."
      },
      "look": "A rack of hanging tools covers the south wall — wrenches, clamps, specialty fabrication heads.",
      "examine": "The tools are all labeled and arranged by use. One slot is empty. The label reads: PLASMA CUTTER.",
      "persist_state": false
    }
  ]
}
```

### Coherence Fixture — Full Example

The Wall Garden demonstrates how a Coherence fixture uses state lines tied to global game state rather than player action:

```json
{
  "id": "wall_garden",
  "names": ["garden", "wall garden", "plants"],
  "category": "coherence",
  "state_lines": {
    "calm":     "Low alien plants press against the south wall, tended and still.",
    "elevated": "Low alien plants press against the south wall. Something about the way they lean is not quite toward the light.",
    "active":   "The wall garden plants are moving without wind, slow and deliberate.",
    "critical": "The wall garden plants have turned toward the settlement interior, all of them, together."
  },
  "look": "Alien flora in tended rows along the base of the south wall. None of it is from Earth. None of it behaves quite like plants should.",
  "examine": "The plants nearest the wall are different from the ones at the front — older, slower, more deliberate. A bonded character might notice that the ones in back are not moving at all. Not still. Just not moving in a way that looks accidental.",
  "state": { "coherence_driven": true },
  "interactions": {
    "examine": { "type": "text" },
    "tend": { "type": "coherence_action", "requires_bonding": 1 }
  },
  "persist_state": false
}
```

State for Coherence fixtures is derived from global threat level at render time, not stored — so `persist_state: false` and `coherence_driven: true` tells the renderer to map threat level to state_lines keys instead of reading stored state.

---

## Firstfall Fixture Inventory

Fixtures planned for existing Firstfall rooms (not yet written into zone files):

| Room | Fixture | Category |
|------|---------|---------|
| Cargo Bay 3 (Z1/R2) | Player locker (per character) | container |
| Receiving — Briefing Room | Orientation screen | structural / info |
| Receiving — Assignment Board | Assignment board | info |
| Main Street Central | Notice boards (×2) | info |
| Supply Depot — Sales Floor | Vendor terminal | commerce |
| Corp Admin — Records Room | Filing cabinet, data terminals (×2) | container, info |
| Corp Admin — Comms Center | Quantum relay terminal | environmental |
| Engineering — Main Workshop | Fabrication bench | crafting_station |
| Engineering — The Forge | Forge | crafting_station |
| Engineering — Electronics Bench | Electronics workbench | crafting_station |
| Engineering — Salvage Yard | Materials bin | container |
| Medical — Treatment Bay | Medical scanner | environmental |
| Medical — Pharmacy | Pharmacy cabinet | container |
| Canteen — Main Hall | Notice boards, coffee urn | info, environmental |
| Settler Quarter — Wall Garden | Garden beds (×3) | coherence |
| Orthodox Gathering — The Hall | Orthodox focus | coherence |
| Research Station — Main Lab | Sample storage, analysis terminal | container, info |
| Research Station — The Roof | Sensor array | info |
| North Gate | Runner assignment board | info |
| Barracks — Common Room | Unofficial data terminal | info |

---

## Room Writing — Fixture Descriptions

The room description should not describe fixtures in detail — that is the fixture's job. The room description may hint that a fixture exists (the smell of the forge, the glow of a terminal), but the detail lives in the fixture's `look` and `examine` text.

This follows the world-building rule: each fixture is independently intelligible. The `look` text is what the player sees when they look at the fixture. The `examine` text is what they see when they examine it carefully — always more than `look`, often containing something worth finding.

A good fixture `examine` always has something that `look` doesn't: a detail, an anomaly, a clue, a mystery, something that rewards the player for taking the extra step.

---

## OLC Implications

When OLC is implemented, builders need to:
- Add fixtures to existing rooms
- Edit fixture look/examine text
- Set fixture interactions and requirements
- Define crafting station types
- Link container fixtures to their persistence
- Set Coherence fixture parameters

Fixture editing through OLC will require the full interaction schema to be accessible in the builder interface. This is a reason to keep interaction definitions as clean data (JSON) rather than embedding code logic — the builder can edit data without touching code.

---

## LLM Integration

When feeding room evolution to an LLM, fixtures should be included separately from the room description:

```
Room: [current description]
Fixtures present: [list with current look text]
Evolution trigger: [time passing / event / faction change]
Instruction: Update room description and fixture descriptions to reflect the change. Do not add new fixtures. Do not change fixture IDs or interaction schemas.
```

The LLM should not modify fixture interaction schemas — those are code-adjacent. It can modify `look`, `examine`, and `read` text freely. The world-building style guide applies to fixture descriptions with the same rules as room descriptions.

---

## Rust Implementation Shape

```rust
pub enum FixturePermanence {
    Permanent,
    Degradable,
}

pub struct Fixture {
    pub id: String,
    pub names: Vec<String>,
    pub category: FixtureCategory,
    pub permanence: FixturePermanence,
    pub minimum_stage: Option<EvolutionStage>, // floor for devolution; only meaningful when Permanent
    pub connects_to_room: Option<u32>,         // Permanent fixtures only: the Room this fixture enters
    pub state_lines: HashMap<String, String>,  // state key → one-liner
    pub look: String,
    pub examine: String,
    pub read: Option<String>,
    pub state: FixtureState,
    pub interactions: HashMap<String, Interaction>,
    pub requirements: HashMap<String, Requirement>,
    pub persist_state: bool,
}

impl Fixture {
    // Called by the room renderer — picks state line for current state
    pub fn state_line(&self, global: &GlobalState) -> &str {
        let key = if self.state.coherence_driven {
            global.coherence_threat_level.as_str()  // "calm" / "elevated" / "active" / "critical"
        } else {
            self.state.current.as_str()
        };
        self.state_lines.get(key)
            .or_else(|| self.state_lines.get("default"))
            .map(|s| s.as_str())
            .unwrap_or("")
    }
}
```

Room render order in the `look` / entry handler:
1. Room name + description
2. `for fixture in room.fixtures` → `fixture.state_line(global)` (skip if empty)
3. `for obj in room.objects` → `obj.room_look_line()`
4. Exit list

State transitions happen in the command handler that caused the change — update `fixture.state.current`, then broadcast the transition message to the room. The *next* look or enter picks up the new state_line automatically.

---

## Open Questions

- Do fixtures have condition/durability? (A workbench can wear out, need repair — interesting but complex)
- Can fixtures be added to rooms dynamically through gameplay? (A player builds a vendor stand — this implies OLC-level access for players, a later feature)
- Mob interaction with fixtures: can an NPC use a crafting station? (Useful for economy simulation but not MVP)
- Hidden fixtures: fixtures that only appear after a `search` action — stored in zone file with `visible: false`, state changes to `visible` on discovery
