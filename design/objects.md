# Object System — The Eye

Status: **Draft** — framework decisions made, ready for implementation planning

---

## What Objects Are

Objects are tangible things that exist in the world and can move through it. They live in rooms, in mob inventories, in player inventories, and inside containers. They do not stay fixed to a location — that is a fixture's job.

Every object in the world is an **instance** of a **template**. The template describes what the object is. The instance describes the specific copy of it that exists right now, and what state it is in.

---

## Template vs Instance

### Templates (Prototypes)
Defined in `data/objects/templates/*.json`. A template describes a category of thing — not a specific copy of it. All instances of a template share the same base properties. Templates are static and loaded at startup.

```json
{
  "id": "hunting_knife",
  "names": ["knife", "hunting knife"],
  "short": "a hunting knife",
  "room_look": "A hunting knife lies here.",
  "description": "Quality steel, full tang, worn leather handle. It has been used.",
  "category": "weapon",
  "subcategory": "blade",
  "weight": "light",
  "value": 40,
  "flags": ["wieldable"],
  "wear_slot": "wielded",
  "combat": {
    "damage_type": "piercing",
    "damage_min": 3,
    "damage_max": 8,
    "speed": "fast"
  }
}
```

### Instances
Instances are created when an object enters the world (spawned in a room, crafted, awarded). They track state that diverges from the template.

```json
{
  "instance_id": "uuid-...",
  "template_id": "hunting_knife",
  "condition": "worn",
  "uses_remaining": null,
  "contents": [],
  "custom_name": null,
  "custom_desc": null,
  "bonded_to": null,
  "flags_override": []
}
```

`custom_name` renames the item ("my father's knife"). `custom_desc` stores a player-written personal description, used for Earth items from character creation (the photograph they described, the letter they wrote).

Instances reference their template for everything not listed. The template is the single source of truth for base properties.

---

## Object Categories

| Category | Examples |
|----------|---------|
| `weapon` | knives, rifles, bows, batons, improvised |
| `armor` | vest, helmet, gloves, boots, full suit |
| `tool` | multi-tool, climbing gear, scanner, drill bits |
| `consumable` | rations, stims, medicine, ammunition |
| `component` | ore, ingots, circuit boards, chemical compounds |
| `container` | bags, pouches, crates, specimen cases |
| `data` | data chips, documents, books, maps |
| `currency` | corporate credits chip, trade goods used as barter |
| `trade_good` | Earth alcohol, quality samples, specialty goods |
| `quest` | specific items with quest flags, tracked by the system |
| `bonded` | symbiont-integrated items, grows with character |

---

## Object Flags

Flags modify how the object interacts with game systems. A template may set base flags; instance flags_override can add or remove individual flags.

| Flag | Meaning |
|------|---------|
| `NO_DROP` | Cannot be dropped — removed from inventory only via system event |
| `NO_SELL` | Cannot be sold to shops or vendors |
| `NO_GIVE` | Cannot be transferred to another player |
| `NO_TRADE` | Cannot be used in player-to-player trade |
| `EARTH_ORIGIN` | Came from Earth. Irreplaceable. Can rarely or never be repaired locally. |
| `QUEST` | Quest item. Tracked by name in quest logs. Cannot be destroyed. |
| `STACKABLE` | Multiple instances of this template merge in inventory (ammo, rations, ore) |
| `CONSUMABLE` | Has uses_remaining; destroyed when uses reach 0 |
| `CONTAINER` | Can hold other objects |
| `TWO_HANDED` | Occupies both `wielded` and `held` equipment slots |
| `LIGHT_SOURCE` | Emits light (future: affects dark room visibility) |
| `HIDDEN` | Does not appear in room look — requires examine/search to find |
| `BONDED` | Symbiont-integrated. NO_DROP + NO_GIVE. Grows with the character. |
| `CORPORATE_ISSUE` | Standardized Corporate equipment. Shops may stock it when ships arrive. |
| `SETTLER_MADE` | Locally crafted. Variable quality. |

---

## Weight / Encumbrance

Weight uses a category system rather than exact numbers. Encumbrance thresholds are determined by the character's physical conditioning stat (to be defined in `player-stats.md`).

| Weight Class | Examples | Carry Impact |
|-------------|---------|-------------|
| `tiny` | data chip, coin, small tool | Negligible |
| `light` | knife, handgun, medical kit, canteen | Normal |
| `medium` | rifle, armor vest, tool kit, backpack | Counts toward load |
| `heavy` | heavy weapon, full armor, generator | Significant load |
| `bulky` | climbing rig, full shelter kit, large container | Movement penalty even if not over weight limit |

Players can carry up to their conditioning-determined limit. Over that limit: movement speed reduced, some actions unavailable. Containers add their weight plus contents' weight.

---

## Equipment Slots

Characters have named equipment slots. Only one item per slot. Worn items provide passive effects; wielded items are used in combat or actions.

| Slot | Examples |
|------|---------|
| `head` | helmet, headlamp, goggles |
| `face` | rebreather, visor, respirator |
| `body` | armor vest, jacket, survival suit |
| `hands` | gloves, gauntlets |
| `waist` | belt, holster, tool harness |
| `legs` | pants, knee pads |
| `feet` | boots |
| `back` | backpack (primary container slot) |
| `neck` | pendant, respirator loop |
| `wrist` | comm unit, medic cuff |
| `wielded` | primary weapon |
| `held` | off-hand weapon, tool, scanner, flashlight |

