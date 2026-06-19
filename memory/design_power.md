---
name: design-power
description: Power generation system — placeholder note for future design
metadata:
  type: project
---

## Status: Placeholder — design not yet started

Power generation is a planned system. This note captures what's known so far and what needs designing.

## Why It Matters

Power is required infrastructure for:
- **Relay towers** — extend PERI satellite uplink range beyond Firstfall. Without power, relay towers are inert and `comm`/PERI access is range-limited. See [[lore-world]].
- **Future:** crafting stations, medical equipment, lighting, security systems, other player-built structures

## Known Constraints

- Must fit the survival/colony setting — players build and maintain it, not a given
- Should have interesting trade-offs (fuel cost, coherence interaction, vulnerability)
- Coherence field is a natural power source candidate given the setting
- Should matter enough that it creates meaningful decisions and risk

## Candidate Generation Types (not decided)

- **Solar** — reliable, low output, no fuel cost, can be disrupted by environmental events
- **Chemical/fuel cell** — higher output, requires fuel (crafted or scavenged), finite
- **Coherence tap** — uses the alien field as a power source; high output, potentially dangerous, Corporate would very much like to know about it
- **Salvaged ship components** — stripped from the *Perihelion* shuttle or debris field; high output, irreplaceable, story-significant

## Open Questions

- Is power tracked per-structure or as a zone-wide grid?
- Can power lines/cables be damaged (by weather, monsters, hostile players)?
- Does power generation tie into the coherence attunement system?
- Should Corporate be able to detect coherence taps remotely?

## Connection to [[design-npc-ai]]

Relay towers that extend PERI comm range require power. This makes `comm` access and NPC/PERI conversations over `comm` a meaningful infrastructure investment for players far from Firstfall.
