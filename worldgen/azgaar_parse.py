#!/usr/bin/env python3
"""
azgaar_parse.py — Converts an Azgaar Fantasy Map Generator JSON export
into the WorldCell grid format used by The Eye's game engine.

Usage:
    python3 azgaar_parse.py <path-to-azgaar-export.json>

Outputs (written to the worldgen/ directory AND data/world/):
    world_cells.json    — one record per Voronoi cell (7k+ cells)
    world_rivers.json   — one record per river
    world_features.json — lakes, ocean basins, continent outline
    world_summary.json  — stats and bounding box for the Rust loader
    world_map.txt       — pre-rasterized ASCII map for the wmap command

Biome → lore name mapping (Azgaar names → The Eye lore names):
    Glacier                    → Highland Barrens   (^)
    Tundra                     → Scoured Flats       (.)
    Grassland                  → Scrubland           (,)
    Temperate deciduous forest → Canopy Zone         (%)
    Taiga                      → Deep Canopy         (T)
    Temperate rainforest       → Verdant Zone        (&)
    Wetland                    → Brine Flats         (w)
    Savanna                    → Dust Plains         (")
    Cold/Hot desert            → Ash Fields          (:)
    Marine                     → Ocean               (~)
    Lake (feature)             → Lake                (o)
    Firstfall colony           → [colony marker]     (*)
"""

import json
import sys
import os
import numpy as np
from collections import Counter
from scipy.ndimage import gaussian_filter
from scipy.spatial import KDTree

# ─── Input ─────────────────────────────────────────────────────────────────────

if len(sys.argv) < 2:
    default = os.path.expanduser("~/Downloads/The Eye Full 2026-06-14-20-39.json")
    if os.path.exists(default):
        src = default
    else:
        print("Usage: python3 azgaar_parse.py <path-to-azgaar-export.json>")
        sys.exit(1)
else:
    src = sys.argv[1]

print(f"Loading: {src}")
with open(src) as f:
    raw = json.load(f)

info     = raw["info"]
pack     = raw["pack"]
grid     = raw["grid"]
biomes   = raw["biomesData"]

map_w = info["width"]
map_h = info["height"]
print(f"Map: {map_w}×{map_h}  |  Azgaar v{info['version']}  |  seed {info['seed']}")

# ─── Biome mappings ────────────────────────────────────────────────────────────

BIOME_NAMES = biomes["name"]   # index → Azgaar name

# Azgaar name → lore name used in The Eye
LORE_BIOME: dict[str, str] = {
    "Marine":                       "Ocean",
    "Hot desert":                   "Ash Fields",
    "Cold desert":                  "Ash Fields",
    "Savanna":                      "Dust Plains",
    "Grassland":                    "Scrubland",
    "Tropical seasonal forest":     "Canopy Zone",
    "Temperate deciduous forest":   "Canopy Zone",
    "Tropical rainforest":          "Verdant Zone",
    "Temperate rainforest":         "Verdant Zone",
    "Taiga":                        "Deep Canopy",
    "Tundra":                       "Scoured Flats",
    "Glacier":                      "Highland Barrens",
    "Wetland":                      "Brine Flats",
}

# Lore name → ASCII display character for the wmap command
BIOME_CHARS: dict[str, str] = {
    "Ocean":             "~",
    "Ash Fields":        ":",
    "Dust Plains":       '"',
    "Scrubland":         ",",
    "Canopy Zone":       "%",
    "Verdant Zone":      "&",
    "Deep Canopy":       "T",
    "Scoured Flats":     ".",
    "Highland Barrens":  "^",
    "Brine Flats":       "w",
    "Lake":              "o",
}

# ─── Grid cell lookup (for temperature + precipitation) ────────────────────────

grid_cells  = grid["cells"]
grid_lookup = {gc["i"]: gc for gc in grid_cells}

# ─── Feature lookup ────────────────────────────────────────────────────────────

pack_features = pack["features"]
feature_lookup: dict[int, dict] = {}
for feat in pack_features:
    if isinstance(feat, dict):
        feature_lookup[feat["i"]] = feat

def is_lake_feature(feat_id: int) -> bool:
    feat = feature_lookup.get(feat_id)
    return bool(feat and feat.get("type") == "lake")

def feature_group(feat_id: int) -> str | None:
    feat = feature_lookup.get(feat_id)
    return feat.get("group") if feat else None

# ─── River set ────────────────────────────────────────────────────────────────

pack_rivers = pack["rivers"]
cell_to_river: dict[int, int] = {}
for river in pack_rivers:
    if isinstance(river, dict):
        for ci in river.get("cells", []):
            cell_to_river[ci] = river["i"]

