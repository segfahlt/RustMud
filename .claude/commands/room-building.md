You are writing or reviewing descriptions for RustMud, a science-fiction MUD set on an alien world. Follow every rule below without exception. If reviewing existing descriptions, flag every violation with the rule number and a corrected version.

**Terminology:** This world uses two distinct navigable space types:
- **Area** — outdoor or open space (wilderness, paths, roads, open settlement ground). Amorphous, AI-generated, traffic-driven. This is what traditional MUDs call a "room."
- **Room** — a permanent built structure or interior (buildings, shelters, constructed spaces). Fixed, mappable, created by players, builders, or AI settlement generation.

All rules below apply to both. Areas describe open terrain, sky, and horizon. Rooms describe enclosure, construction, and interior architecture. The voice and perspective rules are identical.

---

## Voice and Perspective

**Rule 1 — Third person, present tense. Never use "you", "your", or "yours" in a description.**

The room description is narrated by an omniscient observer who has no knowledge of any specific player. It describes a place, not someone's experience of it.

```
VIOLATION: Your muscles ache in ways you don't have words for yet.
CORRECTED: The air smells of glycerol and recycled oxygen. Emergency
           lighting casts everything pale blue.
```

```
VIOLATION: You see it for the first time — a sky the wrong shade of violet.
CORRECTED: A sky the wrong shade of violet hangs overhead, lit from an
           angle no standard survey photograph captured.
```

---

## The Three Hard Prohibitions

**Rule 2 — Do not force actions.**

A description describes. It never makes the player do anything or implies they will do something.

```
VIOLATION: You follow the trail west toward the sound of water.
VIOLATION: You decide to rest here a moment.
CORRECTED: A trail leads west toward a sound that might be water.
```

**Rule 3 — Do not force emotions or physical sensations.**

Describe the cause. Never describe how the player feels about it or what their body does.

```
VIOLATION: The sight of the Eye fills you with dread.
VIOLATION: You feel a chill that has nothing to do with the temperature.
VIOLATION: The warmth here is the most comfort you have felt in weeks.
CORRECTED: The Eye hangs on the horizon, its corona of white fire visible
           even against the violet sky.
CORRECTED: The heating element in the corner ticks steadily, its orange
           glow the only warm color in the room.
```

**Rule 4 — Do not force history, origin, or assumed facts about the character.**

You do not know where the player came from, what they were told, what they have seen before, or what they own. Never assume any of it.

```
VIOLATION: The briefings didn't prepare you for the scale of it.
VIOLATION: A sky lit from an angle that doesn't match any sunrise you grew up with.
VIOLATION: The warmest color you've seen since Earth.
VIOLATION: Yours is here. / Your locker. / Your pod.
VIOLATION: You have been here before.
CORRECTED: The scale resists easy measure.
CORRECTED: A sky lit from an angle that matches no known stellar survey.
CORRECTED: Among the lockers, number 7-Bravo-9 stands slightly ajar.
```

---

## Room Names

**Rule 5 — Names are short, title-cased, and describe the specific space.**

The name you write is the rightmost segment of a location breadcrumb. The renderer prepends zone and building context automatically. Name the specific space only.

```
Player sees:   Firstfall > Engineering Bay > Main Workshop
You write:     Main Workshop
```

Rules:
- Maximum 50 characters.
- No leading articles (A, An, The) unless part of a proper name.
- Be specific. "Dark Corridor" is weak. "Intake Processing Hall" is strong.
- Sub-location compound names use an em-dash: `Bluff Path — Upper`.

```
VIOLATION: A Dark And Foreboding Chamber
VIOLATION: the waiting room
CORRECTED: Disembarkation Ramp
CORRECTED: Harbor Dock
```

---

## Show, Don't Tell

**Rule 6 — State what is there. Never editorialize about what it means.**

Give the player evidence. Let them draw their own conclusions about danger, beauty, or unease.

```
VIOLATION: This is a dangerous area. The forest is dark and gloomy.
CORRECTED: The canopy here is dense enough to reduce the alien sun to
           scattered coins of light on the ground. Sound carries
           strangely. Something large moved through recently — the
           undergrowth is still settling.
```

---

## Sensory Detail

**Rule 7 — Use at least one non-visual sense per room.**

Every description defaults to sight. Force yourself to include smell, sound, temperature, texture, or (rarely) taste. Two or three senses per room is the target — not all five every time.

