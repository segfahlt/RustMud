# Socials

Status: **Placeholder** — straightforward to implement, low priority until communication phase

---

## What This Is

Socials are pre-defined emote commands that generate first-person and third-person messages based on whether a target is specified. They are a core part of MUD culture and player expression.

Examples:
- `smile` → You smile. / Alaric smiles.
- `smile alaric` → You smile at Alaric. / Alaric smiles at you. / (others) Alaric smiles at Segfahlt.
- `wave` → You wave. / Alaric waves.

---

## Message Templates

Each social needs up to four strings:

| Key | Audience | Example (`wave`) |
|-----|----------|-----------------|
| `self_no_target` | Actor, no target | `You wave.` |
| `room_no_target` | Others in room, no target | `{actor} waves.` |
| `self_target` | Actor, with target | `You wave at {target}.` |
| `target_msg` | The target | `{actor} waves at you.` |
| `room_target` | Others in room, with target | `{actor} waves at {target}.` |

---

## Implementation Notes

- Socials defined in a data file (`data/socials.json`) — not hardcoded
- Loaded once at startup, available as a command category
- The registry handles prefix matching — social names are registered like any other command
- Category: `Social` (new category, separate from Communication)
- Permission: player (all characters)
- Target must be in the same room; error if not found

---

## Key Design Questions

- Can socials target objects or mobs, or only players?
- Do socials generate a channel log entry (for monitoring permission)?
- Custom socials: can players define their own? (Builder+ only?)
- Are there "body language" socials that consume no emote slot — just ambient? (e.g., `fidget` with random messages)
- Does the social list need to be theme-appropriate? (Some classic MUD socials won't fit every theme)

---

## Dependencies

- Communication phase (Phase 5 in roadmap)
- `say`, `tell`, `emote` commands (same phase)
- Mob/player lookup by name in room (needed for targeting)
