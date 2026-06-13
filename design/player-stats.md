# Player Stats and Attributes

Status: **Placeholder** — awaiting theme, race, and class decisions

---

## What This Is

The stat system defines the numerical attributes that govern a character's capabilities: combat effectiveness, carrying capacity, spell power, social influence, crafting aptitude, etc. Stats are set at character creation (influenced by race and class), grow with level, and are modified by equipment and buffs.

---

## Likely Stat Categories

### Primary Attributes
Core values set at creation, used to derive secondary stats. Classic examples:
- Strength, Dexterity, Constitution, Intelligence, Wisdom, Charisma
- Exact names and count are **theme-dependent**.

### Secondary / Derived Stats
Calculated from primary attributes + level + equipment:
- Max health (Constitution-based)
- Max mana/energy/power (Intelligence or Wisdom-based — name is theme-dependent)
- Attack bonus, damage bonus (Strength or Dexterity-based)
- Armor class / defense
- Hit rate, dodge rate
- Carry weight (Strength-based)
- Experience to next level

### Soft Stats / Skills
Non-combat capabilities that grow through use or point allocation:
- Crafting skills (blacksmithing, alchemy, etc.)
- Social skills (barter, persuasion)
- Exploration skills (tracking, search)
- Magical disciplines (theme-dependent)

---

## Key Design Questions

- How many primary stats? (fewer = simpler; classic is 6)
- What are they called? (depends on theme — a sci-fi MUD uses different names than fantasy)
- Does Constitution affect max health linearly or with a modifier table?
- Are stats rolled at creation (classic), point-buy, or assigned by race/class template?
- Do stats increase on level-up (player chooses) or only via equipment?
- Is there a stat cap per race/class?
- How does the stat system interact with the class system? (Some classes need minimum stats)
- Experience: kill XP only, or also quest XP, crafting XP, exploration XP?

---

## Dependencies

- Theme/lore (what the stats are called, what the power system is named)
- Races (racial stat modifiers)
- Classes (class-primary stats, stat caps)
- Objects (equipment stat bonuses)
- Combat (stats feed into hit/damage calculations)
- Environment manipulation (mana/power pool)