# ─── Parse pack cells → WorldCells ────────────────────────────────────────────

print("Parsing cells...")
world_cells = []

for cell in pack["cells"]:
    ci    = cell["i"]
    x, y  = cell["p"]
    h     = cell["h"]
    t     = cell["t"]
    feat  = cell["f"]
    biome = cell["biome"]
    r     = cell["r"]
    fl    = cell["fl"]
    pop   = cell["pop"]
    burg  = cell["burg"]

    gc    = grid_lookup.get(cell.get("g", -1), {})
    temp  = gc.get("temp", 0)
    prec  = gc.get("prec", 0)

    azgaar_biome = BIOME_NAMES[biome] if biome < len(BIOME_NAMES) else "Unknown"
    is_lake      = is_lake_feature(feat)
    lore_biome   = "Lake" if is_lake else LORE_BIOME.get(azgaar_biome, azgaar_biome)

    world_cells.append({
        "id":            ci,
        "x":             round(x, 2),
        "y":             round(y, 2),
        "elevation":     h,
        "azgaar_biome":  azgaar_biome,
        "biome":         lore_biome,
        "terrain_dist":  t,
        "is_ocean":      t < 0,
        "is_lake":       is_lake,
        "feature_id":    feat,
        "has_river":     r > 0 or ci in cell_to_river,
        "river_id":      (r if r > 0 else cell_to_river.get(ci)) or None,
        "river_flow":    fl,
        "temperature":   temp,
        "precipitation": prec,
        "population":    pop,
        "burg_id":       burg if burg > 0 else None,
    })

# ─── Parse rivers ─────────────────────────────────────────────────────────────

print("Parsing rivers...")
world_rivers = []

for river in pack_rivers:
    if not isinstance(river, dict):
        continue
    world_rivers.append({
        "id":          river["i"],
        "name":        river.get("name", ""),
        "type":        river.get("type", "River"),
        "source_cell": river.get("source"),
        "mouth_cell":  river.get("mouth"),
        "cells":       river.get("cells", []),
        "discharge":   river.get("discharge", 0),
        "length_km":   round(river.get("length", 0), 1),
        "parent_id":   river.get("parent"),
        "basin_id":    river.get("basin"),
    })

# ─── Parse features ───────────────────────────────────────────────────────────

print("Parsing features...")
world_features = []

for feat in pack_features:
    if not isinstance(feat, dict):
        continue
    world_features.append({
        "id":          feat["i"],
        "type":        feat.get("type", "unknown"),
        "is_land":     feat.get("land", False),
        "is_border":   feat.get("border", False),
        "group":       feat.get("group"),
        "name":        feat.get("name"),
        "cell_count":  feat.get("cells", 0),
        "area_km2":    feat.get("area", 0),
        "height":      feat.get("height"),
        "outlet_cell": feat.get("outlet"),
    })

# ─── Rasterize to ASCII map ────────────────────────────────────────────────────
# Build a KDTree over all cell centers and query it for each display pixel.
# Output: a DISPLAY_H × DISPLAY_W grid of ASCII characters.
#
# Firstfall colony is at ~(1100, 1430) in heightmap pixel space.
# The crater lake (Asar) is the 20-cell freshwater feature centered ~(1088, 1344).

print("Rasterizing ASCII map...")

DISPLAY_W   = 76   # fits inside 80-char terminal with border
DISPLAY_H   = 34

FIRSTFALL_X = 1100
FIRSTFALL_Y = 1430

points = np.array([[c["x"], c["y"]] for c in world_cells])
tree   = KDTree(points)

def cell_char(c: dict) -> str:
    if c["is_lake"]:
        return "o"
    if c["is_ocean"]:
        return "~"
    return BIOME_CHARS.get(c["biome"], "?")

rows: list[str] = []
for row in range(DISPLAY_H):
    line: list[str] = []
    for col in range(DISPLAY_W):
        world_x = (col + 0.5) / DISPLAY_W * map_w
        world_y = (row + 0.5) / DISPLAY_H * map_h
        _, idx  = tree.query([world_x, world_y])
        line.append(cell_char(world_cells[idx]))
    rows.append("".join(line))

# Stamp the Firstfall marker
ff_col = int(FIRSTFALL_X / map_w * DISPLAY_W)
ff_row = int(FIRSTFALL_Y / map_h * DISPLAY_H)
if 0 <= ff_row < DISPLAY_H and 0 <= ff_col < DISPLAY_W:
    r         = list(rows[ff_row])
    r[ff_col] = "*"
    rows[ff_row] = "".join(r)

# ─── Summary stats ────────────────────────────────────────────────────────────

