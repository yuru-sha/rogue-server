# Repository Guidelines

## Project Overview

`rogue-server` is a Go server-authoritative, turn-based dungeon crawler inspired by the 1980 classic Rogue. The server owns game rules, state, turns, and random outcomes; clients exchange JSON commands and state snapshots over WebSocket. The MVP includes a deterministic two-floor world and three enemy types.

## Architecture & Data Flow

- `internal/game/` is the domain and protocol layer: `Game` owns mutable gameplay and per-floor state; `Game.HandleJSON` validates requests, dispatches commands, applies turn effects, and produces response snapshots or structured errors.
- Preserve the centralized turn sequence in `Game.HandleJSON`: successful turn-consuming actions trigger status/trap/stair effects, increment the turn, advance hunger, run enemy actions, then update effects. Invalid actions and inspection-only operations must not consume turns.
- `cmd/rogue-server/` owns HTTP/WebSocket startup and transport. Each WebSocket connection creates its own seeded `Game`, sends an initial snapshot, then passes decoded command requests to the library.
- `cmd/rogue-cli/` is a protocol client, not a second rules implementation. Keep game rules and deterministic randomness in the library; keep socket lifecycle and HTTP concerns in the server.

## Key Directories

- `internal/game/`: game model, rules, protocol, and unit tests.
- `cmd/`: server and reference ASCII client.
- `docs/`: `SPEC.md` defines behavior/protocol; `MVP.md` defines acceptance criteria and exclusions.
- `.github/`: repository workflow and release configuration.

## Code, Tests, Commits, and Comments

- Code should make how the behavior is implemented clear.
- Tests should make what behavior is expected clear.
- Commit messages should explain why the change is needed.
- Comments should explain why not an apparently reasonable alternative was chosen, including relevant constraints or non-obvious intent. Do not use comments to paraphrase the code's behavior.

## Branch And Pull Request Workflow

## Development Commands

```sh
go run ./cmd/rogue-server                         # server at 127.0.0.1:8080
go run ./cmd/rogue-cli                            # ASCII WebSocket client
go run ./cmd/rogue-cli --help                     # client controls
ROGUE_WS=ws://127.0.0.1:9000/ws go run ./cmd/rogue-cli
```

Set `ROGUE_LISTEN` to change the server listen address. For reproducible initial setup, connect to `/ws?seed=<u64>`; the default seed is `1`.

Before handoff, run the repository gate:

```sh
go fmt ./...
go test ./...
go vet ./...
go build ./...
git diff --check
```

For runtime/transport changes, start the server and verify `GET /health`, and run the CLI help command as a smoke check.

## Code Conventions & Common Patterns

- Follow `gofmt`; use descriptive exported Go identifiers and idiomatic Go naming.
- Keep state transitions explicit and centralized. Treat `Game` as authoritative; client/UI code consumes protocol snapshots.
- Protocol types use `encoding/json`. Requests carry `version` and `request_id`; commands use the tagged JSON `type` field. Return structured protocol errors for invalid input rather than consuming a turn.
- Propagate startup and CLI failures; handle message-level JSON/transport errors at the transport boundary.
- Keep dependencies and abstractions minimal; use the standard `net/http` server and the Gorilla WebSocket package.

## Important Files

- `internal/game/` — game model, rules, JSON protocol types, and unit tests.
- `cmd/rogue-server/main.go` — `/health`, `/ws`, connection/game lifecycle, and `ROGUE_LISTEN`.
- `cmd/rogue-cli/main.go` — reference WebSocket client; `ROGUE_WS` selects its endpoint.
- `docs/SPEC.md`, `docs/MVP.md` — behavior contract and MVP boundaries.
- `README.md` — usage, command examples, and runtime configuration.
- `go.mod` / `go.sum` — Go module and pinned dependencies.

## Runtime/Tooling Preferences

Use Go 1.27.4 as declared by `go.mod`; `go.sum` pins external modules.

## Testing & QA

Tests are Go tests in package files ending with `_test.go`. Add focused tests for non-trivial game behavior, preferably exercising state and response/turn-consumption boundaries through seeded `Game` instances. Preserve deterministic setup for identical seeds. Coverage should span world generation, turns, enemies, inventory/equipment, consumables, combat, hunger, traps, and game endings.

Read `docs/SPEC.md` and `docs/MVP.md` before behavior changes. Update the relevant specification, MVP contract, or README when public behavior or supported commands change. Keep persistence, accounts, rankings, and mobile clients out of MVP scope.

## Repository Workflow

Do not work directly on `main` unless explicitly authorized. Keep changes focused, follow `CONTRIBUTING.md`'s commit-message policy, and do not commit secrets, user data, build output, or machine-specific configuration. `.agents/`, `.codex/`, and generic capability-pack content are local-only; do not copy them into project deliverables.