`TWO_HANDED` items occupy both `wielded` and `held`. The backpack slot is special: it is a container that adds carry capacity.

---

## Condition / Durability

All objects have a condition that degrades with use. Condition affects performance (weapons do less damage, armor absorbs less) and sale value.

| Condition | Description |
|----------|-------------|
| `pristine` | New or perfectly maintained. Full stats. |
| `good` | Normal use. Full stats. |
| `worn` | Visible wear. Slight stat reduction. |
| `damaged` | Significant degradation. Moderate stat reduction. Repair urgently. |
| `broken` | Non-functional. Cannot be used until repaired. |

Degradation rate depends on object type and use frequency. Combat weapons degrade faster than stored tools.

**Repair** requires: the appropriate crafting station (workbench for most, forge for metalwork), the Mechanical Repair or relevant skill, and sometimes a component.

**Earth items** cannot be repaired with locally available materials — or only poorly. The right Earth-origin spare parts, if you have them, might work. Otherwise, condition degrades permanently.

---

## Special Object Types

### Earth Items
Brought from Earth aboard the shuttle. Defined in the personal items list from character creation. Properties:
- Always carry the `EARTH_ORIGIN` flag
- No template equivalent exists in the game world (the template IS unique, or instance-only)
- Cannot be restocked from any in-game source
- Degrade normally but repair options are extremely limited
- High social and trade value among settlers — everyone knows what Earth goods are

### Corporate Gear
Standardized issue, quality equipment. Properties:
- Carry `CORPORATE_ISSUE` flag
- Stocked at the Supply Depot when ships arrive (periodic availability)
- Consistent stats across all instances — Corporate doesn't tolerate variation
- Repair possible at Medical or Engineering facilities
- Selling back to Corporate below market rate

### Settler-Made Items
Crafted from local materials. Properties:
- Carry `SETTLER_MADE` flag
- Variable quality depending on crafter's skill and material quality
- High-skill crafters can produce gear equivalent to Corporate issue
- Some local materials produce effects Corporate gear cannot replicate
- Condition starts at `good` (skilled crafter) or `worn` (hasty)

### Symbiont-Bonded Items
Items that have been integrated with the Coherence through extended contact with a deeply bonded character. Properties:
- Carry `BONDED` flag — cannot be dropped, given away, or traded
- A bonded item is specific to one character
- Gains properties as the character's bonding deepens
- If the character dies (permadeath variant) or deliberately releases the bond, the item decoheres — typically becomes `broken`
- These are the rarest and most powerful items in the late game

### Data Items
Books, data chips, maps, documents, printed reports. Properties:
- Can be `read` as an action
- May contain in-game information, lore, recipes, or quest content
- Some are unique (one exists in the world); others are copies
- Data chips may be accessed via a terminal fixture for expanded content
- Knowledge data chips from Earth carry permanent reference bonuses when read

---

## Object Instance Persistence

Object instances need to survive reboots. Storage locations:

| Where | Persistence Location |
|-------|---------------------|
| In an Area | `WorldSave.area_objects: HashMap<AreaRef, Vec<ObjectInstance>>` |
| In a Room | `WorldSave.room_objects: HashMap<RoomRef, Vec<ObjectInstance>>` |
| In player inventory | `CharacterSave.inventory: Vec<ObjectInstance>` |
| Equipped on player | `CharacterSave.equipment: HashMap<WearSlot, ObjectInstance>` |
| In a container in an Area or Room | Nested in the container instance in the relevant map above |
| In a fixture container | `WorldSave.fixture_contents: HashMap<FixtureRef, Vec<ObjectInstance>>` |

Objects in rooms decay: dropped items disappear after a configurable duration (except `QUEST` items, which persist until picked up). Decay timer is stored in the instance.

---

## Template Storage

Templates are loaded from `data/objects/templates/`. Files can be organized by category:

```
data/objects/templates/
  weapons.json         — all weapon templates
  armor.json           — all armor/clothing templates
  tools.json           — tools and utility items
  consumables.json     — food, medicine, stims, ammo
  components.json      — crafting materials, ore, ingots
  earth_items.json     — Earth-origin items from character creation
  corporate_gear.json  — Corporate standard issue
  data_items.json      — books, chips, documents
```

Zone-specific unique items can be defined inline in the zone file under a `loot_templates` key, inheriting the same schema.

---

## The Cargo Locker Scene (Integration Point)

When a new player opens their locker in Cargo Bay 3:
1. System reads their profession → loads profession kit template list → spawns instances
2. System reads their personal item choices → spawns instances of selected Earth items
3. All instances are placed in the locker fixture (a container fixture at `{ zone_id: 1, location_type: Room, location_id: "cargo_bay_3", fixture_id: "player_locker_7delta" }`)
4. Player uses `open locker`, `look in locker`, `get [item] from locker`

Earth item instances carry the player's custom descriptions (the photograph they wrote, the letter they described). These are stored in `ObjectInstance.custom_desc`.

---

## Open Questions

- Ammunition: per-shot tracking or magazine-level? (affects combat design)
- Currency: credit chip as an object instance, or a flat integer on CharacterSave?
- Container weight: does a container add its own weight to its contents' weight?
- Object aging: do items in rooms that nobody picks up for months deteriorate? (interesting emergent effect but complex)
- Crafted items: do they record who made them? (provenance could be interesting for economy)
