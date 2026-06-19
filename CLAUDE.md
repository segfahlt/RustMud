# RustMud — Claude Code Guidelines

## Testing

Every new function or command must ship with tests. Minimum coverage:

- **Parse tests** (`src/commands/mod.rs` test block): one happy-path parse and one error case (missing target, missing direction, etc.) for every new `Command` variant.
- **Behaviour tests** (`src/game.rs` test block, or a dedicated test module): at least one test that exercises the command through `execute()` and asserts on the output string or resulting game state.
- Run `cargo test` before reporting a task complete. All 169+ tests must pass.

## Code style

- No comments unless the WHY is non-obvious.
- No trailing summary comments ("// added for task 2", etc.).
- Match existing formatting — 4-space indents, aligned match arms.