| Sense  | Examples                                                        |
|--------|-----------------------------------------------------------------|
| Sight  | Lighting, color, scale, materials, distance                     |
| Sound  | Machinery, silence, echo, voices, wind, dripping                |
| Smell  | Chemical, organic, salt, smoke, rot, recycled air               |
| Touch  | Temperature, humidity, wind, deck texture underfoot             |
| Taste  | Only when relevant: metallic air, brine, smoke                  |

---

## No Dynamic States in Static Text

**Rule 8 — Do not describe anything that can change state.**

Room descriptions are permanent. Doors open and close. NPCs move. Objects get picked up. Never describe these in the room text.

```
VIOLATION: A door to the north stands closed.
VIOLATION: An empty chair sits beside the desk.
VIOLATION: The shopkeeper leans against the counter.
CORRECTED: A heavy door occupies the north wall.
CORRECTED: A chair has been pulled out from the desk.
CORRECTED: A counter runs along the east wall, worn smooth at the edge.
```

Fixtures handle interactive states. NPCs are separate coded entities. Room descriptions describe only the permanent bones of a place.

---

## Directions

**Rule 9 — Use absolute directions only. Never use relative directions.**

Players enter rooms from multiple exits. "To your left" is wrong half the time.

```
VIOLATION: Ahead of you, the tunnel opens into light.
VIOLATION: The path you came from curves away to your right.
CORRECTED: The tunnel opens into light to the east.
CORRECTED: The path curves back to the south.
```

---

## Examinable Nouns

**Rule 10 — Every significant noun mentioned in a description must be examinable.**

If the description mentions a sign, a device, a marking, a piece of equipment, or any notable object, there must be a fixture or object the player can examine to learn more. Do not describe something interesting and then make it unreachable.

---

## Exit and Direction Descriptions

**Rule 11 — Every compass direction needs a look description, whether or not an exit exists.**

- **With exit:** Describe what is visible down that passage. Suggest what lies ahead without fully revealing it.
- **Without exit:** Describe what blocks or ends that direction.

```
North (exit):    The corridor narrows before bending out of sight.
South (no exit): A solid bulkhead, its seams welded shut and painted
                 over many times.
East (exit):     Steps descend into a space that smells of machine oil.
West (no exit):  The dock edge drops away to the tea-colored water below.
```

---

## Length

| Room type           | Target                      |
|---------------------|-----------------------------|
| Transition / path   | 2–3 sentences               |
| Interior / location | 3–5 sentences               |
| Landmark / set piece| Up to 7 sentences           |

If you need more than 7 sentences, move the excess detail into fixture `examine` text. Players read the room description every time they enter a room. They read fixture text by choice.

---

## Formatting Rules

**Rule 12 — Write numbers as words up to twenty.** Use numerals for technical identifiers.
- "three cryo-pods" not "3 cryo-pods"
- "Locker 7-Bravo-9" and "Level 3 clearance" are fine as numerals

**Rule 13 — Every sentence must have a verb.** "A long corridor." is not a sentence.

**Rule 14 — No color codes in descriptions.** They render inconsistently and break accessibility.

**Rule 15 — No command hints.** Never write "type 'examine sign' to read it" or anything like it.

---

## Quick Violation Reference

| Pattern                              | Rule | Fix                                        |
|--------------------------------------|------|--------------------------------------------|
| "You feel…"                          | 3    | Describe the cause, not the effect         |
| "You see…"                           | 1    | Just describe what is there                |
| "You are…"                           | 1    | Reframe as third-person place description  |
| "…you grew up with"                  | 4    | Drop the reference entirely                |
| "…since Earth" / "…before the ship"  | 4    | Describe the thing itself                  |
| "The briefings said…"                | 4    | Let the world speak for itself             |
| "You decide to…"                     | 2    | Remove entirely                            |
| "Yours is…" / "Your X…"             | 4    | Describe the object without ownership      |
| "For the first time…"                | 4    | Remove; players have their own firsts      |
| "…to your left/right"               | 9    | Use north/south/east/west                  |
| "The door is closed."                | 8    | Describe the door, not its current state   |
| "The guard stands here."             | 8    | Code the NPC separately                    |
| "This place is dangerous."           | 6    | Show the evidence; omit the verdict        |
