#!/usr/bin/env python3
"""
heightmap_gen.py — Generates a grayscale heightmap PNG for The Eye's world.

Run:
    pip install -r requirements.txt
    python3 heightmap_gen.py

Outputs (written to this directory):
    heightmap.png  — grayscale PNG, import into Azgaar Fantasy Map Generator
    preview.png    — false-color terrain preview so you can validate the geography

Azgaar import:
    1. https://azgaar.github.io/Fantasy-Map-Generator/
    2. Tools → Heightmap → Load → select heightmap.png
    3. Set the sea level slider to ~30% when prompted
    4. Let Azgaar generate rivers, lakes, biomes
    5. Export: Tools → Export → to JSON  (save the .map file too as backup)
    The exported JSON becomes input for the next worldgen step (azgaar_parse.py).

Map coordinate convention:
    (0, 0) = northwest corner
    X increases eastward
    Y increases southward (standard image coordinates)
    North = up, South = down

World geography (1 pixel = 1 km at SIZE=2048):
    The planet's defining feature is an ancient impact crater ~160 km across,
    sitting inland in the upper-center of the continent. The crater floor is a
    closed basin — Azgaar will fill it with a lake.

    Ejecta and debris fan SOUTH from the crater ~600 km to the coastline.
    Perihelion Bay is a natural coastal harbor at the outer edge of the ejecta.
    Firstfall colony sits on the bluff above the bay.
    The continental interior (northwest) is the oldest, highest terrain.
    Ocean occupies the southern ~30% of the map.

    Gameplay progression (south to north):
      Firstfall coast → ejecta field (easy, low yield)
      → crater rim (dangerous, high yield)
      → crater lake (endgame)
"""

import numpy as np
from PIL import Image
from scipy.ndimage import gaussian_filter
import os

# ─── Configuration ─────────────────────────────────────────────────────────────
#
# Change SIZE to 1024 or 512 for a faster test run.
# Azgaar works fine with 512×512; use 2048 for the final version.

SIZE = 2048
SEED = 42   # fixed seed = deterministic output; change to get different base noise

# Impact crater — inland, upper-center of the landmass.
# The crater rim seals the floor as a closed basin; Azgaar fills it with a lake.
# Players must travel north through the ejecta field to reach the crater rim.
CRATER_X       = 1050   # pixel column
CRATER_Y       =  900   # pixel row — well inland, away from the coast
CRATER_INNER_R =   55   # lake basin floor radius, km
CRATER_RIM_R   =   80   # rim ring peak radius, km
CRATER_OUTER_R =  165   # outer slope fadeout, km

# Perihelion Bay — a natural coastal harbor at the outer edge of the ejecta field.
# Not connected to the crater. Sheltered port for the Firstfall colony.
BAY_CX = 1130   # pixel column — slightly east of crater (ejecta fans SE here)
BAY_CY = 1510   # pixel row — right at the coastline
BAY_RX =   90   # east-west radius, km
BAY_RY =   55   # north-south radius — gentle coastal indent, not a channel

# Coastal bluff — Firstfall sits on this ridge overlooking Perihelion Bay.
BLUFF_Y     = 1430
BLUFF_X_MIN = 1000
BLUFF_X_MAX = 1230

# Ejecta / debris field — fans SOUTH from crater toward the coast.
# The outer edge (EJECTA_DIST_MAX) reaches Perihelion Bay.
# Bearings clockwise from north: 0=N, 90=E, 180=S, 270=W.
EJECTA_ANGLE_MIN = 140   # SE start of fan
EJECTA_ANGLE_MAX = 220   # SW end of fan (centered on due-south = 180°)
EJECTA_DIST_MIN  =  85   # km from crater center (starts just past the rim)
EJECTA_DIST_MAX  = 615   # km — reaches crater (y≈900) to coast (y≈1515)

# Sea level: normalized height [0, 1] below which terrain becomes ocean.
# When you import into Azgaar, set the sea level slider to this percentage.
SEA_LEVEL = 0.30

# ─── Coordinate grids ──────────────────────────────────────────────────────────

print(f"Generating {SIZE}×{SIZE} heightmap  (1 pixel = 1 km at full size)...")
rng = np.random.default_rng(SEED)

gx, gy = np.meshgrid(
    np.arange(SIZE, dtype=np.float64),
    np.arange(SIZE, dtype=np.float64),
)
xx = gx / (SIZE - 1)   # 0..1 west→east
yy = gy / (SIZE - 1)   # 0..1 north→south

# ─── Base terrain: fractional Brownian motion ──────────────────────────────────
# Stack octaves of Gaussian-blurred random noise at halving scales.

