# OLC — Online Creation

Status: **Placeholder** — Phase 6 in roadmap, depends on all world systems being stable

---

## What This Is

OLC (Online Creation) lets builders create and edit world content in-game without editing JSON files or rebooting. Changes are saved to zone files in real time. This is how the world grows after initial development — zone files become the output of OLC, not just the input.

---

## Planned Editors

| Command | Edits | Notes |
|---------|-------|-------|
| `redit` | Rooms | Name, description, flags, exits, fixtures |
| `zedit` | Zones | Zone name, level range, respawn rates, zone flags |
| `medit` | Mobs | Stats, behavior flags, loot tables, dialogue |
| `oedit` | Objects | Name, type, stats, flags, base value |
| `sedit` | Shops | Inventory, markup/markdown, hours |
| `goto <room>` | — | Teleport to any room by vnum/id |
| `stat room` | — | Inspect live room data |
| `stat mob` | — | Inspect live mob data |
| `stat object` | — | Inspect live object data |

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
