# Repository Guidelines

## Project Overview

`rogue-server` is a Rust 2024 server-authoritative, turn-based dungeon crawler inspired by the 1980 classic Rogue. The server owns game rules, state, turns, and random outcomes; clients exchange JSON commands and state snapshots over WebSocket. The MVP includes a deterministic two-floor world and three enemy types.

## Architecture & Data Flow

- `src/lib.rs` is the domain and protocol layer: `Game` owns mutable gameplay and per-floor state; `Game::handle` validates requests, dispatches commands, applies turn effects, and produces response snapshots or structured errors.
- Preserve the centralized turn sequence in `Game::handle`: successful turn-consuming actions trigger status/trap/stair effects, increment the turn, advance hunger, run enemy actions, then update effects. Invalid actions and inspection-only operations must not consume turns.
- `src/main.rs` owns Axum HTTP/WebSocket startup and transport. Each WebSocket connection creates its own seeded `Game`, sends an initial snapshot, then passes decoded command requests to the library.
- `src/bin/rogue-cli.rs` is a protocol client, not a second rules implementation. Keep game rules and deterministic randomness in the library; keep socket lifecycle and HTTP concerns in the server.

## Key Directories

- `src/`: library, server binary, and reference ASCII client.
- `docs/`: `SPEC.md` defines behavior/protocol; `MVP.md` defines acceptance criteria and exclusions.
- `.github/`: repository workflow and release configuration.

## Development Commands

```sh
cargo run                                      # server at 127.0.0.1:8080
cargo run --bin rogue-cli                      # ASCII WebSocket client
cargo run --bin rogue-cli -- --help            # client controls
ROGUE_WS=ws://127.0.0.1:9000/ws cargo run --bin rogue-cli
```

Set `ROGUE_LISTEN` to change the server listen address. For reproducible initial setup, connect to `/ws?seed=<u64>`; the default seed is `1`.

Before handoff, run the repository gate:

```sh
cargo fmt -- --check
cargo test --all-targets
cargo clippy --all-targets --all-features -- -D warnings
cargo check --locked --all-targets
git diff --check
```

For runtime/transport changes, start the server and verify `GET /health`, and run the CLI help command as a smoke check.

## Code Conventions & Common Patterns

- Follow Rust formatting (`rustfmt`); use descriptive `snake_case` test/function names and idiomatic Rust type names.
- Keep state transitions explicit and centralized. Treat `Game` as authoritative; client/UI code consumes protocol snapshots.
- Protocol types are serde-driven. Requests carry `version` and `request_id`; commands use the tagged JSON `type` field. Return structured protocol errors for invalid input rather than consuming a turn.
- Propagate startup and CLI failures through `Result`; handle message-level JSON/transport errors at the transport boundary.
- Keep dependencies and abstractions minimal; use existing Tokio/Axum patterns.

## Important Files

- `src/lib.rs` — game model, rules, JSON protocol types, and unit tests.
- `src/main.rs` — `/health`, `/ws`, connection/game lifecycle, and `ROGUE_LISTEN`.
- `src/bin/rogue-cli.rs` — reference WebSocket client; `ROGUE_WS` selects its endpoint.
- `docs/SPEC.md`, `docs/MVP.md` — behavior contract and MVP boundaries.
- `README.md` — usage, command examples, and runtime configuration.
- `Cargo.toml` / `Cargo.lock` — package/dependencies and pinned resolution.

## Runtime/Tooling Preferences

Use Cargo and the Rust toolchain; `Cargo.toml` uses edition 2024 and `Cargo.lock` is committed. There is no separate JavaScript runtime, package manager, or build system. No toolchain pin or `.cargo` configuration is declared; follow the installed stable Rust toolchain and repository commands.

## Testing & QA

Tests are Rust unit tests in the `#[cfg(test)]` module at the end of `src/lib.rs`; there is no separate integration-test directory. Add a focused test for non-trivial game behavior, preferably exercising state and response/turn-consumption boundaries through seeded `Game` instances and the existing request helper. Preserve deterministic setup for identical seeds. Current coverage spans world generation, turns, enemies, inventory/equipment, consumables, combat, hunger, traps, and game endings.

Read `docs/SPEC.md` and `docs/MVP.md` before behavior changes. Update the relevant specification, MVP contract, or README when public behavior or supported commands change. Keep persistence, accounts, rankings, and mobile clients out of MVP scope.

## Repository Workflow

Do not work directly on `main` unless explicitly authorized. Keep changes focused, follow `CONTRIBUTING.md`'s commit-message policy, and do not commit secrets, user data, build output, or machine-specific configuration. `.agents/`, `.codex/`, and generic capability-pack content are local-only; do not copy them into project deliverables.
