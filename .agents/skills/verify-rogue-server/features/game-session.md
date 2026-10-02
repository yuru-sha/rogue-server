# Game session

A connection to `/ws` starts a server-owned game and returns the initial map and player state. An optional seed makes initial placement repeatable.

## Sub-features

- `session-snapshot` returns a version-1 snapshot with map, player state, and floor.
- `seeded-start` accepts `/ws?seed=42` for repeatable initial setup.
- `connection-ownership` gives each WebSocket connection a separate game; disconnect ends that game.

## How to get to it (user POV)

- Connect a game client to `ws://127.0.0.1:<port>/ws`.
- Add `?seed=42` to request deterministic initial placement.

## Driving it with the verification driver

Preconditions:

- Follow Launch and Doctor in `../SKILL.md`. Use the owned port and a unique run ID.

- **Connect and inspect the initial state.** Run the driver command in `../SKILL.md`. It checks an active version-1 snapshot; the evidence records the map, seed, player stats, and command sequence.
- **End the connection's game.** The driver sends `quit` and checks `ended: true` and `result: "quit"` in the response.

## Gotchas

- Use `seed=42` when comparing initial state across runs.
- Reconnecting starts a new game. Game state is not persisted between connections.
- `/health` alone does not establish that a game starts correctly.