land_cells  = [c for c in world_cells if not c["is_ocean"]]
ocean_cells = [c for c in world_cells if c["is_ocean"]]
lake_cells  = [c for c in world_cells if c["is_lake"]]
river_cells = [c for c in world_cells if c["has_river"]]
elevations  = [c["elevation"] for c in land_cells]

summary = {
    "source_file":    os.path.basename(src),
    "azgaar_version": info["version"],
    "map_width":      map_w,
    "map_height":     map_h,
    "total_cells":    len(world_cells),
    "land_cells":     len(land_cells),
    "ocean_cells":    len(ocean_cells),
    "lake_cells":     len(lake_cells),
    "river_cells":    len(river_cells),
    "river_count":    len(world_rivers),
    "feature_count":  len(world_features),
    "elevation_min":  min(elevations) if elevations else 0,
    "elevation_max":  max(elevations) if elevations else 0,
    "elevation_mean": round(sum(elevations) / len(elevations), 1) if elevations else 0,
    "biome_counts":   {},
    "lore_biome_counts": {},
    "lakes": [
        {"id": f["id"], "name": f["name"], "cells": f["cell_count"], "group": f["group"]}
        for f in world_features if f["type"] == "lake"
    ],
    "ascii_map_size": f"{DISPLAY_W}×{DISPLAY_H}",
}

for name, count in Counter(c["azgaar_biome"] for c in land_cells).most_common():
    summary["biome_counts"][name] = count
for name, count in Counter(c["biome"] for c in land_cells).most_common():
    summary["lore_biome_counts"][name] = count

# ─── Write output ─────────────────────────────────────────────────────────────

out_dir      = os.path.dirname(os.path.abspath(__file__))
data_dir     = os.path.join(out_dir, "..", "data", "world")
os.makedirs(data_dir, exist_ok=True)

map_text = "\n".join(rows)

def write_json(path: str, data):
    with open(path, "w") as f:
        json.dump(data, f, separators=(",", ":"))
    kb = os.path.getsize(path) // 1024
    n  = len(data) if isinstance(data, list) else "—"
    print(f"  Wrote {path}  ({kb} KB,  {n} records)")

def write_text(path: str, text: str):
    with open(path, "w") as f:
        f.write(text)
    print(f"  Wrote {path}")

print("Writing output files...")
write_json(os.path.join(out_dir,  "world_cells.json"),    world_cells)
write_json(os.path.join(out_dir,  "world_rivers.json"),   world_rivers)
write_json(os.path.join(out_dir,  "world_features.json"), world_features)
write_json(os.path.join(out_dir,  "world_summary.json"),  summary)
write_text(os.path.join(out_dir,  "world_map.txt"),       map_text)
write_text(os.path.join(data_dir, "map.txt"),             map_text)

# ─── Print summary ────────────────────────────────────────────────────────────

print(f"""
┌─ World parse summary ────────────────────────────────────────┐
│                                                              │
│  Total cells:   {summary['total_cells']:>6}                               │
│    Land:        {summary['land_cells']:>6}                               │
│    Ocean:       {summary['ocean_cells']:>6}                               │
│    Lake:        {summary['lake_cells']:>6}                               │
│    River:       {summary['river_cells']:>6}  (cells with river)         │
│  Rivers:        {summary['river_count']:>6}                               │
│                                                              │
│  Elevation (land):  {summary['elevation_min']}–{summary['elevation_max']}  mean={summary['elevation_mean']}              │
│                                                              │
│  Lore biomes (land cells):""")
for name, cnt in list(summary["lore_biome_counts"].items())[:9]:
    bar = "█" * (cnt // 100)
    print(f"│    {name:<28} {cnt:>5}  {bar}")
print("│")
print("│  Lakes:")
for lake in summary["lakes"]:
    print(f"│    [{lake['id']}] {(lake['name'] or 'unnamed'):<20} {lake['cells']:>3} cells  ({lake['group']})")
print("│")
print(f"│  ASCII map: {DISPLAY_W}×{DISPLAY_H} chars → data/world/map.txt")
print("│")
print("└──────────────────────────────────────────────────────────────┘")
print()
print("ASCII preview:")
print("+" + "-" * DISPLAY_W + "+")
for r in rows:
    print("|" + r + "|")
print("+" + "-" * DISPLAY_W + "+")
print()
print("  ^ Highland Barrens  , Scrubland    % Canopy Zone  T Deep Canopy")
print("  & Verdant Zone      w Brine Flats  . Scoured Flats : Ash Fields")
print('  ~ Ocean             o Lake         * Firstfall colony')
print()
print("Next step: run the Rust game and use 'wmap' in-game.")
