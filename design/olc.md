# OLC — Online Creation

Status: **Placeholder** — Phase 6 in roadmap, depends on all world systems being stable

---

## What This Is

OLC (Online Creation) lets builders create and edit world content in-game without editing JSON files or rebooting. Changes are saved to zone files in real time. This is how the world grows after initial development — zone files become the output of OLC, not just the input.

---

## Planned Editors

| Command | Edits | Notes |
|---------|-------|-------|
| `redit` | Rooms | Name, description, flags, exits, fixtures, building_id. Full manual authoring workflow. |
| `aedit` | Areas | Tweak AI-generated descriptions, override stride, set flags, adjust exits. Not a full authoring tool — Areas are primarily AI-generated. See note below. |
| `zedit` | Zones | Zone name, biome, Coherence seed, adjacent zone links, radius_steps. Tweaks to existing zones; initial zone creation is offline via world-gen scripts. |
| `medit` | Mobs | Stats, behavior flags, loot tables, dialogue |
| `oedit` | Objects | Name, type, stats, flags, base value |
| `sedit` | Shops | Inventory, markup/markdown, hours |
| `goto <id>` | — | Teleport to any Area or Room by id |
| `stat area` | — | Inspect live Area data (evolution stage, visit counts, refactor_pending) |
| `stat room` | — | Inspect live Room data |
| `stat mob` | — | Inspect live mob data |
| `stat object` | — | Inspect live object data |

### Room ID Sequence

Room IDs are assigned by the server from a global sequence stored in `data/state/last_room_id`. This file contains a single integer — the last assigned Room ID. On startup the server loads it into an in-memory atomic counter. OLC `redit` and player `build` commands both claim the next ID by incrementing the counter. The file is flushed on graceful shutdown and periodically during uptime.

No Room files are hand-crafted or loaded from disk outside the initial Firstfall migration. OLC is the sole room-creation path for all subsequent content, making the server the single authority on ID assignment.

---

### `aedit` — Area Editor

Areas are AI-generated, not hand-authored. `aedit` is a correction and override tool, not a creation tool. Planned operations:

- Override the AI-generated description (when the AI got it wrong)
- Manually trigger AI regeneration for the current Area
- Adjust exits (add, remove, redirect)
- Set stride manually (use with caution — triggers refactor_pending)
- Set flags (e.g., force `generated: false` for hand-authored Areas)
- View raw evolution state (stage, visit counts, fixture list)

**Status:** Placeholder. `aedit` will be designed once the AI generation pipeline is operational. In the interim, Areas can be generated offline and loaded from zone files.

---

## Key Design Questions

- **Vnum system**: do rooms/mobs/objects have numeric vnums (classic) or string IDs (our current approach)? String IDs are more readable and git-friendly; vnums are simpler to allocate.
- **Zone ownership**: can a builder only edit zones assigned to them? How is zone assignment managed?
- **Conflict handling**: what happens if two builders edit the same zone simultaneously?
- **Save timing**: save to disk immediately on each change, or on `zsave`/explicit save command?
- **Revert/undo**: is there any rollback mechanism, or is git the safety net?
- **Format stability**: OLC must write zone JSON in the same format the loader reads. The zone loader and OLC writer must share the same schema.
- **Inline editor UX**: classic MUDs use numbered menus. We could do command-argument style instead (`redit name The Dark Corridor`). Which fits better?
- **Fixture editor**: how does `redit` handle creating and editing fixtures?

---

## Dependencies

- All world systems (rooms, mobs, objects, fixtures, shops) must be stable before OLC
- Builder permission
- Zone file format must be finalized
- `schemars` schema should be enforced on OLC output
