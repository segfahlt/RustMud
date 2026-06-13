# Running RustMud

RustMud runs as two separate processes. Start them in order.

## 1. Build

```bash
cargo build
```

## 2. Start the gateway

The gateway holds all player TCP connections and never needs to restart.

```bash
cargo run --bin gateway
```

You should see:
```
Gateway listening on 0.0.0.0:4000
Waiting for game on /tmp/rustmud.sock
```

## 3. Start the game loop

In a second terminal:

```bash
cargo run
```

You should see (in the gateway terminal):
```
Game connected.
```

## 4. Connect as a player

In a third terminal, use `telnet` or `nc`:

```bash
telnet localhost 4000
```
```bash
nc localhost 4000
```

Type `help` for available commands, `quit` to disconnect.

---

## Rebooting the game

To reload game logic or world data without disconnecting players:

1. Kill the game loop (`Ctrl+C` in its terminal).
2. Players see: `[Game is rebooting. Please wait...]`
3. Restart: `cargo run`
4. Players automatically receive their welcome message when the game reconnects.

The gateway never needs to restart. Only restart it if you change gateway code itself.

---

## Other binaries

Print the JSON Schema for zone/room data files:

```bash
cargo run --bin schema
```

Pipe to a file for use with editors or AI agents:

```bash
cargo run --bin schema > schema/zone.json
```
