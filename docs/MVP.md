# MVP Acceptance and Review Contract

This file records the current implementation boundary. It is deliberately
smaller than a full classic-Rogue remake.

## Accepted now

- [x] Server starts on the configured loopback address.
- [x] `/health` returns `ok`.
- [x] `/ws` starts one server-authoritative game per connection.
- [x] Optional URL seed is accepted and exposed in snapshots.
- [x] Same seed reproduces the initial map and enemy positions.
- [x] 80x24 room-and-corridor map with valid floor-only enemy placement.
- [x] Eight-direction movement with wall and corner-cutting validation.
- [x] Wait, melee attack, enemy turn, death, and quit.
- [x] Food pickup, food inventory, eating, hunger drain, and starvation damage.
- [x] Healing potion pickup, drinking, and bounded recovery.
- [x] Strength potion pickup and temporary attack boost.
- [x] Mapping scroll pickup and full current-floor reveal.
- [x] Teleport scroll pickup and safe random relocation.
- [x] Non-consuming inventory inspection.
- [x] Dropping one carried item onto the current floor tile.
- [x] Unidentified cursed weapon, armor, ring, and staff with non-consuming identification.
- [x] Equipment removal with cursed-item restriction.
- [x] Cursed armor with a negative defense modifier.
- [x] Weapon pickup, equipment, and attack bonus.
- [x] Armor pickup, equipment, defense bonus, and 20-slot inventory limit.
- [x] Protection ring pickup, equipment, defense bonus, and removal.
- [x] Striking staff pickup, equipment, attack bonus, and removal.
- [x] Multiple items on one tile are picked up one at a time.
- [x] Items with identical kind and metadata stack in one inventory slot.
- [x] XP, level-up, defeated count, and score reporting.
- [x] Three enemy kinds with distinct stats, symbols, and XP rewards.
- [x] Straight-line ranged attacks with finite arrows.
- [x] One-shot poison trap with three-turn poison status.
- [x] One-shot fire trap with immediate damage.
- [x] Search reveals untriggered traps within two tiles without consuming a turn.
- [x] Two-floor down/up staircase traversal.
- [x] Persistent map, enemy, item, trap, and explored state per floor.
- [x] Amulet pickup and victory on returning to the surface.
- [x] Enemies require short-range straight-line or diagonal line of sight.
- [x] Trolls have a slower, every-other-turn action cadence.
- [x] Trolls regenerate one HP on their action turns.
- [x] Bats can cross diagonal corners while flying.
- [x] Wounded Bats retreat from the player.
- [x] ASCII CLI smoke path over WebSocket.

## Not accepted yet

- [x] Additional enemy AI: Troll regeneration and wounded-Bat retreat.
- [x] Broader search, fire traps, strength and teleport scrolls, rings, staffs,
      and identification/curse rules for all current equipment.
- [x] Items with distinct metadata remain separate inventory stacks.

## Explicitly out of scope

- Persistent death records, saves, accounts, rankings, and mobile UI remain
  outside this server MVP; docs/SPEC.md explicitly excludes them.

## Review checklist

Every implementation cycle is reviewed against:

1. `SPEC.md`: required behavior, protocol, state transitions, and scope.
2. This file: accepted and deferred MVP criteria.
3. Public seams: `Game::handle`, JSON serialization, `/health`, `/ws`, and
   the CLI smoke path.
4. Automated evidence: focused TDD tests, `cargo fmt -- --check`,
   `cargo test --all-targets`, `cargo clippy --all-targets --all-features -- -D warnings`,
   and `cargo check --locked --all-targets`.

A passing test suite does not by itself prove SPEC compliance. Missing
acceptance items and behavior outside the scope are reported separately.
