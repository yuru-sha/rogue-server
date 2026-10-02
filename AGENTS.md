# AGENTS.md

## Project

`rogue-server` is a Rust server-authoritative, turn-based dungeon crawler
inspired by the 1980 classic Rogue. Clients send commands over WebSocket and
render the state; the server owns rules, turns, and random outcomes.

## Repository boundaries

- Keep game rules and deterministic random behavior in `src/lib.rs`.
- Keep HTTP/WebSocket startup and transport concerns in `src/main.rs`.
- Keep the reference ASCII client in `src/bin/rogue-cli.rs`.
- Do not add persistence, accounts, rankings, or a mobile client to the MVP.
- Do not copy personal or generic capability packs into this repository.
  `.agents/`, `.codex/`, and `agent-capabilities` content is local-only.

## Change workflow

1. Read `docs/SPEC.md` and `docs/MVP.md` before changing behavior.
2. Add a focused test for non-trivial behavior before implementing it.
3. Make the smallest implementation that satisfies the test and specification.
4. Refactor only after the tests pass.
5. Update `docs/SPEC.md`, `docs/MVP.md`, or `README.md` when the public behavior or
   supported command changes.

## Branch And Pull Request Workflow

- Do not edit, commit, or push directly to `main`. Make changes on a feature branch and merge them through a pull request.
- Direct work on `main` is allowed only when the user explicitly authorizes it.

## Protocol invariants

- WebSocket endpoint: `/ws`; health endpoint: `/health`.
- Messages are JSON and include `version` and `request_id`.
- Invalid input returns an error and never consumes a turn.
- Successful player actions consume one turn; enemy actions follow them.
- Each WebSocket connection owns one game.
- Preserve deterministic initial setup for the same `seed`.

## Verification

Run the documented full gate before handoff:

```sh
cargo fmt -- --check
cargo test --all-targets
cargo clippy --all-targets --all-features -- -D warnings
cargo check --locked --all-targets
git diff --check
```

For runtime changes, also check `GET /health` and the CLI help command:

```sh
cargo run
curl --fail http://127.0.0.1:8080/health
cargo run --bin rogue-cli -- --help
```

Never commit secrets, real user data, local databases, build output, or
machine-specific configuration.

## GitHub workflow

- GitHub Issues are the canonical work tracker.
- Shared Bug / Feature / Question forms and the default Pull Request template are inherited from `yuru-sha/.github`.
- Shared non-default labels, including `orca:*`, are synchronized from `yuru-sha/project-template`.
- Use `orca:*` labels only for ORCA execution state; do not treat them as release categories.

## Commit Messages

- Follow the commit-message policy in `CONTRIBUTING.md`.
- Keep changes focused and preserve unrelated work.
