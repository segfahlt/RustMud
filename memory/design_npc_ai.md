---
name: design-npc-ai
description: NPC dialogue model — reactive/scripted/AI-integrated hybrid with async delivery
metadata:
  type: project
---

## Overview

NPCs use a three-layer dialogue model. Layers are tried in order; the first match wins.

1. **Reactive** — keyword → canned response. Instant, zero latency.
2. **Scripted** — keyword → state machine node. Can branch, advance NPC state, trigger trade screens, etc. Instant.
3. **AI-driven** — no match in tree, or node explicitly flagged `ai_driven`. Calls Claude API async. Non-blocking.

## Data Model

```rust
struct NpcTemplate {
    id:            String,
    names:         Vec<String>,
    short:         String,
    description:   String,
    room_look:     String,
    persona:       String,            // AI system prompt: identity, knowledge, speech style, secrets
    dialogue_tree: HashMap<String, DialogueNode>,
    // ... combat fields, faction, location, etc.
}

enum DialogueNode {
    Static(String),                           // instant canned response
    Reactive(HashMap<FactionTier, String>),   // varies by standing
    Scripted { text: String, next_state: String }, // advances NPC state, stays scripted
    AiDriven,                                 // fall through to async API call
}
```

## Talk Channels

```rust
enum TalkChannel {
    Say,      // local, broadcast to everyone in location
    Whisper,  // local, private to initiating player
    Shout,    // local-ish, may bleed to adjacent areas
    Tell,     // non-local (implant mesh), private — works anywhere on planet
    Comm,     // non-local, PERI-relayed, range-gated by uplink/relay infrastructure
}
```

Locality rule:
- `Say`, `Whisper`, `Shout` → require player co-location with NPC at delivery time
- `Tell`, `Comm` → deliver wherever player is now

**Why:** `tell` uses subdermal comm implants (peer-to-peer, no infrastructure). `comm` routes through PERI satellite uplink and is range-limited. See [[lore-world]].

## Async AI Flow

1. Player sends `say <text>` or `tell <npc> <text>` to an NPC
2. Check dialogue tree → if match, respond immediately
3. If no match (or `AiDriven` node): immediately push thinking line to player, spawn background task

```
<NPC name> says, "Hmm..."
<NPC name> appears to be thinking...
```

4. Spawned tokio task calls Claude API with: NPC persona + world context snippet + conversation history + player input
5. Task sends `AiResponse` back via channel
6. Main `tokio::select!` loop receives it and applies locality rule (see below)

## In-Flight Record

```rust
struct AiPendingCall {
    client_id:    u32,
    npc_id:       String,
    npc_name:     String,
    npc_location: PlayerLocation,   // snapshot at call time
    channel:      TalkChannel,
    deadline:     Instant,          // fallback timeout
}
```

Stored in `GameState.pending_ai: Vec<AiPendingCall>` (or HashMap keyed by client_id).

## Delivery Logic

```rust
match ai_resp.channel {
    TalkChannel::Say | TalkChannel::Whisper | TalkChannel::Shout => {
        let player_loc = state.players.get(&ai_resp.client_id).map(|p| p.core.location);
        if player_loc == Some(ai_resp.npc_location) {
            send_to_location(ai_resp.npc_location, npc_speech);  // Say/Shout: everyone there
            // Whisper: only client_id
        }
        // else: silently discard — player left before NPC finished thinking
    }
    TalkChannel::Tell | TalkChannel::Comm => {
        send_to_client(ai_resp.client_id, npc_speech);  // deliver wherever they are
    }
}
```

## Timeout / Fallback

If `deadline` passes before API responds, send a fallback:

```
<NPC name> shakes their head slowly. "I'm not sure about that."
```

Timeout: ~8 seconds (tunable). The fallback should be personality-consistent if possible (stored per NpcTemplate as `ai_fallback: String`).

## AI Prompt Shape

When an AI call fires, the prompt includes:
- NpcTemplate.persona (who they are, what they know, speech style, what they won't say)
- World lore snippet relevant to NPC's role (faction, location)
- Last N lines of conversation history for this session
- The player's input
- Format constraint: respond as the NPC in character, 1–3 sentences, plain text

Conversation history is held in `GameState` per (client_id, npc_id) pair and cleared on NPC or player location change for local channels, or on session end for tell/comm.

## Deferred

- Multi-player awareness (others in the room hearing the conversation)
- NPC memory across sessions (persistent conversation summaries)
- NPC-initiated conversation (NPC calls out to a passing player)
