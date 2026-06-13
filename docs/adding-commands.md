# Adding Commands

The command system has three layers. Adding a command touches all three.

---

## 1. Add the `Command` variant (`src/commands/mod.rs`)

```rust
pub enum Command {
    // ... existing ...
    Say(String),        // example: say <message>
}
```

Variants carry typed, parsed arguments. No raw strings past this point.

---

## 2. Register it (`src/commands/registry.rs`)

Add a `CommandDef` to the `vec!` in `Registry::build()`:

```rust
CommandDef {
    name: "say", priority: 10, aliases: &["'"],
    category: Category::Communication,
    usage: "say <message>",
    description: "Say something to everyone in the room.",
    parse: |rest| {
        if rest.is_empty() {
            Err(ParseError::UnknownCommand("say what?".into()))
        } else {
            Ok(Command::Say(rest.to_string()))
        }
    },
},
```

### Priority tiers

| Value | Used for |
|-------|----------|
| 5 | Direction commands (`north`, `south`, etc.) — always win single-char prefix |
| 10 | Regular commands (`look`, `help`, `quit`, `say`, ...) |
| 20 | Admin commands (`shutdown`, `reboot`) — lose to any single-char |

Lower number = higher priority. When a prefix matches multiple commands, the lowest priority number wins. If two commands at the same priority share a prefix, that's a configuration error (`AmbiguousCommand`).

### Aliases

Use aliases for shortcuts that can't be expressed as name prefixes — special characters (`'` for say, `"` for emote, `?` for help) or full alternative words (`exit` for quit). Do **not** use aliases as abbreviations; the prefix matcher handles those automatically.

### Categories

`Movement`, `Info`, `Communication`, `Admin`. Add new categories to the `Category` enum and the `label()` match, then include in the `order` array in `help_text()`.

---

## 3. Handle it (`src/game.rs`)

Add a match arm in `execute()`:

```rust
pub fn execute(cmd: Command, client_id: u32, state: &mut GameState) -> (String, bool) {
    match cmd {
        // ... existing ...
        Command::Say(msg) => (say_message(msg, client_id, state), true),
    }
}
```

Return `(output_text, keep_playing)`. `false` for keep_playing ends the session (used only by `Quit`).

---

## Admin commands

Admin commands need permission checks and a signal to the main loop — they can't be handled entirely in `execute()` because they need to interact with the gateway (broadcast, disconnect all, etc.).

Handle them in `on_command()` in `src/main.rs` **before** the `Ok(cmd) =>` catch-all:

```rust
Ok(Command::NewAdminThing) => {
    if !has_perm(&permissions, Permission::Admin) {
        "Permission denied.\n\n> ".to_string()
    } else {
        // send gateway messages, fire signal, etc.
        let _ = signal_tx.send(Signal::SomeSignal).await;
        "Done.\n\n> ".to_string()
    }
}
```

If you need a new signal type, add it to the `Signal` enum in `main.rs` and handle it in the `signal_rx.recv()` arm of the main select loop.

---

## Checklist

- [ ] `Command` variant in `mod.rs`
- [ ] `CommandDef` in `registry.rs`
- [ ] Match arm in `execute()` (or `on_command()` for admin commands)
- [ ] Test in `commands/mod.rs` tests block (`parse_*` tests)
- [ ] Test in `game.rs` tests block (behavior tests)
- [ ] Update `agent_actions/` log
