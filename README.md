# rogue-server

[English](README.md) | [日本語](README.ja.md)

[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/yuru-sha/rogue-server)

The MVP currently includes three enemy types in addition to the core combat loop.

A minimal Rogue-like, turn-based dungeon server. The server owns the game state, while clients exchange commands and snapshots over JSON via WebSocket. It currently supports round trips between floors 1 and 2 with per-floor state, a victory item, line-of-sight enemy AI, enemy-specific action speeds and movement traits, Troll regeneration, retreating wounded Bats, food, hunger, experience, leveling, weapon, armor, ring, and staff equipment, ranged attacks, poison and fire traps, healing and strength potions, mapping and teleport scrolls, consumable item stacking, and end-game scoring.

## Running

```sh
cargo run
curl http://127.0.0.1:8080/health
```

In another terminal, start the ASCII client:

```sh
cargo run --bin rogue-cli
```

See the available controls with:

```sh
cargo run --bin rogue-cli -- --help
```

Set `ROGUE_WS` to change the connection target:

```sh
ROGUE_WS=ws://127.0.0.1:9000/ws cargo run --bin rogue-cli
```

Set `ROGUE_LISTEN` to change the listen address.

## Command examples

```json
{"version":1,"request_id":"1","command":{"type":"move","dx":1,"dy":0}}
{"version":1,"request_id":"2","command":{"type":"wait"}}
{"version":1,"request_id":"3","command":{"type":"pickup"}}
{"version":1,"request_id":"4","command":{"type":"eat"}}
{"version":1,"request_id":"5","command":{"type":"equip"}}
{"version":1,"request_id":"6","command":{"type":"equip_armor"}}
{"version":1,"request_id":"7","command":{"type":"shoot","dx":1,"dy":0}}
{"version":1,"request_id":"8","command":{"type":"search"}}
{"version":1,"request_id":"9","command":{"type":"drink"}}
{"version":1,"request_id":"10","command":{"type":"quit"}}
```

Pass a seed in the WebSocket URL's `seed` query parameter to reproduce the map and initial placements.

Use `inventory` to inspect your inventory and `drop` to drop one carried item.
`drop` accepts one of `food`, `potion`, `strength_potion`, `scroll`, `teleport_scroll`, `amulet`, `weapon`, `armor`, `ring`, or `staff`.
Use `identify` with `weapon`, `armor`, `ring`, or `staff` to identify an item. Use `teleport` to use a teleport scroll.
Items of the same type and metadata stack in one slot. Using or dropping an item removes one at a time. Items with different bonuses or identification/curse states remain in separate slots.

```text
ws://127.0.0.1:8080/ws?seed=42
```

## GitHub Release

See [docs/agents/release.md](docs/agents/release.md) for the release note format and creation procedure. The shared body template is [.github/release-notes-template.md](.github/release-notes-template.md), and the generated-note categories are managed in [.github/release.yml](.github/release.yml).
