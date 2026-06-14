# World Building Guide — The Eye

Status: **Active** — this is the authoritative style reference for room writing. All content agents should receive this document as context before generating or modifying rooms.

---

## Core Rule

Each room must be independently intelligible. Assume the player has no context about adjacent rooms, the zone's geography, or how they arrived. Describe what exists in this space through concrete sensory details. Convey the zone's atmosphere and progression through environmental accumulation, not through direct reference to other areas.

---

## Room Anatomy

A complete room description covers these elements, in roughly this order:

1. **Space** — What is this room or area? Size, shape, construction, light source and quality.
2. **Contents** — What exists here? Equipment, furniture, objects, people in place, evidence of use.
3. **Atmosphere** — What does it feel like to be here? Temperature, sound, smell, emotional weight.
4. **Exit cues** *(optional)* — Physical or sensory hints of accessible directions. Never name the destination room.

Not every room needs all four. A storage room may be space + contents only. A transit corridor may be atmosphere + exit cues. Match depth to significance.

---

## Exit Description Rules

The exit list handles navigation. The description handles immersion. These are separate systems.

**Do not:**
- Name adjacent rooms in the description: `"Cargo Bay 3 is to the east."`
- Name destinations: `"The bluff path begins to the west."`
- List exits as a closing sentence: `"The forge is north, the electronics bench is south, the street is east."`

**Do:**
- Describe what is physically visible or sensed in that direction: `"Heat and the ring of hammered metal come from the north."`
- Reference physical features, not room names: `"A heavy gate mechanism stands to the south."` not `"The West Gate is south."`
- Let sensory accumulation imply the exit: `"The smell of real coffee drifts from the west."` is better than `"The canteen is west."`

When an exit must be mentioned explicitly, describe it physically: `"A pressure door stands east."` — not `"The shuttle corridor is east."`

---

## Sensory Priority

Describe in this rough priority:
1. The dominant visual — what the eye catches first
2. What draws attention second (contrast, movement, anomaly)
3. What the body registers (temperature, air quality, pressure)
4. Sound
5. Smell
6. What the gut says about this place

The planet is not explained; it is observed. Strange things are described exactly as perceived, not interpreted. `"Something moves in the water below that is not a fish."` not `"An alien creature stirs beneath the surface."`

---

## Tone and Voice

- **Present tense.** The room exists now.
- **No second-person "you" in descriptions** except at moments of deliberate narrative weight (the Disembarkation Ramp, the Cryo-Bay revival). Use sparingly.
- **Concrete specifics over abstract atmosphere.** `"the smell of glycerol and recycled oxygen"` beats `"a clinical smell."` `"water the color of strong tea"` beats `"dark water."`
- **One precise word beats three vague ones.** No adjective stacking.
- **No interpretation.** Do not tell the player how to feel. Describe; let them feel it.

---

## The Corporate vs. Settler Aesthetic

**Corporate spaces:**
- Clean lines, gray prefab, functional
- Maintained but not warm — built to spec, not to live in
- Color-coded, labeled, regulated
- Feels like a logistics operation, not a home

**Settler spaces:**
- Salvaged, layered, added to over time
- Things don't quite match — different materials, improvised repairs
- Shows evidence of human presence: wear patterns, personal effects, uneven but intentional
- Warmer, more complicated, more alive

**Transition zones** (like Main Street north end) show the line where Corporate ends and settler begins through material contrast, not statement.

---

## The Coherence

The quantum biotic network is never named or explained in room descriptions. It manifests as:

- Things that move with more coordination than individual organisms should show
- Sounds that carry a pattern without identifiable source
- A sense of being observed without a visible observer
- Air that feels different in ways that resist description
- Plants or growth in configurations that don't match what random evolution would produce
- An absence of the usual random noise of unconnected life

The player accumulates understanding through repeated observation, not exposition. Never write `"you can feel the Coherence here."` Write what you can actually perceive.

**Coherence gradient:** Weakest at the impact center (Coherence-dead zone), strongest in undisturbed deep terrain and in certain settler-tended spaces. Firstfall interior suppresses it — infrastructure and human presence dampen the signal. The Wall Garden, the Orthodox Hall, and similar spaces are exceptions.

---

## The Eye

Always visible when outside, provided the sky is clear. Never explain it in a room description. Reference its position as a natural feature, the way you'd reference the sun — but with the weight of something that does not behave like a sun.

