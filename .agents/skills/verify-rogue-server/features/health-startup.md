# Health and startup

The operator starts the local server and checks `/health` to confirm it is ready to accept games.

## Sub-features

- `health-ready` returns `ok` from the configured loopback address.
- `owned-process` confirms the verification run's recorded PID owns the selected port.

## How to get to it (user POV)

- Start the server with `ROGUE_LISTEN=127.0.0.1:<port>`.
- Request `GET /health` with curl.

## Driving it with curl and lsof

Preconditions:

- Follow Launch in `../SKILL.md` with a unique run ID and unused port.

- **Check readiness and ownership.** Run Doctor in `../SKILL.md`. `lsof` must show the recorded PID listening on the selected port, and curl must return exactly `ok`.
- **Capture proof.** Save the command, body, PID, port, and relevant server log at `artifacts/verify-rogue-server/<RUN_ID>/health.txt`.

## Gotchas

- A healthy response from an unknown process is not proof. Confirm the PID owns the port.
- Port 8080 is the default. Choose an unused port for verification.
- Health does not prove WebSocket behavior. Verify `/ws` separately.