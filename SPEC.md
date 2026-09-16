# Rogue Server Specification

## Goal

`rogue-server` is a server-authoritative, turn-based dungeon crawler inspired
by the 1980 classic Rogue. Clients send commands and render the state; the
server owns all game rules and random outcomes.

## Transport

- Rust single binary.
- WebSocket endpoint: `GET /ws`.
- Optional deterministic seed: `/ws?seed=<u64>`.
- Health endpoint: `GET /health`.
- Default listen address: `127.0.0.1:8080`, overridden by `ROGUE_LISTEN`.
- Messages are JSON with `version`, `request_id`, and a command or response
  `type`.
- Invalid JSON, unsupported commands, and invalid moves return an error and do
  not consume a turn.
- One connection owns one game. Disconnecting ends that game.

## Turn rules

- A successful player action consumes one turn.
- Enemy actions run after the player action.
- Invalid actions and inspection-only operations do not consume a turn.
- Search reveals untriggered traps within two tiles of the player without
  consuming a turn.
- Inventory inspection reports carried item counts without consuming a turn.
- The drop command returns one named carried item (food, potion, scroll,
  amulet, weapon, or armor) to the current tile and consumes one turn.
- Cursed weapons, armor, rings, and staffs are unidentified until inspected;
  identifying an item reports whether it is cursed without consuming a turn.
- A cursed armor item has a negative defense modifier and cannot be removed
  after equipping.
- Equipped non-cursed weapons and armor can be removed; cursed equipment
  cannot be removed.
- Movement supports eight directions. Diagonal corner cutting is forbidden.
- Moving into an enemy performs a melee attack.
- Waiting heals one HP up to maximum HP.

## World

- Logical map size is 80x24.
- Floors contain rooms connected by corridors.
- The initial implementation supports a deterministic two-floor round trip.
- Each floor preserves its map, enemies, items, traps, and explored state
  while the player is elsewhere.
- Floor 1 has a down staircase; floor 2 has an up staircase.
- The same seed must reproduce the same initial map and enemy placement.
- Enemies do not start adjacent to the player and only occupy floor cells.
- Enemies pursue or attack only within eight tiles when a straight or diagonal
  line to the player is unobstructed.
- Trolls act every other turn; Goblins and Bats act each turn.
- Trolls regenerate one HP whenever they act, up to their maximum HP.
- Goblins and Trolls cannot cut diagonal corners; Bats may fly through them.
- A wounded Bat with one HP retreats from the player when it can see them.

## Player and combat

- Initial player state: 20 HP, level 1, zero XP, hunger 100.
- Defeated enemies award 10 XP.
- Each level requires `level * 20` XP, up to level 12.
- A level-up increases maximum HP by 2 and restores HP.
- Food reduces hunger by one per consumed turn and restores 25 hunger when
  eaten, capped at 100.
- A healing potion restores 5 HP when drunk, capped at maximum HP, and
  consumes one turn.
- A strength potion grants +2 attack for the next 10 consumed turns.
- A mapping scroll reveals the entire current floor and consumes one turn.
- A teleport scroll moves the player to an unoccupied floor tile and consumes
  one turn.
- Starvation damages the player by 1 HP per consumed turn at hunger 0.
- A weapon can be picked up and equipped. The initial weapon bonus is 0 and
  the placed weapon provides +1 attack bonus.
- Armor can be picked up and equipped. The placed armor provides +1 defense
  bonus.
- A protection ring can be picked up and equipped. It provides +1 defense
  bonus and can be removed unless cursed.
- A striking staff can be picked up and equipped. It provides +3 attack
  bonus and can be removed unless cursed.
- The inventory has 20 slots. Items with identical kind and metadata stack in
  one slot; items with different bonuses or identification/curse metadata
  remain separate. One pickup consumes at most one item from a tile.
- An amulet on floor 2 is the victory item. Picking it up and ascending to
  floor 1 ends the game with victory.
- Armor reduces incoming damage, with damage still bounded below by 1.
- The player starts with three arrows and may shoot orthogonally in a straight
  line. Each shot consumes one arrow and hits the first enemy before a wall.
- Damage is never less than 1.
- The initial dungeon contains hidden poison and fire traps. Stepping on a
  poison trap deals 1 damage and applies poison for the next three consumed
  turns; each of those turns deals 1 damage. A fire trap deals 4 immediate
  damage. Traps trigger only once.
- The MVP enemy roster contains Goblin (`g`), Bat (`b`), and Troll (`T`) with
  distinct stats and XP rewards.

## End state and score

- Death occurs at HP 0.
- Quit, death, and victory produce an ended response and close the connection.
- Scores use victory bonus, floor, defeated enemies, HP, hunger, and turns.
- Persistent saves, accounts, rankings, and network services are out of scope.

## Clients

- `rogue-cli` is the reference ASCII client.
- It supports Rogue-style movement keys, wait, pickup, eat, drink, equip,
  read-scroll, inventory, shoot, search, and quit.
- Other clients may use the same JSON/WebSocket contract.
