# Environment Manipulation

Status: **Placeholder** — theme-dependent, design after lore is established

---

## What This Is

Environment manipulation covers any system where a character uses an internal power resource to affect the world around them — spells, psionics, ki, technology, mutations, divine favor, or other thematic equivalents. The name "environment manipulation" is intentionally generic; the in-game name, flavor, and mechanics are **entirely theme-dependent**.

This is a catch-all design doc for all power systems. Once theme is decided, this may split into multiple docs (e.g., `design/magic.md`, `design/psionics.md`).

---

## Power System Archetypes

| Name | Flavor | Resource |
|------|--------|---------|
| Magic / Spells | Arcane or divine | Mana |
| Psionics | Mental force | Psi points / focus |
| Ki / Chi | Internal life force | Ki |
| Technology | Gadgets, charges | Power cells / crafted items |
| Divine | Prayer, faith | Favor / conviction |
| Mutations | Biological | Cooldowns, health cost |
| Elemental | Nature-bound | Mana or elemental charges |

We may have one system, multiple parallel systems (different classes use different ones), or a hybrid.

---

## Interaction Categories

Regardless of flavor, effects fall into these functional buckets:

| Category | Examples |
|----------|---------|
| Combat offense | Damage spells, debuffs, stuns |
| Combat defense | Shields, heals, buffs |
| Utility | Teleport, detect invisible, water breathing, light |
| Environmental | Create fire, move walls, summon creatures, weather change |
| Social | Charm, fear, truthsense |
| Crafting | Enchant item, brew potion, transmute material |
| Information | Identify item, detect alignment, scry |

---

## Key Design Questions

- What is the primary power system in this world? (Depends entirely on theme)
- Is there one system or multiple parallel ones?
- What is the resource called and how is it recovered? (Rest, time, consumables, meditation)
- Are abilities selected from a list (spell memorization), always available (cooldown-based), or learned permanently and always castable?
- How does the environment respond to manipulation? (A `create fire` spell should light torches, warm rooms, and be a fire hazard)
- Are there counterspells / resistance systems?
- Does the power system have in-world costs beyond the resource? (Verbal components, material components, concentration)
- Is there a "backlash" or wild magic mechanic for failure at high difficulty?
- How do innate racial abilities fit — same system or separate?

---

## Environmental Effects (the interesting part)

This is where the system gets unique. Classic MUDs treat spells as combat buttons. A more interesting model: spells change the environment.

- `fireball` cast in a library could set the room on fire
- `freeze` could create ice on the floor (movement penalty, fixture)
- `earthquake` could collapse exits temporarily
- `summon water` could flood a room
- Room state (on fire, flooded, frozen) persists in `WorldSave.rooms` and affects all players in it

This interacts heavily with the **fixtures** system — fire, ice, and water could be temporary fixtures with their own look/examine descriptions and interaction triggers.

---

## Dependencies

- Theme/lore (what the power system is and why it exists in this world)
- Player stats (power pool size, regeneration rate)
- Classes (which classes access which systems)
- Races (innate abilities)
- Fixtures (environmental effects create/modify temporary fixtures)
- Combat (offensive abilities integrate with combat loop)
- Crafting (enchanting, potion-making are crafting-adjacent)