- **First view** (Disembarkation Ramp): This is the player's only chance at a full "first time" moment. Write it fully.
- **Subsequent outdoor rooms**: Brief acknowledgment — position, quality of light it casts, that it is there. Not a repeated dramatic moment.
- `"The Eye hangs where it always does."` — for rooms where it's visible but unremarkable now.
- Settlers don't look up at it. They know which direction it is without checking.

---

## Environmental Accumulation

Atmosphere and worldbuilding travel through repetition across rooms, not through statements. Examples:

- **Scarcity:** Three rooms where shelves are half-empty or equipment shows wear establishes the colony's resource pressure without a single line of exposition.
- **Mortality:** The missing persons board in Security, the empty bunks in the Barracks, the original landing roster with dates — these compound. Never state that many people have died.
- **The Coherence building:** Each zone further from Firstfall has more signs of ecosystem coordination. Let the gradient accumulate.
- **Faction tension:** Signs of Corporate control in one room, signs of settler improvisation around or under it in the next.

---

## What Not to Do

| Wrong | Right |
|-------|-------|
| `"This room connects the cryo-bay to cargo storage."` | Describe what the room IS, not its relationship to others |
| `"After arriving from the south gate..."` | Never reference how the player arrived |
| `"The South Gate is to the north."` | `"A security gate and checkpoint are to the north."` |
| `"This is the central hub of Firstfall."` | Show centrality through foot traffic, notice boards, cross-faction presence |
| `"The kitchen is north, the porch is west."` | Sensory cues pointing in directions, or nothing — exits handle navigation |
| `"You feel uneasy here."` | Describe what produces the unease |
| `"The alien creature..."` | `"Something that is not a fish..."` |

---

## Room Output Order

When a player enters a room or types `look`, output is rendered in this order:

```
[Room name]
[Room description]
                          ← blank line
[Fixture state lines]     ← one per visible fixture, state-aware
[Object room-look lines]  ← one per object on the floor
                          ← blank line
Exits: [directions]
```

The **room description** sets the space and atmosphere. It does not describe fixtures in detail — at most a sensory hint that a fixture exists ("heat rolling from the north wall"). The **fixture state line** is where the fixture announces itself. The player can then `look at <fixture>` for more.

Fixture state lines are single sentences written to the same standard as room descriptions: concrete, sensory, present tense, no interpretation. They change based on the fixture's current state — a cold forge and a running forge have different lines. Coherence fixtures have state lines that change with global threat level.

---

## Room Length

- **Transit rooms** (corridors, paths): 2–4 sentences. They exist to move through, not to study.
- **Key rooms** (faction hubs, major NPCs, significant locations): 5–8 sentences. Density earns attention.
- **Critical narrative rooms** (Disembarkation Ramp, Assignment Board): As long as the moment demands, no longer.

---

## Evolving Rooms (ML/LLM Integration Context)

When feeding a room to an LLM for evolution, aging, or expansion, provide:

1. **This document** as system context / style guide
2. **The room's current description** (verbatim)
3. **Zone state**: age, population, current Coherence threat level
4. **Relevant world events**: faction changes, resource surges or shortages, Corporate pressure level
5. **Instruction type**: `evolve` (time passing), `age` (deterioration), `expand` (add a room), `react` (respond to a player event)

The LLM should produce changes that reflect time and event, not rewrites that change the room's architectural purpose. A room that was a supply depot when the colony was new should still be a supply depot when it's three years old — but the shelves may be different, the personnel may have changed, and the walls may have settler additions.

**Do not** ask the LLM to rewrite descriptions from scratch without providing the current description. Context continuity matters.

---

## Zone Writing Checklist

Before committing a zone, verify:

- [ ] Every room is independently intelligible without knowing adjacent rooms
- [ ] No room names an adjacent room in its description
- [ ] Exit hints use sensory/physical language, not destination names
- [ ] Corporate spaces feel Corporate; settler spaces feel settler
- [ ] Outdoor rooms acknowledge The Eye without dramatizing it (except first view)
- [ ] The Coherence is never named or explained — only observed
- [ ] Descriptions of strange planet behavior say what is perceived, not what it means
- [ ] No room tells the player how to feel
- [ ] Dead, missing, or absent people are present as evidence (empty bunks, roster dates, wear patterns), not statements

