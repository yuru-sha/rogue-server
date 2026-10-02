# ASCII reference client

`rogue-cli` connects to the game server, renders an ASCII dungeon map, and accepts Rogue-style keyboard commands.

## Sub-features

- `cli-help` prints usage and the default WebSocket URL.
- `cli-connect` connects to the configured server and renders the initial map.
- `cli-quit` accepts `q` and exits after the game ends.

## How to get to it (user POV)

- Run `cargo run --bin rogue-cli -- --help` for help.
- Run `ROGUE_WS=ws://127.0.0.1:<port>/ws cargo run --bin rogue-cli` in a terminal.
- Enter `q` at the game prompt to quit.

## Driving it with an interactive terminal

Preconditions:

- Follow Launch and Doctor in `../SKILL.md`.
- Build with `cargo build --bin rogue-cli`.
- Run `ROGUE_WS="ws://127.0.0.1:$PORT/ws?seed=42" target/debug/rogue-cli` in an interactive terminal and capture its transcript.

- **Check help.** Run `cargo run --bin rogue-cli -- --help`. Exit status is 0; output names the default WebSocket URL and `ROGUE_WS` override.
- **Connect and render.** Start the client command above. Capture the status display and map, including `@` at the player position.
- **Quit.** Enter `q`; capture the keypress and resulting ended game/process exit.

## Gotchas

- A pipe-only test may not verify terminal rendering or keyboard behavior.
- Set `ROGUE_WS` to the server started by this run. The default target uses port 8080.
- Preserve the transcript after stopping the client and server. WebSocket driver proof does not cover CLI rendering.