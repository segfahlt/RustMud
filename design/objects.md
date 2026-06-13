# Objects (Items)

Status: **Placeholder** — awaiting theme/lore decisions before detailed design

---

## What This Is

The object system covers all tangible items that exist in the world: weapons, armor, gear, consumables, containers, currency, quest items, crafting components, and anything a player can pick up, carry, drop, or use. Objects exist in rooms, in mob inventory/equipment, and in player inventory/equipment.

---

## Planned Sub-systems

- **Object definitions** — static templates defined in zone files; instances spawned from templates
- **Inventory** — players and mobs carry a list of object instances
- **Equipment slots** — worn/wielded items occupy named slots (head, body, weapon, off-hand, etc.)
- **Object flags** — no-drop, no-sell, quest, cursed, hidden, etc.
- **Containers** — objects that hold other objects (bags, chests, corpses)
- **Corpses** — spawned on mob/player death, decay over time, contain loot
- **Object persistence** — objects in rooms persisted in `WorldSave.rooms`

---

## Key Design Questions

- What are the equipment slots? (theme-dependent — fantasy armor vs. something else)
- How are objects identified? (template_id + instance_uuid? or template only?)
- Do objects have condition/durability?
- How does item weight/encumbrance work, if at all?
- Are magic items distinguished from mundane at the type level or via flags?
- How do containers interact with weight limits?
- What is the loot drop format in zone files?

---

## Dependencies

- Theme/lore decisions (what kinds of items exist)
- Player stats (damage ranges, armor values reference stats)
- Races/classes (equipment restrictions)
- Crafting (recipe inputs/outputs are objects)
- Economy (items have base_value for shop pricing)
