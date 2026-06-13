# RustMud Roadmap

The goal is a fully-formed, highly robust MUD with a real economy, online world creation, and eventually LLM-driven variability. This document tracks the build phases and their status.

---

## Phase 1 — Foundation (complete)

Core architecture that everything else builds on.

- [x] World model: zones, rooms, exits, directions
- [x] Gateway / game split — reboot without dropping connections
- [x] Session state machine: login → character select → playing
- [x] Account system with argon2id passwords
- [x] Multiple characters per account
- [x] Per-character permissions (player, remort, builder, monitor, dev, admin)
- [x] Bootstrap admin: first character gets Admin automatically
- [x] State save/restore: room positions and health persisted across reboots
- [x] Command registry: prefix matching, priority-based disambiguation, aliases
- [x] Admin commands: shutdown, reboot, reboot refresh
- [x] Zone data files (JSON), schema binary

---

## Phase 2 — World & Characters

Making the world feel real and characters worth playing.

- [ ] Objects: items that exist in rooms and can be picked up
- [ ] Inventory: players carry items, `get`/`drop`/`inventory` commands
- [ ] Equipment: wear slots, `wear`/`remove`/`equipment` commands
- [ ] Character stats: strength, dexterity, constitution, etc.
- [ ] Character advancement: experience points, levels
- [ ] `sethome` command (home_room field already wired in CharacterFile)
- [ ] `who` command: list online players
- [ ] `score` / `stats` command: display character info
- [ ] Room descriptions: time-of-day variations, weather
- [ ] Zone flags: indoor/outdoor, safe, no-magic, etc.

---

## Phase 3 — Mobs & Combat

The danger that makes the world worth navigating.

- [ ] Mob definitions in zone files
- [ ] Mob spawning: respawn timers, max count per room
- [ ] Basic combat loop: initiative, attack/defend rolls, damage
- [ ] `kill` / `flee` commands
- [ ] Death handling: corpse left in room, player sent to home room
- [ ] Mob loot tables
- [ ] Mob behavior flags: aggressive, assist, wanders
- [ ] Simple mob AI: wander, patrol, guard

---

## Phase 4 — Economy

The systems that make the world feel like a living place.

- [ ] **Currency**: gold coins (carried), bank accounts (per character), deposits/withdrawals
- [ ] **Shops**: static NPC vendors, buy/sell/list commands, item markup/markdown
- [ ] **Player vendor stands**: players rent a stand in a market room, set their own prices, items sell while offline
- [ ] **Auction system**: `auction <item> <min-bid>`, timed bidding, winner gets item, loser gold returned
- [ ] **Crafting**: recipes, component items, skill-gated
- [ ] **Loot economy balance**: drop rates, vendor prices tuned so gold has meaningful value

See `design/economy.md` for detailed design.

---

## Phase 5 — Social & Communication

Player-to-player interaction infrastructure.

- [ ] `say` — room-local chat
- [ ] `tell` — private message to a named player
- [ ] `shout` / `yell` — zone-wide
- [ ] `chat` / channels — global or group-based
- [ ] `emote` / `pose` — freeform action text
- [ ] `group` — party system, shared XP
- [ ] `friends` list
- [ ] Message of the Day (MOTD) on login

---

## Phase 6 — Builder Tools & Online Creation (OLC)

Letting the world grow without redeploying.

- [ ] Builder permission gates a separate command set
- [ ] `redit` — room editor (name, description, flags, exits)
- [ ] `zedit` — zone editor (new zones, zone flags, respawn rates)
- [ ] `medit` — mob editor
- [ ] `oedit` — object/item editor
- [ ] `goto` — builder teleport
- [ ] `stat room` / `stat mob` / `stat object` — inspect live world data
- [ ] Changes saved to JSON zone files in real time (no reboot needed)
- [ ] Version/diff tracking for zone edits

---

## Phase 7 — LLM / AI Integration

Using language models to make the world feel alive and infinitely variable. This is exploratory — design will evolve.

**Ideas under consideration:**

- **Dynamic NPC dialogue**: instead of static keyword-response trees, NPCs use an LLM to respond contextually to player input. Personality and lore defined in the mob file; the model fills in the conversation.
- **Procedural room descriptions**: a base description is stored in the zone file; at render time an LLM adds time-of-day, weather, and ambient detail variation so no two visits feel identical.
- **Dynamic quest generation**: LLM proposes quests based on world state, player level, and recent history. Quests are validated against world data before being offered.
- **Lore oracle**: a `lore` or `think` command lets players ask in-character questions; an LLM answers based on a curated world-lore context document.
- **Mob behavior narration**: combat actions and mob deaths are described with varied, evocative prose rather than fixed templates.
- **Content moderation**: player-generated content (vendor stand names, character descriptions) runs through a moderation check before persistence.

**Open questions:**
- Streaming output vs. buffered (MUDs are line-oriented; streaming needs care)
- Latency budget: LLM calls add 0.5–3s; which interactions can afford that?
- Cost control: per-call vs. cached responses for high-volume descriptions
- Model selection: local (ollama) for low-latency/cost vs. API (Claude/GPT) for quality
- Context injection: how much world state goes in the prompt? How is it kept fresh?

See `design/llm-integration.md` for design as it develops.

---

## Guiding Principles

- **No magic numbers in zone files** — behavior comes from flags and structured data, not hardcoded logic.
- **Everything is a file** — accounts, characters, zones, world save. No database until there's a clear reason to add one.
- **Reboots are free** — the gateway/game split means code changes never drop players. Lean on it.
- **Test the registry and the world model** — these are the stable cores everything else touches. Keep their test coverage high.
- **Design before code** — write the design doc before the first implementation PR. Put open questions there. Answer them before implementation starts.
