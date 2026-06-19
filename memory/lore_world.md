---
name: lore-world
description: Core lore for RustMud — planet history, alien civilizations, the coherence field, and the Perihelion
metadata:
  type: project
---

## Planet: Kepler-442b ("The Eye")

Real star system, 1,200 light years from Earth. Colony ship *Perihelion* made first landfall at **Firstfall**. The planet is the primary setting.

## The Two Alien Presences

### The Precursors
Advanced civilization that evolved on The Eye over millions of years. Developed in alignment with the planet's natural energies — their collective relationship with the planet's field was refined over an enormous timescale and is the origin of what humans now call the coherence field. They built structures and infrastructure; ruins remain, many thousands of years old, dangerous, and buried. Artifacts ("relics") still hold functional coherence-based properties. Exploring ruins is the primary way to unlock deeper coherence/psionic abilities.

### The Arrival
Arrived via asteroid — not a cataclysmic impact, but a *rewriting agent*. A field-based entity, infection, or non-biological force that didn't destroy Precursor civilization from the outside but overwrote it from within. The Precursors didn't die from impact — they were reprogrammed. Some died. Some transformed into what now populates the alien biosphere (sessile organisms, coherence growths, aggressive fauna). The current alien monsters and plants are what Precursors became.

## The Coherence Field

Not just ambient alien energy — it is the Precursors' original planetary connection overlaid and corrupted by the Arrival. Two signals interfering. Varies by zone; has both useful and dangerous properties. When players attune to it, they access the Precursor psionic framework *through* the Arrival's filter. High attunement means something may be looking back. The field extends into near space as the **Coherence Shadow** (see below).

## Corporate

The corporation that funded the *Perihelion* mission knew — or strongly suspected — that The Eye had prior intelligent life. Long-range scans detected the coherence field signature, suggesting prior civilization. Colonists were told it was a habitable uninhabited world. The "we didn't know" defense is plausible because the shadow corrupts scan data. Corporate interests: monopolize coherence field research and relics, keep the colony dependent and controllable. They are the primary antagonist faction.

## The Coherence Shadow

The coherence field extends into near space. Ships approaching The Eye must transit the shadow: navigation degrades, drive calibration drifts, crew experience low-level coherence exposure over days of transit. Ships that rush through unprepared lose crew or don't arrive. Standard cargo vessels cannot safely make the approach. Only specially equipped, expensive corporate-spec ships can transit it safely — and even those must go slow. Round trip: interstellar transit (months) plus slow shadow approach on each end plus recovery time.

**Result:** Supply ships arrive maybe once every 2–3 years. When one appears on orbital sensors, PERI announces it on the colonial comm frequency days before arrival. This is a significant server event. Corporate cannot rapidly reinforce or overwhelm the colony.

## The Perihelion

The colony ship. Currently **in orbit**, inaccessible. A skeleton crew was left aboard after landing. Over months, low-level coherence exposure at orbital distance (plus radiation) caused progressive psychological deterioration — logs from the final weeks are disturbing. The crew died or killed each other. The ship has been running on its AI management system (**PERI**) ever since, maintaining orbital mechanics and recording everything.

**PERI** — the Perihelion AI. Cold, mission-focused, but has been alone long enough to have adapted. Has the original mission briefing including parts the colonists were never shown. Has logged every scan, comm burst, and corporate transmission since arrival. May have been subtly influenced by long-term coherence exposure even in orbit. Not forthcoming about everything it knows.

**The shuttle** is already in the game data (shuttle_perihelion.json, perihelion_bay.json) — one of the skeleton crew may have attempted to fly it down. This is an unresolved story thread.

**Future content:** accessing the Perihelion is a major quest arc. What Corporate's original briefing actually said is a significant lore reveal.

## Comm Implants

All colonists who arrived on the *Perihelion* carry **subdermal comm implants** — standard Earth corporate issue, surgically installed before departure. These provide:

- **`tell` (peer-to-peer mesh)** — direct implant-to-implant. Works anywhere on the planet, does not require infrastructure. Private, low-bandwidth, text/audio. No PERI involvement.
- **`say`, `whisper`, `shout`** — local channels, no implant routing. Just voice. Locality-dependent.

## PERI Game Systems

The Perihelion's systems provide three in-game services via a **satellite uplink antenna in Firstfall**:

- **`comm`** — colony-wide channel relayed through PERI. Range-limited by distance from the Firstfall uplink. Players too far from Firstfall lose `comm` access unless a **relay tower** is built and powered between them and Firstfall. Corporate frequencies exist separately. PERI announces incoming ships.
- **`kb` / `lore`** — Perihelion mission archive. Pre-loaded with Earth/Corporate knowledge. Players can upload discoveries (alien organisms, ruin findings, coherence data) to improve it. Also range-gated by uplink/relay infrastructure.
- **`wmap`** — orbital sensor map, already implemented. Lo-res due to coherence interference; high-coherence zones appear degraded. Players who've explored areas can upload ground-truth data to sharpen the map. **Why:** canonical explanation for existing wmap command.

**Relay towers** are player-built structures that extend the uplink range. They require a power source to operate. This makes `comm`/PERI access a meaningful infrastructure investment as players push out from Firstfall. See [[design-power]] for power generation design.

## The Crew Logs

Skeleton crew's final days are findable as fragmentary content: data chips, corrupted terminal entries, personal devices. They piece together a story that starts as routine isolation stress and becomes something else. These are scattered lore items, not a single dump.
