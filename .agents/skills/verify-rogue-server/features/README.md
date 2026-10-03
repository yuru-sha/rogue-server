# rogue-server verification map

This directory tracks user-visible rogue-server behavior. Read the feature file for the path under test. The primary path is the server-authoritative JSON/WebSocket game; `rogue-cli` is the reference terminal client.

## Baseline preconditions

- Follow `../SKILL.md` Launch and Doctor using a unique run ID and unused port.
- Drive only the server started by this verification run. Each WebSocket connection owns one game.
- Use `/ws?seed=42` for repeatable initial placement. The app has no auth or persisted game data.
- Keep proof in `artifacts/verify-rogue-server/<RUN_ID>/`; cleanup removes only `/tmp/rogue-verify-<RUN_ID>`.

## Driving conventions

- Public routes: `GET /health` and `GET /ws`; messages are versioned JSON.
- Use `node .agents/skills/verify-rogue-server/scripts/drive-session.mjs <ws-url> <evidence-json>` for scripted protocol checks.
- Use a real interactive terminal for CLI rendering and keyboard behavior.
- Record the action and response, including turn and ended state. A WebSocket proof does not verify CLI rendering.

## Features

- [Health and startup](./health-startup.md) covers process readiness and port ownership.
- [Game session](./game-session.md) covers initial snapshots, seeded starts, and connection ownership.
- [Turns and commands](./turns-commands.md) covers inventory, wait, and quit.
- [ASCII reference client](./ascii-client.md) covers help, map rendering, and keyboard quit.