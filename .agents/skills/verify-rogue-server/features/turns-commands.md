# Turns and commands

The player sends versioned JSON commands to the game. Inventory inspection is informational; a successful wait consumes one turn.

## Sub-features

- `inspect-inventory` returns inventory details without changing the turn.
- `wait-turn` consumes one turn and returns an updated snapshot.
- `quit-game` ends the connection's game.

## How to get to it (user POV)

- Connect to `/ws` and send JSON commands with `version`, `request_id`, and `command`.
- Use command types `inventory`, `wait`, and `quit`.

## Driving it with the verification driver

Preconditions:

- Follow Launch and Doctor in `../SKILL.md`. Use the run's owned server and unique evidence directory.

- **Inspect inventory.** Run the driver command in `../SKILL.md`. It sends the public `inventory` command and checks that the response message begins `Inventory:` and the turn remains 0.
- **Consume a turn.** The driver sends `wait` and checks an active snapshot at turn 1.
- **Quit.** The driver sends `quit` and checks `ended: true`, `result: "quit"`. Inspect all recorded command/response entries in `artifacts/verification/$RUN_ID/session.json`.

## Gotchas

- Invalid commands must not consume turns; this recipe covers valid inventory and wait behavior.
- The driver starts a fresh game, so its first turn is 0.
- A final screen without the command and response sequence is insufficient proof.