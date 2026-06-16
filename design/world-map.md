# World Map

Status: **Active** — design settled, implementation pending

---

## Overview

The world map exists at two levels: a **macro map** (zone-level, hex grid) and a **micro map** (area-level, per-zone graph). These are distinct systems serving distinct purposes.

For the three-tier structural model (Zone → Area → Room), see `design/world-structure.md`.

---

## Macro Map — Zone Grid

The macro map is derived from the Azgaar world generator output. It is a hex grid where each cell represents one Zone (~500m flat-to-flat). It answers the question "where am I in the world?"

**Properties:**
- Hex grid in axial coordinates `(q, r)`
- Each cell carries biome type (from Azgaar symbol: `^`, `,`, `%`, `T`, `&`, `w`, `.`, `:`, `~`, `o`, `*`)
- Six directions between zones: N, NE, SE, S, SW, NW
- Stored in `data/world/map.txt` (current ASCII representation) and eventually in a structured hex grid format

**What it shows:**
- Biome regions at a glance
- Approximate zone adjacency
- Key landmarks (Firstfall `*`, major water bodies, terrain transitions)

**`wmap` command:** Renders the current ASCII map with legend. Already implemented. As the zone system matures, this will become a player-visible fog-of-war map — zones the player has not entered show as unexplored.

**Fog of war:** Zones are revealed when any player enters them for the first time. Until then, the macro map shows terrain type (derived from Azgaar biome) but no zone detail. A separate "surveyed" flag per zone tracks whether the zone's Areas have been generated.

---

## Micro Map — Area Graph

The micro map is the navigation graph within a zone. It answers "where am I in this zone, and what's around me?"

**Properties:**
- Graph of Area nodes connected by exits (N, S, E, W, Up, Down)
- Areas have `offset (i16, i16)` from zone origin for approximate spatial layout
- Not a grid — amorphous, non-Euclidean
- Populated lazily as players explore

**Client minimap:** For clients that support it, the server exposes `map_x, map_y, map_z` coordinates per Area for rendering in a local minimap widget. These are server-managed, set at generation time to approximate the offset, and normalized so new Areas generated between existing ones fit without disrupting the existing layout.

The micro map is the main tool for exploration feedback. Players who have visited an area see it on the minimap. Areas they haven't visited are blank.

---

## Directions

Navigation mode is determined by whether the player is in an Area or a Room.

**Area mode** (outdoor, Zone/Area travel): `N, S, NE, NW, SE, SW`
- No E, W, Up, or Down
- Applies to all movement within and between Zones
- Zone crossing is transparent — player types `north`, game resolves which Zone they land in

**Room mode** (interior, settlement/dungeon travel): `N, S, E, W, NE, NW, SE, SW, Up, Down`
- Full ten-direction set
- Applies inside all Room clusters: settlements, buildings, caves, dungeons
- Settlements (including Firstfall) are 100% Room clusters — E/W and Up/Down are valid inside them

The mode switch happens at the boundary exit: an Area fixture exit drops the player into Room mode; a Room exit pointing to an Area returns them to Area mode. The breadcrumb format reflects the current mode (`Zone > Area` vs. `Zone > Building > Room`).

---

## Map Data Sources

| Data | Source | Format |
|---|---|---|
| Biome grid | Azgaar export → `worldgen/azgaar_parse.py` | `data/world/map.txt` |
| Zone hex coordinates | Derived from Azgaar cell positions | Zone struct `(q, r)` |
| Zone adjacency | Computed from hex grid | Zone struct `adjacent_zone_ids` |
| Area offsets | Set at generation time | Area struct `offset` |
| Client coordinates | Derived from offset, managed by server | Area struct `map_x/y/z` |

---

## Open Questions

- **Fog of war granularity**: Reveal at zone level (entered the zone) or Area level (visited this specific Area)?
- **Player map persistence**: Does a player's explored map persist across sessions? Almost certainly yes.
- **Shared exploration**: If one player reveals a zone, is it revealed for all players, or per-character?
- **Map command variants**: `wmap` (world macro), `map` (zone micro), `minimap` (always-on sidebar)?
