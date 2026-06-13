# Fixtures

Status: **Placeholder** — new concept, needs design before implementation

---

## What This Is

Fixtures are objects that are part of a room but are not items — they cannot be picked up. They exist to enrich the environment, reward player curiosity, and enable non-combat interactivity. Examples: a bookshelf that can be read, a lever that opens a hidden door, a painting that hints at lore, a fireplace that can be lit, a locked chest that is part of the room itself.

This is distinct from the object/item system. Fixtures are defined in the room definition in the zone file, not in a separate item template pool.

---

## Planned Interactions

| Interaction | Example |
|-------------|---------|
| `look at <fixture>` | Returns an extended description beyond the room desc |
| `examine <fixture>` | More detailed — may reveal hidden info, secret exits |
| `read <fixture>` | For books, signs, inscriptions |
| `pull / push / turn <fixture>` | Triggers: opens exits, spawns mobs, fires events |
| `search <fixture>` | May reveal hidden items or exits with a skill/stat check |

---

## Key Design Questions

- How are fixtures stored in zone JSON? (array on the room object, with `name`, `description`, `examine_text`, `trigger?`)
- What is the trigger system? Simple flags (open_exit: north) or a scripting hook?
- Can fixtures have state? (lever up/down, door open/closed, torch lit/unlit) — if so, state needs to live in `WorldSave.rooms`
- Do fixtures persist manipulation across reboots? (e.g., a pulled lever stays pulled)
- Can builders create fixtures via OLC?
- Hidden exits: are they a fixture trigger, or a separate exit flag on the room?
- Quest triggers: how does a fixture fire a quest event? (emit an event to the game loop?)
- Can fixtures be destroyed or permanently changed?

---

## Fixture Definition Sketch (zone JSON)

```json
"fixtures": [
  {
    "id": "old_bookshelf",
    "names": ["bookshelf", "shelf", "books"],
    "look": "A tall oak bookshelf lines the wall, packed with dusty volumes.",
    "examine": "One book is titled 'Histories of the Iron Mages.' The spine is cracked with use.",
    "read": "The opening page reads: 'In the year of the Sundering...'",
    "trigger": null
  },
  {
    "id": "iron_lever",
    "names": ["lever"],
    "look": "An iron lever protrudes from the wall, crusted with age.",
    "examine": "It looks like it could be pulled.",
    "trigger": { "action": "pull", "effect": "open_exit", "direction": "north", "message": "Stone grinds as a hidden door swings open to the north." }
  }
]
```

---

## Dependencies

- World model (fixtures live on `Room`)
- OLC (builders need to create/edit fixtures)
- WorldSave (stateful fixtures need persistence)
- Quest system (triggers as quest hooks) — future
