# Crafting

Status: **Placeholder** — core system, needs design after objects and classes are defined

---

## What This Is

Crafting lets players create items from component materials. It is one of the primary economic drivers — crafted items fuel player vendor stands and the auction house. A robust crafting system gives non-combat players a meaningful progression path and creates interdependency between player archetypes (gatherers, crafters, adventurers).

---

## Design Pillars

- **Meaningful skill investment**: crafting skills require deliberate leveling; a warrior who dabbles gets inferior results to a dedicated crafter
- **Component sourcing creates content**: rare crafting materials come from dangerous mobs, remote zones, or player trade — not just shops
- **Output feeds the economy**: crafted items are competitive with or superior to shop-bought gear
- **Failure is possible but not punishing**: failed crafts waste materials but don't destroy equipment; exceptional successes produce better-than-normal items

---

## Planned Crafting Disciplines

Exact names are **theme-dependent**, but rough archetypes:

| Discipline | Produces |
|------------|---------|
| Smithing | Weapons, armor, metal tools |
| Leatherworking | Light armor, bags, saddles |
- Alchemy | Potions, poisons, reagents |
| Cooking | Food/drink that provides buffs |
| Enchanting | Adds magical properties to existing items |
| Jewelcrafting | Rings, amulets, gems |
| Tailoring | Cloth armor, robes, bags |
| Engineering / Tinkering | Gadgets, traps, lockpicks — theme-dependent |

---

## Recipe System

Recipes define inputs → output. They are either:
- **Known from the start** (basic recipes)
- **Learned** from recipe items found in the world
- **Discovered** by experimenting with components (optional mechanic)

```json
{
  "recipe_id": "iron_longsword",
  "discipline": "smithing",
  "skill_required": 40,
  "components": [
    { "item_id": "iron_ingot", "qty": 3 },
    { "item_id": "leather_strip", "qty": 1 }
  ],
  "output": { "item_id": "longsword_iron", "qty": 1 },
  "tool_required": "forge",
  "fail_chance_at_minimum_skill": 0.30,
  "exceptional_chance_at_max_skill": 0.10
}
```

`tool_required` means the player must be in a room that has that fixture (a forge, an alchemy bench, a cooking fire).

---

## Key Design Questions

- How are crafting skills leveled? (practice points, use-to-improve, trainers)
- Is there a dedicated Crafter class, or is crafting open to everyone with investment?
- Can a single character max all crafting disciplines, or must they specialize?
- Do exceptional crafts have visible markers on the item? ("+1 Iron Longsword", or "masterwork")
- Are crafting recipes tradeable between players?
- How does enchanting interact with already-crafted items? (applied after, or part of the recipe?)
- Component gathering: are raw materials found only via combat/exploration, or also harvestable from the environment (mining, herbalism, skinning)?

---

## Dependencies

- Objects (recipes produce and consume object instances)
- Fixtures (crafting stations are fixtures in rooms)
- Classes/skills (crafting disciplines gate on skill level)
- Economy (crafted items enter the vendor/auction ecosystem)
- Environment (harvesting raw materials may require environmental interaction)