def fbm(size, octaves, rng, base_sigma=600.0):
    result    = np.zeros((size, size))
    amplitude = 1.0
    total     = 0.0
    for i in range(octaves):
        sigma = base_sigma / (2.0 ** i)
        if sigma < 0.5:
            break
        layer   = gaussian_filter(rng.random((size, size)), sigma=sigma)
        result += layer * amplitude
        total  += amplitude
        amplitude *= 0.52
    return result / total

print("  [1/7] Base terrain noise...")
base = fbm(SIZE, octaves=9, rng=rng)
base = (base - base.min()) / (base.max() - base.min())

# ─── Continental tilt ──────────────────────────────────────────────────────────
# Northwest = high interior highlands; south = ocean; slight east-low bias.

print("  [2/7] Continental tilt and ocean push...")

tilt = (1.0 - yy) * 0.50 + (1.0 - xx) * 0.15
base = base * 0.50 + tilt * 0.50

# Push the far south firmly below sea level (open ocean floor).
ocean_push = np.clip((yy - 0.70) / 0.18, 0, 1) ** 1.5
base -= ocean_push * 0.70

# ─── Impact crater ─────────────────────────────────────────────────────────────

print("  [3/7] Stamping impact crater...")

dist_crater = np.sqrt((gx - CRATER_X) ** 2 + (gy - CRATER_Y) ** 2)

# Rim: a Gaussian ring of raised material at CRATER_RIM_R
rim = np.exp(-((dist_crater - CRATER_RIM_R) ** 2) / (2.0 * 16.0 ** 2))

# Interior depression: terrain falls toward the center inside the rim
inner = np.clip(1.0 - dist_crater / CRATER_RIM_R, 0.0, 1.0) ** 1.3
inner_mask = (dist_crater < CRATER_RIM_R).astype(float)

# Blend fades to zero at CRATER_OUTER_R
fade = np.clip(1.0 - dist_crater / CRATER_OUTER_R, 0.0, 1.0) ** 1.4

crater_delta  =  rim  * 0.48
crater_delta -= inner * inner_mask * 0.52

base += crater_delta * fade

# Crater lake basin — extra depression ensures a closed basin below drainage level.
# The sealed rim means Azgaar will generate a lake here automatically.
floor_mask = dist_crater < CRATER_INNER_R
base[floor_mask] -= 0.14

# ─── Perihelion Bay ─────────────────────────────────────────────────────────────
# Natural coastal harbor in the outer ejecta field. A gentle indentation —
# not carved from the crater. Gives Firstfall a sheltered port.

print("  [4/7] Shaping Perihelion Bay...")

bay_ell  = np.sqrt(((gx - BAY_CX) / BAY_RX) ** 2 + ((gy - BAY_CY) / BAY_RY) ** 2)
bay_pull = np.clip(1.0 - bay_ell, 0.0, 1.0) ** 2.0
base -= bay_pull * 0.40

# ─── Coastal bluff ─────────────────────────────────────────────────────────────
# Narrow elevated ridge at the bay's northern shore — Firstfall's ground.

print("  [5/7] Raising coastal bluff (Firstfall position)...")

bluff_bool = (
    (gx >= BLUFF_X_MIN) & (gx <= BLUFF_X_MAX) &
    (gy >= BLUFF_Y - 30) & (gy <= BLUFF_Y + 20)
)
bluff_soft = gaussian_filter(bluff_bool.astype(float), sigma=12)
base += bluff_soft * 0.25

# ─── Ejecta / debris field ─────────────────────────────────────────────────────
# Fans south from the crater. The outer edge reaches Perihelion Bay at the coast.

print("  [6/7] Building ejecta/debris field...")

# Bearing from crater center: 0°=N, 90°=E, 180°=S, 270°=W.
# Normalize to [0, 360) to handle the 140–220° range crossing the arctan2 sign boundary.
bearing = (np.degrees(np.arctan2(gx - CRATER_X, -(gy - CRATER_Y))) + 360) % 360

in_fan  = (bearing >= EJECTA_ANGLE_MIN) & (bearing <= EJECTA_ANGLE_MAX)
in_dist = (dist_crater >= EJECTA_DIST_MIN) & (dist_crater <= EJECTA_DIST_MAX)
ejecta  = (in_fan & in_dist).astype(float)

dist_taper = np.clip(
    1.0 - (dist_crater - EJECTA_DIST_MIN) / (EJECTA_DIST_MAX - EJECTA_DIST_MIN),
    0.0, 1.0
)
ejecta_rough = gaussian_filter(rng.random((SIZE, SIZE)), sigma=7) * 0.14

base += ejecta * (dist_taper * 0.24 + ejecta_rough)

# ─── Normalize ─────────────────────────────────────────────────────────────────

print("  [7/7] Normalizing...")
base = np.clip(base, 0.0, 1.0)
base = (base - base.min()) / (base.max() - base.min())

water_pct = np.mean(base < SEA_LEVEL) * 100
print(f"         Water: {water_pct:.1f}%   Land: {100-water_pct:.1f}%")

