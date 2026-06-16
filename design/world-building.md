# World Building Guide — The Eye

Status: **Active** — this is the authoritative style reference for Area and Room writing. All content agents should receive this document as context before generating or modifying Areas or Rooms.

> **Structural model:** This document covers writing style and description rules. For the three-tier world structure (Zone → Area → Room), see `design/world-structure.md`.
>
> **Terminology:** In this world, outdoor navigable spaces are called **Areas**. Permanent built structures are called **Rooms**. The word "room" in traditional MUD contexts maps to Area here. Both follow the rules in this document — Areas describe open terrain, Rooms describe enclosed built space.

---

## Core Rule

Each room must be independently intelligible. Assume the player has no context about adjacent rooms, the zone's geography, or how they arrived. Describe what exists in this space through concrete sensory details. Convey the zone's atmosphere and progression through environmental accumulation, not through direct reference to other areas.

---

## Area and Room Anatomy

A complete description covers these elements, in roughly this order:

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
- **No second-person "you" in descriptions** except at moments of deliberate narrative weight (the Disembarkation Ramp, the Cryo-Bay revival). Use sparingly. **AI agents follow a stricter rule: never use "you"** — the exception is reserved for human authors making a deliberate craft choice. See `.claude/commands/room-building.md`.
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

## Output Render Order

When a player enters an Area or Room, or types `look`, output is rendered in this order:

```
[Location breadcrumb]
[Description]
                          ← blank line
[Fixture state lines]     ← one per visible fixture, state-aware
[Object room-look lines]  ← one per object on the floor
                          ← blank line
Exits: [directions]
```

### Location Breadcrumb

The first line of every location output is a spatial breadcrumb, not a bare name. It tells the player where they are in the world hierarchy without requiring a separate `where` command.

```
In an Area:   Zone Name > Area Name
In a Room:    Zone Name > Building Name > Room Name
```

The **Zone Name** is the geographic zone the location belongs to. The **Area Name** or **Building Name** is the mid-tier context — the outdoor area or building cluster. The **Room Name** is the specific space. For a Room that is not part of a multi-room building cluster, the mid-tier segment is the Area the building is entered from.

Examples:
```
Firstfall > Engineering Bay > Main Workshop
Firstfall > Settler Quarter > Wall Garden
Southern Debris Field > Crumbling Bluff Path
Perihelion Bay Zone > Harbor Dock
Firstfall > Medical Facility > Treatment Bay
```

The breadcrumb uses `>` as the separator. No trailing punctuation. Title-cased. The full breadcrumb is the location title — there is no separate "room name" line distinct from it.

**Breadcrumb on movement:** The breadcrumb line is shown every time a player moves (strides between Areas, enters a Room, exits a Room). It is the first thing printed on any move event, followed immediately by the description.

**AI note:** When generating a name for an Area or Room, you are generating only the rightmost segment of the breadcrumb. The Zone and mid-tier context are prepended by the renderer automatically. Name the specific space; the system provides the hierarchy.

The **description** sets the space and atmosphere. It does not describe fixtures in detail — at most a sensory hint that a fixture exists ("heat rolling from the north wall"). The **fixture state line** is where the fixture announces itself. The player can then `look at <fixture>` for more.

Fixture state lines are single sentences written to the same standard as room descriptions: concrete, sensory, present tense, no interpretation. They change based on the fixture's current state — a cold forge and a running forge have different lines. Coherence fixtures have state lines that change with global threat level.

---

## Description Length

- **Transit Areas / Rooms** (corridors, paths, connecting spaces): 2–4 sentences. They exist to move through, not to study.
- **Key Areas / Rooms** (faction hubs, major NPCs, significant locations): 5–8 sentences. Density earns attention.
- **Critical narrative spaces** (Disembarkation Ramp, Assignment Board): As long as the moment demands, no longer.

---

## Evolving Areas and Rooms (ML/LLM Integration Context)

Areas evolve primarily from traffic and use — see `design/world-structure.md` for the evolution model. When feeding an Area or Room to an LLM for evolution, aging, or expansion, provide:

1. **This document** as system context / style guide
2. **The Area or Room's current description** (verbatim)
3. **Evolution stage** (for Areas): current and target stage if evolving
4. **Zone state**: biome, Coherence level, faction footprint
5. **Adjacent Area/Room context**: 1–2 neighboring descriptions for continuity
6. **Relevant world events**: faction changes, resource surges or shortages, Corporate pressure level
7. **Instruction type**: `evolve` (traffic-driven change), `age` (time passing, devolution), `expand` (generate adjacent Area), `build` (add a Room to an Area), `react` (respond to a world event)

The LLM should produce changes that reflect the instruction, not rewrites that change the space's fundamental nature. A supply depot under resource pressure should still be a supply depot — but the shelves are different, the wear shows, the mood has shifted.

**Do not** ask the LLM to rewrite descriptions from scratch without providing the current description. Context continuity matters.

---

## Zone Writing Checklist

Before committing a zone, verify:

- [ ] Every Area and Room is independently intelligible without knowing adjacent spaces
- [ ] No Area or Room names an adjacent space in its description
- [ ] Exit hints use sensory/physical language, not destination names
- [ ] Corporate spaces feel Corporate; settler spaces feel settler
- [ ] Outdoor Areas acknowledge The Eye without dramatizing it (except first view)
- [ ] The Coherence is never named or explained — only observed
- [ ] Descriptions of strange planet behavior say what is perceived, not what it means
- [ ] No description tells the player how to feel
- [ ] Dead, missing, or absent people are present as evidence (empty bunks, roster dates, wear patterns), not statements
- [ ] Area evolution stage is consistent with its description (a Pristine Area has no worn paths; a Trail has clear infrastructure)
- [ ] Rooms (buildings) reference their enclosure and construction, not open terrain

