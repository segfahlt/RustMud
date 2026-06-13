# Classes

Status: **Placeholder** — awaiting theme/lore decisions

---

## What This Is

Classes define a character's role and advancement path. They determine which skills and abilities a character can learn, how fast they advance, what equipment they can use, and their playstyle identity. Class is chosen at creation and is permanent until remort (which may allow class change or multi-classing).

---

## What Classes Typically Provide

| Feature | Example |
|---------|---------|
| Primary stat focus | Warrior uses Strength; Mage uses Intelligence |
| Hit points per level | Warriors gain more HP than Mages |
| Mana/power per level | Casters gain more than fighters |
| Skill list | Which skills are available, at what level |
| Equipment restrictions | Mages can't wear heavy armor |
| Combat style | Melee, ranged, spells, stealth |
| Advancement rate | XP required per level may differ by class |
| Special abilities | Backstab, lay on hands, turn undead — theme-dependent |

---

## Key Design Questions

- What classes exist? Entirely **theme-dependent**.
- Classic quartet (warrior/mage/cleric/thief) or something original?
- Are there hybrid classes (paladin = warrior + cleric)?
- Is multi-classing supported? At creation, or through remort?
- Does remort change your class, or stack on top of it?
- Are some classes unlocked via race (e.g., only elves can be arcane archers)?
- Are there prestige/subclasses unlocked at higher levels or via quests?
- How is the skill system structured — level-gated, practice points, use-to-improve?
- Does class define the power system used (mana vs. stamina vs. something else)?

---

## The Remort System
Remort (a MUD tradition) is when a max-level character resets to level 1 in exchange for permanent bonuses and access to content unavailable to non-remort characters. This interacts heavily with classes:
- Can you change class on remort?
- Do you keep some abilities from your previous class?
- Are there remort-only classes?
- Does the `Remort` permission gate remort-exclusive content?

---

## Dependencies

- Theme/lore (what the power/role archetypes are in this world)
- Races (race/class restrictions)
- Player stats (class determines which stats matter most)
- Environment manipulation (which class accesses which power system)
- Objects (class-restricted equipment)
- Remort system (class × remort interactions)