# ─── Output paths ──────────────────────────────────────────────────────────────

out_dir   = os.path.dirname(os.path.abspath(__file__))
hmap_path = os.path.join(out_dir, "heightmap.png")
prev_path = os.path.join(out_dir, "preview.png")

# ─── Grayscale heightmap ───────────────────────────────────────────────────────

img_gray = (base * 255).astype(np.uint8)
Image.fromarray(img_gray, mode='L').save(hmap_path)
print(f"\nSaved grayscale: {hmap_path}")

# ─── False-color preview ───────────────────────────────────────────────────────

preview = np.zeros((SIZE, SIZE, 3), dtype=np.uint8)

def paint(mask, rgb):
    preview[mask] = rgb

sl = SEA_LEVEL

paint(base < sl * 0.40,                          [  15,  50, 120])  # deep ocean
paint((base >= sl * 0.40) & (base < sl),         [  40, 100, 185])  # shallow ocean
paint((base >= sl)        & (base < sl + 0.06),  [ 155, 200, 115])  # coastal plain
paint((base >= sl + 0.06) & (base < sl + 0.22),  [  70, 135,  65])  # lowland / scrubland
paint((base >= sl + 0.22) & (base < sl + 0.40),  [ 105,  95,  55])  # upland
paint(base >= sl + 0.40,                          [ 200, 190, 175])  # highlands / interior

# Ejecta field — slightly distinct color
ejecta_vis = (in_fan & in_dist & (base >= sl))
paint(ejecta_vis, [130, 110, 65])

# Crater — rim area (above sea) and lake floor (below sea)
paint(floor_mask & (base >= sl), [170, 90,  75])   # any dry crater floor
paint(floor_mask & (base <  sl), [ 30, 120, 180])  # lake surface

# Perihelion Bay — distinct from open ocean
bay_close = (bay_ell < 0.4) & (base < sl)
paint(bay_close, [30, 80, 160])

# Firstfall position — yellow marker (on bluff above bay)
ff_x, ff_y = int(BAY_CX - 30), int(BLUFF_Y)
r = 10
y0, y1 = max(0, ff_y - r), min(SIZE, ff_y + r)
x0, x1 = max(0, ff_x - r), min(SIZE, ff_x + r)
preview[y0:y1, x0:x1] = [255, 220, 0]

Image.fromarray(preview, mode='RGB').save(prev_path)
print(f"Saved preview:   {prev_path}")

# ─── Summary ───────────────────────────────────────────────────────────────────

print(f"""
┌─ Geography summary ──────────────────────────────────────────┐
│                                                              │
│  Impact crater center:   ({CRATER_X:>4}, {CRATER_Y:>4})  — inland          │
│    Lake basin:           radius {CRATER_INNER_R} km  (Azgaar fills lake)   │
│    Rim peak:             radius {CRATER_RIM_R} km                        │
│    Outer slope end:      radius {CRATER_OUTER_R} km                      │
│                                                              │
│  Ejecta field:           bearing {EJECTA_ANGLE_MIN}° to {EJECTA_ANGLE_MAX}° (fans south)  │
│                          {EJECTA_DIST_MIN}–{EJECTA_DIST_MAX} km from crater              │
│                                                              │
│  Perihelion Bay:         ({BAY_CX:>4}, {BAY_CY:>4})  — coastal harbor    │
│    Ellipse radii:        {BAY_RX} km × {BAY_RY} km (E-W × N-S)        │
│                                                              │
│  Firstfall (bluff):      ({ff_x:>4}, {BLUFF_Y:>4})  — yellow marker    │
│                                                              │
│  Ocean / sea level:      {water_pct:.1f}% of map below threshold {SEA_LEVEL}   │
│                                                              │
└──────────────────────────────────────────────────────────────┘

Zone progression (south → north):
  Firstfall (coast, Perihelion Bay)
  → Ejecta field  [low yield, relatively safe]
  → Crater rim    [high yield, dangerous]
  → Crater lake   [endgame content]

Preview colors:
  light gray/white = highlands (NW interior)
  brown            = ejecta field
  dark red dot     = crater rim / dry crater floor
  bright blue dot  = crater lake
  dark blue indent = Perihelion Bay
  yellow square    = Firstfall colony

Azgaar import steps:
  1. Open https://azgaar.github.io/Fantasy-Map-Generator/
  2. Tools → Heightmap → Load → heightmap.png
  3. Set sea level to ~{int(SEA_LEVEL * 100)}% on the slider
  4. Generate — Azgaar will route rivers, fill lakes, assign biomes
  5. The crater should auto-generate as a lake (closed basin below drainage)
  6. Export → to JSON  (and save the .map file as backup)

Next step after Azgaar export:
  azgaar_parse.py  — converts Azgaar JSON to our WorldCell grid format
""")
