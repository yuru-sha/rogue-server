---
name: verify-rogue-server
description: Verify rogue-server's HTTP health endpoint, JSON/WebSocket game protocol, and ASCII CLI through real local processes; use after changing startup, transport, commands, turns, or rendered game state.
---

# Verify rogue-server

## Launch

The primary user path is the server-authoritative game at `/ws`; `rogue-cli` is the reference terminal client. Start only a server owned by this run. The game has no authentication or persisted state. Each WebSocket connection owns one game.

Launch uses a named supervised service so a single Bash call owns the lifecycle and the listener PID is the recorded ownership token. From the repository root, choose a unique run ID and unused port:

```sh
RUN_ID=verify-20261002 PORT=18083
cargo build --bin rogue-server
```

Then start the server via the named service tool:

```json
{"command":"env ROGUE_LISTEN=127.0.0.1:$PORT target/debug/rogue-server","name":"rogue-verify-$RUN_ID","ready":{"port":$PORT,"host":"127.0.0.1","timeout":30}}
```

The service reports `ready` once the port answers. The supervisor records the listener PID; reference it as `$SERVICE_PID` (the runtime reports it as the `pid` in the ready message). Capture it for cleanup:

```sh
echo "$SERVICE_PID" > "/tmp/rogue-verify-$RUN_ID/server.pid"
mkdir -p "/tmp/rogue-verify-$RUN_ID"
test "$(curl --fail --silent "http://127.0.0.1:$PORT/health")" = ok
```

Ready means `/health` returns exactly `ok`; the listener PID owns this run's port. The default address is `127.0.0.1:8080`; choose a unique port to avoid collisions. Seed game connections with `/ws?seed=42` for repeatable initial placement. If startup never becomes healthy, stop and inspect the supervised service output.

## Doctor

Run this read-only check before driving whenever ownership or responses look wrong. Set the same run ID and port as Launch, with the recorded listener PID:

```sh
RUN_ID=verify-20261002 PORT=18083 sh -eu -c '
  pid=$(cat "/tmp/rogue-verify-$RUN_ID/server.pid")
  kill -0 "$pid"
  lsof -nP -a -p "$pid" -iTCP:"$PORT" -sTCP:LISTEN
  test "$(curl --fail --silent "http://127.0.0.1:$PORT/health")" = ok
  printf "healthy server pid=%s port=%s\n" "$pid" "$PORT"
'
```

Stop if the recorded PID is not listening on this port or health does not return `ok`. Do not drive a server whose ownership cannot be established.

## Drive

Node.js 24 or newer provides the built-in WebSocket API; no npm packages are needed. Use this public WebSocket interaction from the repository root:

```sh
export RUN_ID=verify-20261002 PORT=18083
node skills/verify-rogue-server/scripts/drive-session.mjs \
  "ws://127.0.0.1:$PORT/ws?seed=42" \
  "artifacts/verification/$RUN_ID/session.json"
```

The script records the initial snapshot, sends `inventory` and checks turn 0, sends `wait` and checks turn 1, then sends `quit` and checks the ended response. It exits nonzero on a failed assertion and writes the complete command/response sequence, map, and player state to the evidence file.

To verify the terminal-specific client path, build with `cargo build --bin rogue-cli`, then run `ROGUE_WS="ws://127.0.0.1:$PORT/ws?seed=42" target/debug/rogue-cli` in an interactive terminal. It renders the map and accepts keyboard input; `q` quits. Capture the keypress and resulting terminal output. This is separate from the scripted WebSocket proof.

## Evidence

Keep proof in `artifacts/verification/<RUN_ID>/`, outside the temporary directory removed by Cleanup. Capture the action and returned state, not just the final screen. The scripted session artifact contains both commands and responses; include run ID, URL/seed, command exit status, and relevant server log output in the handoff. Capture the health response and CLI help output when those paths change.

Use the real `/health`, `/ws`, or CLI path. Do not substitute direct library calls, internal state changes, tests, or test-only endpoints. Verify observable transitions: inventory leaves the turn unchanged, wait increments it, and quit ends that connection. No external services are involved, so no mocks are needed.

## Cleanup

Stop only the supervised service started by this run, never a process selected by name. Then remove its temporary PID and log directory:

```sh
RUN_ID=verify-20261002 sh -eu -c '
  pid=$(cat "/tmp/rogue-verify-$RUN_ID/server.pid")
  kill "$pid" 2>/dev/null || true
  while kill -0 "$pid" 2>/dev/null; do sleep 0.1; done
  rm -rf "/tmp/rogue-verify-$RUN_ID"
'
```

If the run used a supervisor, prefer its `kill` action over `kill "$pid"` so the supervisor records the stop. Keep `artifacts/verification/<RUN_ID>/` after cleanup. If the PID file is missing or the recorded PID no longer owns the port, stop and inspect state instead of killing by process name.

## Helpers

`skills/verify-rogue-server/scripts/drive-session.mjs` is executable and uses Node's built-in WebSocket client. Its invocation is the Drive command above; run `node --check skills/verify-rogue-server/scripts/drive-session.mjs` to check JavaScript syntax.