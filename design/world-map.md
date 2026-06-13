# World Map

Status: **Placeholder** — concept TBD, needs design discussion

---

## What This Is

A world map system gives players spatial orientation in the game world. This is distinct from the room/exit graph — it's a higher-level representation of geography. Classic MUDs have no map at all (players draw their own). Modern MUDs often provide ASCII maps. This doc is a placeholder for whatever form we decide to implement.

---

## Options to Consider

### Option A: No Built-In Map (Classic)
Players navigate by reading room descriptions and exits. Exploration and mapping are player skills. Some players use external MUD mapping tools. Zero implementation cost; maximum immersion for exploration.

### Option B: ASCII Room-Local Mini-Map
A small grid rendered adjacent to the room description showing the immediate vicinity (e.g., 5×5 rooms centered on player). Generated dynamically from the exit graph. Only shows rooms the player has visited (fog of war) or all rooms (no fog).

### Option C: Zone Overview Map
Each zone has a hand-authored ASCII map stored in the zone file. Displayed with `map` command. No dynamic generation — builders draw it.

### Option D: Coordinate-Based World Grid
Rooms have explicit (x, y, z) coordinates in addition to exit links. A world map command renders a top-down ASCII view. Requires retrofitting coordinates onto all existing rooms.

---

## Key Design Questions

- Should the map be auto-generated from exit graph data, or hand-authored by builders?
- Is fog-of-war (only show visited rooms) worth the complexity?
- Does the world have a coherent geography that a grid map would represent well, or is it a graph that doesn't map to 2D? (theme-dependent)
- Should this be in-client or send as text? (ANSI color will help a lot)
- Is there a `map` command, or is the mini-map always shown alongside room descriptions?
- How does indoor/underground space look on a map?

---

## Dependencies

- Theme/lore (world geography shape)
- Zone design (how zones relate spatially)
- ANSI color support (maps are much more usable with color)
- OLC (builders may need to define map data)
