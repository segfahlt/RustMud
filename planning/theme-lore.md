# Theme and Lore

Status: **Discussion needed** — this doc must be filled in before detailed design of races, classes, environment manipulation, objects, or world geography can proceed. Almost every other design doc has a dependency on this one.

---

## Why This Comes First

Every design decision downstream of "what kind of world is this?" depends on the answers here. Races, classes, power systems, item names, zone aesthetics, NPC dialogue, crafting disciplines, economy flavor — all of it flows from theme. Building without a theme produces a generic mud. Building with a strong theme produces something people remember.

---

## Questions to Settle

### Genre / Setting

What is the fundamental nature of this world?

- High fantasy (elves, dwarves, dragons, arcane magic)?
- Dark fantasy (grim, morally grey, corrupted gods)?
- Sci-fantasy (technology and magic coexist)?
- Post-apocalyptic fantasy (a fallen civilization, ruins of something greater)?
- Mythpunk (real-world mythology remixed)?
- Something original that doesn't map cleanly to a genre?

### The World's Central Conflict

Every memorable MUD has a tension at the heart of its world that motivates why players are out there adventuring. What is the conflict here?

Examples:
- A war between civilizations
- An encroaching darkness / corruption spreading through the land
- Gods are dead or absent — power vacuum
- Two factions fighting over a resource, ideology, or artifact
- The world itself is dying / changing
- A mystery: something happened in the past and nobody knows what

### Power System

What is the in-world name and explanation for "magic" (or whatever the environment manipulation system is)?

- Classical arcane magic (studied, precise, dangerous if misused)?
- Divine power (granted by gods — what gods, what do they want)?
- Psionics (mental discipline — rare, feared, or common)?
- Elemental manipulation (bound to the natural world)?
- Something technology-adjacent?
- Multiple parallel systems that have history/conflict between them?

### Races

What sapient peoples exist in this world, and what is their relationship to each other?

- Are the classic fantasy races (elf/dwarf/human/etc.) present, absent, or transformed?
- Are there original races that emerge from the world's specific history?
- What is the political/historical relationship between races?
- Are any races player-hostile by default (pkill races)?

### Tone

Where does this world sit on these spectrums?

- Light ↔ Dark (hopeful adventure vs. grim survival)
- Serious ↔ Playful (epic stakes vs. irreverent humor)
- Grounded ↔ Mythic (low-magic realism vs. world-shaking powers)
- Familiar ↔ Original (comfort of the known vs. surprise of the new)

### Inspirations

What MUDs, games, books, or films capture something of what you're going for?

- You mentioned "The Forest's Edge" as your favorite MUD — what specifically did you love about it?
- Are there other works whose world or systems you want to draw from?

---

## Once Decided, This Doc Becomes

A **World Bible** — the reference document that answers "does this fit the world?" for any design question. It should eventually include:

- World name and cosmology
- History / timeline (the events that shaped the present)
- Geography (continent/region overview, zone placement logic)
- Factions and their relationships
- Pantheon (gods, their domains, their attitudes toward mortals)
- The power system and its in-world rules and limits
- Race descriptions (appearance, culture, history, relationship to others)
- Class archetypes and their in-world role
- The "feel" of everyday life (what do common people do? what do they fear?)
