use serde::{Deserialize, Serialize};

pub const WIDTH: i32 = 80;
pub const HEIGHT: i32 = 24;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tile {
    Wall,
    Floor,
    Up,
    Down,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EnemyKind {
    Goblin,
    Bat,
    Troll,
}

impl EnemyKind {
    fn from_roll(roll: u64) -> Self {
        match roll {
            0 => Self::Goblin,
            1 => Self::Bat,
            _ => Self::Troll,
        }
    }
    fn stats(self) -> (i32, i32, i32, u32, char) {
        match self {
            Self::Goblin => (5, 2, 0, 10, 'g'),
            Self::Bat => (3, 1, 0, 8, 'b'),
            Self::Troll => (12, 3, 1, 20, 'T'),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Enemy {
    pub id: u32,
    pub pos: Pos,
    pub kind: EnemyKind,
    pub hp: i32,
    pub attack: i32,
    pub defense: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Item {
    pub pos: Pos,
    pub kind: ItemKind,
    pub identified: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemKind {
    Food,
    Weapon { bonus: i32 },
    Armor { bonus: i32 },
    Ring { bonus: i32 },
    Staff { bonus: i32 },
    Potion,
    StrengthPotion,
    Amulet,
    MappingScroll,
    TeleportScroll,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Trap {
    pub pos: Pos,
    pub kind: TrapKind,
    pub triggered: bool,
    pub revealed: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrapKind {
    Poison,
    Fire,
}

#[derive(Clone, Debug)]
struct InventoryItem {
    kind: ItemKind,
    identified: bool,
    cursed: bool,
    quantity: u32,
}

#[derive(Clone, Debug)]
struct FloorState {
    tiles: Vec<Tile>,
    stair: Pos,
    up_stair: Pos,
    enemies: Vec<Enemy>,
    items: Vec<Item>,
    traps: Vec<Trap>,
    known: Vec<bool>,
}

#[derive(Clone, Debug)]
pub struct Game {
    pub seed: u64,
    pub tiles: Vec<Tile>,
    pub player: Pos,
    pub stair: Pos,
    pub hp: i32,
    pub max_hp: i32,
    pub enemies: Vec<Enemy>,
    pub turn: u64,
    pub ended: bool,
    pub result: Option<String>,
    pub score: Option<i32>,
    pub floor: u8,
    pub up_stair: Pos,
    pub hunger: i32,
    pub items: Vec<Item>,
    pub food: u32,
    pub weapons: u32,
    pub weapon_bonus: i32,
    pub strength_turns: u32,
    weapon_cursed: bool,
    pub armor: u32,
    pub armor_bonus: i32,
    pub ring_bonus: i32,
    pub rings: u32,
    ring_cursed: bool,
    pub staff_bonus: i32,
    pub staffs: u32,
    staff_cursed: bool,
    armor_cursed: bool,
    pub potions: u32,
    pub strength_potions: u32,
    pub amulets: u32,
    pub scrolls: u32,
    pub teleport_scrolls: u32,
    pub arrows: u32,
    pub traps: Vec<Trap>,
    pub poison_turns: u32,
    pub level: u32,
    pub xp: u32,
    pub defeated: u32,
    inventory_items: Vec<InventoryItem>,
    floor_states: [Option<FloorState>; 2],
    known: Vec<bool>,
    rng: Rng,
}
#[derive(Clone, Copy, Debug)]
struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed.max(1))
    }
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.0
    }
    fn range(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

impl Game {
    pub fn new(seed: Option<u64>) -> Self {
        let seed = seed.unwrap_or(1);
        let mut rng = Rng::new(seed);
        let mut tiles = vec![Tile::Wall; (WIDTH * HEIGHT) as usize];
        let rooms = [
            (2, 2, 12, 7),
            (22, 2, 14, 7),
            (46, 2, 14, 7),
            (10, 14, 14, 7),
            (38, 14, 18, 7),
        ];
        for &(x, y, width, height) in &rooms {
            for row in y..y + height {
                for col in x..x + width {
                    tiles[index(Pos { x: col, y: row })] = Tile::Floor;
                }
            }
        }
        for pair in rooms.windows(2) {
            let (x1, y1, w1, h1) = pair[0];
            let (x2, y2, _, h2) = pair[1];
            let mut x = x1 + w1 / 2;
            let target_x = x2 + 2;
            let y = y1 + h1 / 2;
            while x != target_x {
                tiles[index(Pos { x, y })] = Tile::Floor;
                x += (target_x - x).signum();
            }
            let target_y = y2 + h2 / 2;
            let mut row = y;
            while row != target_y {
                tiles[index(Pos {
                    x: target_x,
                    y: row,
                })] = Tile::Floor;
                row += (target_y - row).signum();
            }
        }
        let player = Pos { x: 2, y: 2 };
        let stair = Pos { x: 54, y: 18 };
        let up_stair = Pos { x: 3, y: 2 };
        tiles[index(stair)] = Tile::Down;
        let mut enemies = Vec::new();
        let mut id = 0;
        while enemies.len() < 5 {
            let pos = Pos {
                x: 8 + rng.range(60) as i32,
                y: 3 + rng.range(16) as i32,
            };
            if matches!(tiles[index(pos)], Tile::Floor)
                && pos != player
                && pos != stair
                && (pos.x - player.x).abs() + (pos.y - player.y).abs() > 4
                && !enemies.iter().any(|enemy: &Enemy| enemy.pos == pos)
            {
                let kind = EnemyKind::from_roll(rng.range(3));
                let (hp, attack, defense, _, _) = kind.stats();
                enemies.push(Enemy {
                    id,
                    pos,
                    kind,
                    hp,
                    attack,
                    defense,
                });
                id += 1;
            }
        }
        let items = vec![
            Item {
                pos: Pos { x: 3, y: 2 },
                kind: ItemKind::Food,
                identified: true,
            },
            Item {
                pos: Pos { x: 4, y: 2 },
                kind: ItemKind::Weapon { bonus: 1 },
                identified: true,
            },
            Item {
                pos: Pos { x: 5, y: 2 },
                kind: ItemKind::Armor { bonus: 1 },
                identified: true,
            },
            Item {
                pos: Pos { x: 7, y: 2 },
                kind: ItemKind::Potion,
                identified: true,
            },
            Item {
                pos: Pos { x: 11, y: 2 },
                kind: ItemKind::StrengthPotion,
                identified: true,
            },
            Item {
                pos: Pos { x: 8, y: 2 },
                kind: ItemKind::MappingScroll,
                identified: true,
            },
            Item {
                pos: Pos { x: 12, y: 2 },
                kind: ItemKind::TeleportScroll,
                identified: true,
            },
            Item {
                pos: Pos { x: 13, y: 2 },
                kind: ItemKind::Ring { bonus: 1 },
                identified: true,
            },
            Item {
                pos: Pos { x: 14, y: 2 },
                kind: ItemKind::Staff { bonus: 3 },
                identified: true,
            },
            Item {
                pos: Pos { x: 15, y: 2 },
                kind: ItemKind::Ring { bonus: -1 },
                identified: false,
            },
            Item {
                pos: Pos { x: 16, y: 2 },
                kind: ItemKind::Staff { bonus: -2 },
                identified: false,
            },
            Item {
                pos: Pos { x: 9, y: 2 },
                kind: ItemKind::Weapon { bonus: -1 },
                identified: false,
            },
            Item {
                pos: Pos { x: 10, y: 2 },
                kind: ItemKind::Armor { bonus: -1 },
                identified: false,
            },
        ];
        let traps = vec![
            Trap {
                pos: Pos { x: 6, y: 2 },
                kind: TrapKind::Poison,
                triggered: false,
                revealed: false,
            },
            Trap {
                pos: Pos { x: 5, y: 3 },
                kind: TrapKind::Fire,
                triggered: false,
                revealed: false,
            },
        ];
        let mut game = Self {
            seed,
            tiles,
            player,
            stair,
            hp: 20,
            max_hp: 20,
            enemies,
            turn: 0,
            ended: false,
            result: None,
            score: None,
            floor: 1,
            up_stair,
            hunger: 100,
            items,
            food: 0,
            weapons: 0,
            weapon_bonus: 0,
            strength_turns: 0,
            weapon_cursed: false,
            armor: 0,
            armor_bonus: 0,
            ring_bonus: 0,
            rings: 0,
            ring_cursed: false,
            staff_bonus: 0,
            staffs: 0,
            staff_cursed: false,
            armor_cursed: false,
            potions: 0,
            strength_potions: 0,
            amulets: 0,
            scrolls: 0,
            teleport_scrolls: 0,
            arrows: 3,
            traps,
            poison_turns: 0,
            level: 1,
            xp: 0,
            defeated: 0,
            inventory_items: Vec::new(),
            floor_states: std::array::from_fn(|_| None),
            known: vec![false; (WIDTH * HEIGHT) as usize],
            rng,
        };
        game.reveal();
        game
    }

    pub fn handle(&mut self, request: CommandRequest) -> GameResponse {
        let request_id = request.request_id.clone();
        if request.version != 1 {
            return GameResponse::error("unsupported_version", "version must be 1".into())
                .with_request_id(request_id);
        }
        if self.ended {
            return GameResponse::error("game_ended", "game is already over".into())
                .with_request_id(request_id);
        }
        let mut message = String::new();
        let consumed = match request.command {
            Command::Move { dx, dy }
                if (-1..=1).contains(&dx) && (-1..=1).contains(&dy) && (dx != 0 || dy != 0) =>
            {
                let consumed = self.move_or_attack(dx, dy, &mut message);
                if !consumed {
                    return GameResponse::error("invalid_command", message)
                        .with_request_id(request_id);
                }
                consumed
            }
            Command::Wait => {
                self.hp = (self.hp + 1).min(self.max_hp);
                message = "You wait.".into();
                true
            }
            Command::Pickup => {
                if let Some(i) = self.items.iter().position(|item| item.pos == self.player) {
                    if self.inventory_slots() >= 20 {
                        return GameResponse::error(
                            "inventory_full",
                            "Your inventory is full.".into(),
                        )
                        .with_request_id(request_id);
                    }
                    match self.items[i].kind {
                        ItemKind::Food => self.food += 1,
                        ItemKind::Weapon { .. } => self.weapons += 1,
                        ItemKind::Armor { .. } => self.armor += 1,
                        ItemKind::Ring { .. } => self.rings += 1,
                        ItemKind::Staff { .. } => self.staffs += 1,
                        ItemKind::Potion => self.potions += 1,
                        ItemKind::StrengthPotion => self.strength_potions += 1,
                        ItemKind::Amulet => self.amulets += 1,
                        ItemKind::MappingScroll => self.scrolls += 1,
                        ItemKind::TeleportScroll => self.teleport_scrolls += 1,
                    }
                    let kind = self.items[i].kind;
                    let identified = self.items[i].identified;
                    let cursed = matches!(
                        kind,
                        ItemKind::Weapon { bonus }
                            | ItemKind::Armor { bonus }
                            | ItemKind::Ring { bonus }
                            | ItemKind::Staff { bonus } if bonus < 0
                    );
                    if let Some(record) = self.inventory_items.iter_mut().find(|record| {
                        record.kind == kind
                            && record.identified == identified
                            && record.cursed == cursed
                    }) {
                        record.quantity += 1;
                    } else {
                        self.inventory_items.push(InventoryItem {
                            kind,
                            identified,
                            cursed,
                            quantity: 1,
                        });
                    }
                    self.items.remove(i);
                    message = "You pick up an item.".into();
                    true
                } else {
                    return GameResponse::error(
                        "invalid_command",
                        "There is nothing to pick up here.".into(),
                    )
                    .with_request_id(request_id);
                }
            }
            Command::Eat => {
                if self.food == 0 {
                    return GameResponse::error("invalid_command", "You have no food.".into())
                        .with_request_id(request_id);
                }
                self.food -= 1;
                self.remove_inventory_record(|kind| matches!(kind, ItemKind::Food));
                self.hunger = (self.hunger + 25).min(100);
                message = "You eat the food.".into();
                true
            }
            Command::Drink => {
                if self.potions == 0 {
                    return GameResponse::error("invalid_command", "You have no potion.".into())
                        .with_request_id(request_id);
                }
                self.potions -= 1;
                self.remove_inventory_record(|kind| matches!(kind, ItemKind::Potion));
                self.hp = (self.hp + 5).min(self.max_hp);
                message = "You drink a healing potion.".into();
                true
            }
            Command::DrinkStrength => {
                if self.strength_potions == 0 {
                    return GameResponse::error(
                        "invalid_command",
                        "You have no strength potion.".into(),
                    )
                    .with_request_id(request_id);
                }
                self.strength_potions -= 1;
                self.remove_inventory_record(|kind| matches!(kind, ItemKind::StrengthPotion));
                self.strength_turns = 10;
                message = "You drink a strength potion.".into();
                true
            }
            Command::EquipRing => {
                let Some(bonus) = self
                    .inventory_items
                    .iter()
                    .find_map(|item| match item.kind {
                        ItemKind::Ring { bonus } => Some(bonus),
                        _ => None,
                    })
                else {
                    return GameResponse::error("invalid_command", "You have no ring.".into())
                        .with_request_id(request_id);
                };
                self.rings -= 1;
                self.ring_cursed = self
                    .inventory_items
                    .iter()
                    .find(|item| matches!(item.kind, ItemKind::Ring { .. }))
                    .is_some_and(|item| item.cursed);
                self.remove_inventory_record(|kind| matches!(kind, ItemKind::Ring { .. }));
                self.ring_bonus += bonus;
                message = "You equip the ring.".into();
                true
            }
            Command::UnequipRing => {
                if self.ring_bonus == 0 {
                    return GameResponse::error(
                        "invalid_command",
                        "You have no equipped ring.".into(),
                    )
                    .with_request_id(request_id);
                }
                if self.ring_cursed {
                    return GameResponse::error(
                        "invalid_command",
                        "The cursed ring cannot be removed.".into(),
                    )
                    .with_request_id(request_id);
                }
                self.ring_bonus -= 1;
                self.rings += 1;
                self.inventory_items.push(InventoryItem {
                    kind: ItemKind::Ring { bonus: 1 },
                    identified: true,
                    cursed: false,
                    quantity: 1,
                });
                message = "You remove the ring.".into();
                true
            }
            Command::EquipStaff => {
                let Some(bonus) = self
                    .inventory_items
                    .iter()
                    .find_map(|item| match item.kind {
                        ItemKind::Staff { bonus } => Some(bonus),
                        _ => None,
                    })
                else {
                    return GameResponse::error("invalid_command", "You have no staff.".into())
                        .with_request_id(request_id);
                };
                self.staffs -= 1;
                self.staff_cursed = self
                    .inventory_items
                    .iter()
                    .find(|item| matches!(item.kind, ItemKind::Staff { .. }))
                    .is_some_and(|item| item.cursed);
                self.remove_inventory_record(|kind| matches!(kind, ItemKind::Staff { .. }));
                self.staff_bonus += bonus;
                message = "You equip the staff.".into();
                true
            }
            Command::UnequipStaff => {
                if self.staff_bonus == 0 {
                    return GameResponse::error(
                        "invalid_command",
                        "You have no equipped staff.".into(),
                    )
                    .with_request_id(request_id);
                }
                if self.staff_cursed {
                    return GameResponse::error(
                        "invalid_command",
                        "The cursed staff cannot be removed.".into(),
                    )
                    .with_request_id(request_id);
                }
                self.staff_bonus -= 3;
                self.staffs += 1;
                self.inventory_items.push(InventoryItem {
                    kind: ItemKind::Staff { bonus: 3 },
                    identified: true,
                    cursed: false,
                    quantity: 1,
                });
                message = "You remove the staff.".into();
                true
            }
            Command::ReadScroll => {
                if self.scrolls == 0 {
                    return GameResponse::error(
                        "invalid_command",
                        "You have no mapping scroll.".into(),
                    )
                    .with_request_id(request_id);
                }
                self.scrolls -= 1;
                self.remove_inventory_record(|kind| matches!(kind, ItemKind::MappingScroll));
                self.known.fill(true);
                message = "The dungeon map is revealed.".into();
                true
            }
            Command::Teleport => {
                if self.teleport_scrolls == 0 {
                    return GameResponse::error(
                        "invalid_command",
                        "You have no teleport scroll.".into(),
                    )
                    .with_request_id(request_id);
                }
                let Some(pos) = self.random_open_floor() else {
                    return GameResponse::error(
                        "invalid_command",
                        "There is no safe teleport destination.".into(),
                    )
                    .with_request_id(request_id);
                };
                self.teleport_scrolls -= 1;
                self.remove_inventory_record(|kind| matches!(kind, ItemKind::TeleportScroll));
                self.player = pos;
                message = "You teleport to another part of the dungeon.".into();
                true
            }
            Command::Inventory => {
                let individual = self
                    .inventory_items
                    .iter()
                    .map(|item| {
                        let name = match item.kind {
                            ItemKind::Food => "food",
                            ItemKind::Weapon { .. } => "weapon",
                            ItemKind::Armor { .. } => "armor",
                            ItemKind::Ring { .. } => "ring",
                            ItemKind::Staff { .. } => "staff",
                            ItemKind::Potion => "potion",
                            ItemKind::StrengthPotion => "strength potion",
                            ItemKind::Amulet => "amulet",
                            ItemKind::MappingScroll => "scroll",
                            ItemKind::TeleportScroll => "teleport scroll",
                        };
                        let curse = if item.identified {
                            item.cursed.to_string()
                        } else {
                            "unknown".into()
                        };
                        format!(
                            "{name} x{} (identified={}, cursed={curse})",
                            item.quantity, item.identified
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                message = format!(
                    "Inventory: food {} potions {} strength potions {} scrolls {} teleport scrolls {} amulets {} weapons {} armor {} rings {} staffs {}. Items: {}.",
                    self.food,
                    self.potions,
                    self.strength_potions,
                    self.scrolls,
                    self.teleport_scrolls,
                    self.amulets,
                    self.weapons,
                    self.armor,
                    self.rings,
                    self.staffs,
                    individual
                );
                false
            }
            Command::Identify { item } => {
                let Some(record) = self.inventory_items.iter_mut().find(|record| {
                    !record.identified
                        && match item.as_str() {
                            "weapon" => matches!(record.kind, ItemKind::Weapon { .. }),
                            "armor" => matches!(record.kind, ItemKind::Armor { .. }),
                            "ring" => matches!(record.kind, ItemKind::Ring { .. }),
                            "staff" => matches!(record.kind, ItemKind::Staff { .. }),
                            _ => false,
                        }
                }) else {
                    return GameResponse::error(
                        "invalid_command",
                        "There is no unidentified item of that kind.".into(),
                    )
                    .with_request_id(request_id);
                };
                record.identified = true;
                message = if record.cursed {
                    format!("You identify a cursed {}.", item)
                } else {
                    format!("You identify the {}.", item)
                };
                false
            }
            Command::Drop { item } => {
                let Some(record) = self.take_item(&item) else {
                    return GameResponse::error(
                        "invalid_command",
                        "You do not have that item.".into(),
                    )
                    .with_request_id(request_id);
                };
                self.items.push(Item {
                    pos: self.player,
                    kind: record.kind,
                    identified: record.identified,
                });
                message = "You drop an item.".into();
                true
            }
            Command::Equip => {
                if self.weapons == 0 {
                    return GameResponse::error(
                        "invalid_command",
                        "You have no weapon to equip.".into(),
                    )
                    .with_request_id(request_id);
                }
                self.weapons -= 1;
                let bonus = self
                    .inventory_items
                    .iter()
                    .find_map(|item| match item.kind {
                        ItemKind::Weapon { bonus } => Some(bonus),
                        _ => None,
                    })
                    .unwrap_or(1);
                self.weapon_cursed = self
                    .inventory_items
                    .iter()
                    .find(|item| matches!(item.kind, ItemKind::Weapon { .. }))
                    .is_some_and(|item| item.cursed);
                self.remove_inventory_record(|kind| matches!(kind, ItemKind::Weapon { .. }));
                self.weapon_bonus += bonus;
                message = "You equip the weapon.".into();
                true
            }
            Command::EquipArmor => {
                if self.armor == 0 {
                    return GameResponse::error(
                        "invalid_command",
                        "You have no armor to equip.".into(),
                    )
                    .with_request_id(request_id);
                }
                self.armor -= 1;
                let bonus = self
                    .inventory_items
                    .iter()
                    .find_map(|item| match item.kind {
                        ItemKind::Armor { bonus } => Some(bonus),
                        _ => None,
                    })
                    .unwrap_or(1);
                self.armor_cursed = self
                    .inventory_items
                    .iter()
                    .find(|item| matches!(item.kind, ItemKind::Armor { .. }))
                    .is_some_and(|item| item.cursed);
                self.remove_inventory_record(|kind| matches!(kind, ItemKind::Armor { .. }));
                self.armor_bonus += bonus;
                message = "You equip the armor.".into();
                true
            }
            Command::UnequipWeapon => {
                if self.weapon_bonus == 0 {
                    return GameResponse::error(
                        "invalid_command",
                        "You have no equipped weapon.".into(),
                    )
                    .with_request_id(request_id);
                }
                if self.weapon_cursed {
                    return GameResponse::error(
                        "invalid_command",
                        "The cursed weapon cannot be removed.".into(),
                    )
                    .with_request_id(request_id);
                }
                self.weapon_bonus -= 1;
                self.weapons += 1;
                self.inventory_items.push(InventoryItem {
                    kind: ItemKind::Weapon { bonus: 1 },
                    identified: true,
                    cursed: false,
                    quantity: 1,
                });
                message = "You remove the weapon.".into();
                true
            }
            Command::UnequipArmor => {
                if self.armor_bonus == 0 {
                    return GameResponse::error(
                        "invalid_command",
                        "You have no equipped armor.".into(),
                    )
                    .with_request_id(request_id);
                }
                if self.armor_cursed {
                    return GameResponse::error(
                        "invalid_command",
                        "The cursed armor cannot be removed.".into(),
                    )
                    .with_request_id(request_id);
                }
                self.armor_bonus -= 1;
                self.armor += 1;
                self.inventory_items.push(InventoryItem {
                    kind: ItemKind::Armor { bonus: 1 },
                    identified: true,
                    cursed: false,
                    quantity: 1,
                });
                message = "You remove the armor.".into();
                true
            }
            Command::Shoot { dx, dy }
                if (dx == 0) != (dy == 0) && dx.abs() <= 1 && dy.abs() <= 1 =>
            {
                if self.arrows == 0 {
                    return GameResponse::error("invalid_command", "You have no arrows.".into())
                        .with_request_id(request_id);
                }
                self.arrows -= 1;
                self.shoot(dx, dy, &mut message);
                true
            }
            Command::Search => {
                let mut found = false;
                for trap in &mut self.traps {
                    if (trap.pos.x - self.player.x).abs() <= 2
                        && (trap.pos.y - self.player.y).abs() <= 2
                        && !trap.triggered
                    {
                        trap.revealed = true;
                        found = true;
                    }
                }
                message = if found {
                    "You find a trap.".into()
                } else {
                    "You find nothing.".into()
                };
                false
            }
            Command::Quit => {
                self.end("quit");
                false
            }
            _ => {
                return GameResponse::error("invalid_command", "unsupported command".into())
                    .with_request_id(request_id);
            }
        };
        let mut transitioned = false;
        if consumed && !self.ended {
            self.advance_poison(&mut message);
            if !self.ended {
                self.trigger_trap(&mut message);
            }
            transitioned = self.transition_stairs(&mut message);
            self.turn += 1;
            self.advance_hunger(&mut message);
            if !self.ended {
                self.enemy_turn(&mut message);
            }
            if self.strength_turns > 0 {
                self.strength_turns -= 1;
            }
        }
        if transitioned && self.floor == 1 && self.amulets > 0 {
            self.end("victory");
            message = "You return to the surface with the amulet.".into();
        }
        self.reveal();
        GameResponse::snapshot_with(self, message).with_request_id(request_id)
    }

    fn move_or_attack(&mut self, dx: i32, dy: i32, message: &mut String) -> bool {
        let target = Pos {
            x: self.player.x + dx,
            y: self.player.y + dy,
        };
        if target.x <= 0 || target.x >= WIDTH - 1 || target.y <= 0 || target.y >= HEIGHT - 1 {
            *message = "You cannot move there.".into();
            return false;
        }
        if matches!(self.tiles[index(target)], Tile::Wall)
            || (dx != 0
                && dy != 0
                && (matches!(
                    self.tiles[index(Pos {
                        x: self.player.x + dx,
                        y: self.player.y
                    })],
                    Tile::Wall
                ) || matches!(
                    self.tiles[index(Pos {
                        x: self.player.x,
                        y: self.player.y + dy
                    })],
                    Tile::Wall
                )))
        {
            *message = "You cannot move there.".into();
            return false;
        }
        if let Some(i) = self.enemies.iter().position(|e| e.pos == target) {
            let damage = (4
                + self.rng.range(4) as i32
                + 5 / 2
                + self.weapon_bonus
                + self.staff_bonus
                + if self.strength_turns > 0 { 2 } else { 0 }
                - self.enemies[i].defense)
                .max(1);
            self.enemies[i].hp -= damage;
            *message = format!("You hit the enemy for {damage} damage.");
            if self.enemies[i].hp <= 0 {
                let xp = self.enemies[i].kind.stats().3;
                self.enemies.remove(i);
                self.defeated += 1;
                self.xp += xp;
                *message = "You defeated the enemy.".into();
                let threshold = self.level * 20;
                if self.level < 12 && self.xp >= threshold {
                    self.xp -= threshold;
                    self.level += 1;
                    self.max_hp += 2;
                    self.hp = self.max_hp;
                    *message = format!("You defeated the enemy and reached level {}.", self.level);
                }
            }
            return true;
        }
        self.player = target;
        true
    }

    fn enemy_turn(&mut self, message: &mut String) {
        let mut occupied: Vec<Pos> = self.enemies.iter().map(|enemy| enemy.pos).collect();
        for i in 0..self.enemies.len() {
            let enemy_pos = self.enemies[i].pos;
            if !self.can_see_player(enemy_pos) {
                continue;
            }
            if self.enemies[i].kind == EnemyKind::Troll && self.turn % 2 == 1 {
                continue;
            }
            if self.enemies[i].kind == EnemyKind::Troll {
                let max_hp = self.enemies[i].kind.stats().0;
                self.enemies[i].hp = (self.enemies[i].hp + 1).min(max_hp);
            }
            let retreating = self.enemies[i].kind == EnemyKind::Bat && self.enemies[i].hp <= 1;
            let dx = (self.player.x - enemy_pos.x).signum() * if retreating { -1 } else { 1 };
            let dy = (self.player.y - enemy_pos.y).signum() * if retreating { -1 } else { 1 };
            if (enemy_pos.x - self.player.x).abs() <= 1 && (enemy_pos.y - self.player.y).abs() <= 1
            {
                let enemy = &self.enemies[i];
                self.hp -= (enemy.attack - 1 - self.armor_bonus - self.ring_bonus).max(1);
                *message = "The enemy hits you.".into();
                if self.hp <= 0 {
                    self.end("death");
                    return;
                }
            } else if dx != 0 || dy != 0 {
                let next = Pos {
                    x: enemy_pos.x + dx,
                    y: enemy_pos.y + dy,
                };
                if !matches!(self.tiles[index(next)], Tile::Wall)
                    && next != self.player
                    && (self.enemies[i].kind == EnemyKind::Bat
                        || dx == 0
                        || dy == 0
                        || (!matches!(
                            self.tiles[index(Pos {
                                x: enemy_pos.x + dx,
                                y: enemy_pos.y
                            })],
                            Tile::Wall
                        ) && !matches!(
                            self.tiles[index(Pos {
                                x: enemy_pos.x,
                                y: enemy_pos.y + dy
                            })],
                            Tile::Wall
                        )))
                    && !occupied
                        .iter()
                        .enumerate()
                        .any(|(j, pos)| j != i && *pos == next)
                {
                    let enemy = &mut self.enemies[i];
                    enemy.pos = next;
                    occupied[i] = next;
                }
            }
        }
    }
    fn can_see_player(&self, from: Pos) -> bool {
        let dx = self.player.x - from.x;
        let dy = self.player.y - from.y;
        if dx.abs() > 8 || dy.abs() > 8 {
            return false;
        }
        if dx != 0 && dy != 0 && dx.abs() != dy.abs() {
            return false;
        }
        let step_x = dx.signum();
        let step_y = dy.signum();
        let mut pos = from;
        while pos != self.player {
            pos.x += step_x;
            pos.y += step_y;
            if pos != self.player && matches!(self.tiles[index(pos)], Tile::Wall) {
                return false;
            }
        }
        true
    }
    fn advance_hunger(&mut self, message: &mut String) {
        if self.hunger > 0 {
            self.hunger -= 1;
        } else {
            self.hp -= 1;
            *message = "Starvation hurts you.".into();
            if self.hp <= 0 {
                self.end("death");
            }
        }
    }
    fn trigger_trap(&mut self, message: &mut String) {
        if let Some(trap) = self
            .traps
            .iter_mut()
            .find(|trap| trap.pos == self.player && !trap.triggered)
        {
            trap.triggered = true;
            match trap.kind {
                TrapKind::Poison => {
                    self.poison_turns = 3;
                    self.hp -= 1;
                    *message = "A poison trap wounds you.".into();
                }
                TrapKind::Fire => {
                    self.hp -= 4;
                    *message = "A fire trap burns you.".into();
                }
            }
            if self.hp <= 0 {
                self.end("death");
            }
        }
    }
    fn advance_poison(&mut self, message: &mut String) {
        if self.poison_turns > 0 {
            self.poison_turns -= 1;
            self.hp -= 1;
            *message = "Poison harms you.".into();
            if self.hp <= 0 {
                self.end("death");
            }
        }
    }
    fn inventory_slots(&self) -> u32 {
        if self.inventory_items.is_empty() {
            self.food
                + self.weapons
                + self.armor
                + self.rings
                + self.staffs
                + self.potions
                + self.strength_potions
                + self.amulets
                + self.scrolls
                + self.teleport_scrolls
        } else {
            self.inventory_items.len() as u32
        }
    }
    fn take_item(&mut self, item: &str) -> Option<InventoryItem> {
        let kind = match item {
            "food" if self.food > 0 => {
                self.food -= 1;
                ItemKind::Food
            }
            "potion" if self.potions > 0 => {
                self.potions -= 1;
                ItemKind::Potion
            }
            "strength_potion" if self.strength_potions > 0 => {
                self.strength_potions -= 1;
                ItemKind::StrengthPotion
            }
            "scroll" if self.scrolls > 0 => {
                self.scrolls -= 1;
                ItemKind::MappingScroll
            }
            "teleport_scroll" if self.teleport_scrolls > 0 => {
                self.teleport_scrolls -= 1;
                ItemKind::TeleportScroll
            }
            "amulet" if self.amulets > 0 => {
                self.amulets -= 1;
                ItemKind::Amulet
            }
            "weapon" if self.weapons > 0 => {
                self.weapons -= 1;
                ItemKind::Weapon { bonus: 1 }
            }
            "armor" if self.armor > 0 => {
                self.armor -= 1;
                ItemKind::Armor { bonus: 1 }
            }
            "ring" if self.rings > 0 => {
                self.rings -= 1;
                ItemKind::Ring { bonus: 1 }
            }
            "staff" if self.staffs > 0 => {
                self.staffs -= 1;
                ItemKind::Staff { bonus: 3 }
            }
            _ => return None,
        };
        if let Some(i) = self.inventory_items.iter().position(|record| match item {
            "weapon" => matches!(record.kind, ItemKind::Weapon { .. }),
            "armor" => matches!(record.kind, ItemKind::Armor { .. }),
            "ring" => matches!(record.kind, ItemKind::Ring { .. }),
            "staff" => matches!(record.kind, ItemKind::Staff { .. }),
            "food" => matches!(record.kind, ItemKind::Food),
            "potion" => matches!(record.kind, ItemKind::Potion),
            "strength_potion" => matches!(record.kind, ItemKind::StrengthPotion),
            "scroll" => matches!(record.kind, ItemKind::MappingScroll),
            "teleport_scroll" => matches!(record.kind, ItemKind::TeleportScroll),
            "amulet" => matches!(record.kind, ItemKind::Amulet),
            _ => false,
        }) {
            let mut record = self.inventory_items[i].clone();
            if record.quantity > 1 {
                self.inventory_items[i].quantity -= 1;
                record.quantity = 1;
            } else {
                self.inventory_items.remove(i);
            }
            Some(record)
        } else {
            Some(InventoryItem {
                kind,
                identified: true,
                cursed: false,
                quantity: 1,
            })
        }
    }
    fn remove_inventory_record(&mut self, matches: impl Fn(&ItemKind) -> bool) {
        if let Some(i) = self
            .inventory_items
            .iter()
            .position(|item| matches(&item.kind))
        {
            if self.inventory_items[i].quantity > 1 {
                self.inventory_items[i].quantity -= 1;
            } else {
                self.inventory_items.remove(i);
            }
        }
    }
    fn random_open_floor(&mut self) -> Option<Pos> {
        for _ in 0..(WIDTH * HEIGHT) {
            let pos = Pos {
                x: 1 + self.rng.range((WIDTH - 2) as u64) as i32,
                y: 1 + self.rng.range((HEIGHT - 2) as u64) as i32,
            };
            if matches!(self.tiles[index(pos)], Tile::Floor)
                && pos != self.player
                && !self.enemies.iter().any(|enemy| enemy.pos == pos)
                && !self.items.iter().any(|item| item.pos == pos)
            {
                return Some(pos);
            }
        }
        None
    }
    fn shoot(&mut self, dx: i32, dy: i32, message: &mut String) {
        let mut pos = self.player;
        loop {
            pos.x += dx;
            pos.y += dy;
            if pos.x <= 0
                || pos.x >= WIDTH - 1
                || pos.y <= 0
                || pos.y >= HEIGHT - 1
                || matches!(self.tiles[index(pos)], Tile::Wall)
            {
                *message = "You shoot into the darkness.".into();
                return;
            }
            if let Some(i) = self.enemies.iter().position(|enemy| enemy.pos == pos) {
                let damage = (3
                    + self.weapon_bonus
                    + self.staff_bonus
                    + if self.strength_turns > 0 { 2 } else { 0 }
                    - self.enemies[i].defense)
                    .max(1);
                self.enemies[i].hp -= damage;
                *message = format!("You shoot the enemy for {damage} damage.");
                if self.enemies[i].hp <= 0 {
                    let xp = self.enemies[i].kind.stats().3;
                    self.enemies.remove(i);
                    self.defeated += 1;
                    self.xp += xp;
                }
                return;
            }
        }
    }
    fn transition_stairs(&mut self, message: &mut String) -> bool {
        if self.floor == 1 && self.player == self.stair {
            self.floor_states[0] = Some(self.capture_floor());
            let next = self.floor_states[1].take().unwrap_or_else(|| {
                let mut state = self.capture_floor();
                state.tiles[index(state.up_stair)] = Tile::Up;
                state.items.push(Item {
                    pos: state.stair,
                    kind: ItemKind::Amulet,
                    identified: true,
                });
                state
            });
            self.restore_floor(next);
            self.floor = 2;
            self.tiles[index(self.up_stair)] = Tile::Up;
            self.player = self.up_stair;
            *message = "You descend to floor 2.".into();
            true
        } else if self.floor == 2 && self.player == self.up_stair {
            self.floor_states[1] = Some(self.capture_floor());
            let previous = self.floor_states[0].take().expect("floor 1 state");
            self.restore_floor(previous);
            self.floor = 1;
            self.player = Pos {
                x: self.stair.x - 1,
                y: self.stair.y,
            };
            *message = "You ascend to floor 1.".into();
            true
        } else {
            false
        }
    }
    fn capture_floor(&self) -> FloorState {
        FloorState {
            tiles: self.tiles.clone(),
            stair: self.stair,
            up_stair: self.up_stair,
            enemies: self.enemies.clone(),
            items: self.items.clone(),
            traps: self.traps.clone(),
            known: self.known.clone(),
        }
    }
    fn restore_floor(&mut self, state: FloorState) {
        self.tiles = state.tiles;
        self.stair = state.stair;
        self.up_stair = state.up_stair;
        self.enemies = state.enemies;
        self.items = state.items;
        self.traps = state.traps;
        self.known = state.known;
    }
    fn reveal(&mut self) {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                if (x - self.player.x).abs() <= 6 && (y - self.player.y).abs() <= 6 {
                    self.known[index(Pos { x, y })] = true;
                }
            }
        }
    }
    fn end(&mut self, result: &str) {
        self.ended = true;
        self.result = Some(result.into());
        let bonus = if result == "victory" { 1_000 } else { 0 };
        self.score = Some(
            (bonus
                + self.floor as i32 * 100
                + self.defeated as i32 * 100
                + self.hp.max(0)
                + self.hunger
                - self.turn as i32)
                .max(0),
        );
    }
}
fn index(pos: Pos) -> usize {
    (pos.y * WIDTH + pos.x) as usize
}

#[derive(Deserialize)]
pub struct CommandRequest {
    pub version: u8,
    pub request_id: String,
    pub command: Command,
}
#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Command {
    #[serde(rename = "move")]
    Move { dx: i32, dy: i32 },
    #[serde(rename = "wait")]
    Wait,
    #[serde(rename = "pickup")]
    Pickup,
    #[serde(rename = "eat")]
    Eat,
    #[serde(rename = "drink")]
    Drink,
    #[serde(rename = "drink_strength")]
    DrinkStrength,
    #[serde(rename = "read_scroll")]
    ReadScroll,
    #[serde(rename = "teleport")]
    Teleport,
    #[serde(rename = "inventory")]
    Inventory,
    #[serde(rename = "drop")]
    Drop { item: String },
    #[serde(rename = "identify")]
    Identify { item: String },
    #[serde(rename = "equip")]
    Equip,
    #[serde(rename = "equip_armor")]
    EquipArmor,
    #[serde(rename = "equip_ring")]
    EquipRing,
    #[serde(rename = "unequip")]
    UnequipWeapon,
    #[serde(rename = "unequip_armor")]
    UnequipArmor,
    #[serde(rename = "unequip_ring")]
    UnequipRing,
    #[serde(rename = "equip_staff")]
    EquipStaff,
    #[serde(rename = "unequip_staff")]
    UnequipStaff,
    #[serde(rename = "shoot")]
    Shoot { dx: i32, dy: i32 },
    #[serde(rename = "search")]
    Search,
    #[serde(rename = "quit")]
    Quit,
}
#[derive(Deserialize, Serialize)]
pub struct GameResponse {
    #[serde(rename = "type")]
    pub type_: String,
    pub version: u8,
    pub request_id: String,
    pub turn: u64,
    pub ended: bool,
    pub result: Option<String>,
    pub score: Option<i32>,
    pub hp: i32,
    pub max_hp: i32,
    pub hunger: i32,
    pub food: u32,
    pub weapons: u32,
    pub weapon_bonus: i32,
    pub strength_turns: u32,
    pub armor: u32,
    pub armor_bonus: i32,
    pub ring_bonus: i32,
    pub rings: u32,
    pub staff_bonus: i32,
    pub staffs: u32,
    pub potions: u32,
    pub strength_potions: u32,
    pub amulets: u32,
    pub scrolls: u32,
    pub teleport_scrolls: u32,
    pub arrows: u32,
    pub poison_turns: u32,
    pub level: u32,
    pub xp: u32,
    pub defeated: u32,
    pub seed: u64,
    pub floor: u8,
    pub map: Vec<String>,
    pub message: String,
    pub error: Option<ErrorBody>,
}
#[derive(Deserialize, Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
}

impl GameResponse {
    pub fn snapshot(game: &Game) -> Self {
        Self::snapshot_with(game, "Welcome to the dungeon.".into())
    }
    fn snapshot_with(game: &Game, message: String) -> Self {
        let mut map = vec![vec![' '; WIDTH as usize]; HEIGHT as usize];
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let pos = Pos { x, y };
                let visible =
                    (pos.x - game.player.x).abs() <= 6 && (pos.y - game.player.y).abs() <= 6;
                if game.known[index(pos)] {
                    map[y as usize][x as usize] = match game.tiles[index(pos)] {
                        Tile::Wall => '#',
                        Tile::Floor => '.',
                        Tile::Up => '<',
                        Tile::Down => '>',
                    };
                    if !visible {
                        map[y as usize][x as usize] =
                            map[y as usize][x as usize].to_ascii_lowercase();
                    }
                }
            }
        }
        for item in &game.items {
            if (item.pos.x - game.player.x).abs() <= 6 && (item.pos.y - game.player.y).abs() <= 6 {
                map[item.pos.y as usize][item.pos.x as usize] = match item.kind {
                    ItemKind::Food => '!',
                    ItemKind::Weapon { .. } => ')',
                    ItemKind::Armor { .. } => '[',
                    ItemKind::Ring { .. } => '=',
                    ItemKind::Staff { .. } => '/',
                    ItemKind::Potion => '?',
                    ItemKind::StrengthPotion => '?',
                    ItemKind::Amulet => ',',
                    ItemKind::MappingScroll => '~',
                    ItemKind::TeleportScroll => '~',
                };
            }
        }
        for e in &game.enemies {
            if (e.pos.x - game.player.x).abs() <= 6 && (e.pos.y - game.player.y).abs() <= 6 {
                map[e.pos.y as usize][e.pos.x as usize] = e.kind.stats().4;
            }
        }
        for trap in &game.traps {
            if trap.revealed && !trap.triggered && game.known[index(trap.pos)] {
                map[trap.pos.y as usize][trap.pos.x as usize] = '^';
            }
        }
        map[game.player.y as usize][game.player.x as usize] = '@';
        Self {
            type_: "snapshot".into(),
            version: 1,
            request_id: "server".into(),
            turn: game.turn,
            ended: game.ended,
            result: game.result.clone(),
            score: game.score,
            floor: game.floor,
            hp: game.hp,
            max_hp: game.max_hp,
            hunger: game.hunger,
            food: game.food,
            weapons: game.weapons,
            weapon_bonus: game.weapon_bonus,
            strength_turns: game.strength_turns,
            armor: game.armor,
            armor_bonus: game.armor_bonus,
            ring_bonus: game.ring_bonus,
            rings: game.rings,
            staff_bonus: game.staff_bonus,
            staffs: game.staffs,
            potions: game.potions,
            strength_potions: game.strength_potions,
            amulets: game.amulets,
            scrolls: game.scrolls,
            teleport_scrolls: game.teleport_scrolls,
            arrows: game.arrows,
            poison_turns: game.poison_turns,
            level: game.level,
            xp: game.xp,
            defeated: game.defeated,
            seed: game.seed,
            map: map
                .into_iter()
                .map(|row| row.into_iter().collect())
                .collect(),
            message,
            error: None,
        }
    }
    pub fn error(code: &str, message: String) -> Self {
        Self {
            type_: "error".into(),
            version: 1,
            request_id: "server".into(),
            turn: 0,
            ended: false,
            result: None,
            score: None,
            floor: 0,
            hp: 0,
            max_hp: 0,
            hunger: 0,
            food: 0,
            weapons: 0,
            weapon_bonus: 0,
            strength_turns: 0,
            armor: 0,
            armor_bonus: 0,
            ring_bonus: 0,
            rings: 0,
            staff_bonus: 0,
            staffs: 0,
            potions: 0,
            strength_potions: 0,
            amulets: 0,
            scrolls: 0,
            teleport_scrolls: 0,
            arrows: 0,
            poison_turns: 0,
            level: 0,
            xp: 0,
            defeated: 0,
            seed: 0,
            map: Vec::new(),
            message: String::new(),
            error: Some(ErrorBody {
                code: code.into(),
                message,
            }),
        }
    }

    fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = request_id;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn request(command: Command) -> CommandRequest {
        CommandRequest {
            version: 1,
            request_id: "1".into(),
            command,
        }
    }
    #[test]
    fn same_seed_reproduces_map_and_enemies() {
        let a = Game::new(Some(42));
        let b = Game::new(Some(42));
        assert_eq!(a.tiles, b.tiles);
        assert_eq!(
            a.enemies.iter().map(|e| e.pos).collect::<Vec<_>>(),
            b.enemies.iter().map(|e| e.pos).collect::<Vec<_>>()
        );
    }
    #[test]
    fn wait_consumes_turn_and_heals() {
        let mut game = Game::new(Some(1));
        game.hp = 10;
        let response = game.handle(request(Command::Wait));
        assert_eq!(response.turn, 1);
        assert_eq!(game.hp, 11);
    }
    #[test]
    fn wall_move_does_not_consume_turn() {
        let mut game = Game::new(Some(1));
        game.player.x = 1;
        let response = game.handle(request(Command::Move { dx: -1, dy: 0 }));
        assert_eq!(response.turn, 0);
        assert_eq!(response.type_, "error");
    }
    #[test]
    fn descending_and_ascending_changes_floor() {
        let mut game = Game::new(Some(1));
        game.player = Pos {
            x: game.stair.x - 1,
            y: game.stair.y,
        };
        game.enemies.clear();
        let response = game.handle(request(Command::Move { dx: 1, dy: 0 }));
        assert_eq!(game.floor, 2);
        assert_eq!(response.floor, 2);
        game.player = Pos {
            x: game.up_stair.x + 1,
            y: game.up_stair.y,
        };
        let response = game.handle(request(Command::Move { dx: -1, dy: 0 }));
        assert_eq!(game.floor, 1);
        assert_eq!(response.floor, 1);
    }

    #[test]
    fn floor_world_state_survives_a_round_trip() {
        let mut game = Game::new(Some(1));
        game.enemies.clear();
        game.player = Pos { x: 3, y: 2 };
        game.handle(request(Command::Pickup));
        game.player = game.stair;
        game.handle(request(Command::Wait));
        assert_eq!(game.floor, 2);
        game.items.clear();
        game.player = game.up_stair;
        game.handle(request(Command::Wait));
        assert_eq!(game.floor, 1);
        assert_eq!(game.food, 1);
        game.player = game.stair;
        game.handle(request(Command::Wait));
        assert_eq!(game.floor, 2);
        assert!(game.items.is_empty());
    }

    #[test]
    fn enemies_do_not_act_without_line_of_sight() {
        let mut game = Game::new(Some(1));
        game.enemies = vec![Enemy {
            id: 1,
            pos: Pos { x: 14, y: 5 },
            kind: EnemyKind::Goblin,
            hp: 5,
            attack: 2,
            defense: 0,
        }];
        game.handle(request(Command::Wait));
        assert_eq!(game.enemies[0].pos, Pos { x: 14, y: 5 });
    }

    #[test]
    fn trolls_only_act_every_other_turn() {
        let mut game = Game::new(Some(1));
        game.enemies = vec![Enemy {
            id: 1,
            pos: Pos { x: 8, y: 2 },
            kind: EnemyKind::Troll,
            hp: 12,
            attack: 3,
            defense: 1,
        }];
        game.handle(request(Command::Wait));
        assert_eq!(game.enemies[0].pos, Pos { x: 8, y: 2 });
        game.handle(request(Command::Wait));
        assert_eq!(game.enemies[0].pos, Pos { x: 7, y: 2 });
    }

    #[test]
    fn trolls_regenerate_one_hp_when_they_act() {
        let mut game = Game::new(Some(1));
        game.enemies = vec![Enemy {
            id: 1,
            pos: Pos { x: 8, y: 2 },
            kind: EnemyKind::Troll,
            hp: 5,
            attack: 3,
            defense: 1,
        }];
        game.handle(request(Command::Wait));
        assert_eq!(game.enemies[0].hp, 5);
        game.handle(request(Command::Wait));
        assert_eq!(game.enemies[0].hp, 6);
    }

    #[test]
    fn wounded_bats_retreat_from_the_player() {
        let mut game = Game::new(Some(1));
        game.enemies = vec![Enemy {
            id: 1,
            pos: Pos { x: 8, y: 2 },
            kind: EnemyKind::Bat,
            hp: 1,
            attack: 1,
            defense: 0,
        }];
        game.handle(request(Command::Wait));
        assert_eq!(game.enemies[0].pos, Pos { x: 9, y: 2 });
    }

    #[test]
    fn mapping_scroll_reveals_the_dungeon() {
        let mut game = Game::new(Some(1));
        game.enemies.clear();
        game.player = Pos { x: 8, y: 2 };
        game.handle(request(Command::Pickup));
        assert_eq!(game.scrolls, 1);
        let response = game.handle(request(Command::ReadScroll));
        assert_eq!(game.scrolls, 0);
        assert!(game.inventory_items.is_empty());
        assert!(game.known.iter().all(|known| *known));
        assert_eq!(response.turn, 2);
    }

    #[test]
    fn inventory_inspection_does_not_consume_a_turn() {
        let mut game = Game::new(Some(1));
        game.food = 2;
        game.potions = 1;
        let response = game.handle(request(Command::Inventory));
        assert_eq!(response.turn, 0);
        assert!(response.message.contains("food 2"));
        assert!(response.message.contains("potions 1"));
    }

    #[test]
    fn dropping_an_item_returns_it_to_the_floor() {
        let mut game = Game::new(Some(1));
        game.food = 1;
        let response = game.handle(request(Command::Drop {
            item: "food".into(),
        }));
        assert_eq!(game.food, 0);
        assert!(
            game.items
                .iter()
                .any(|item| item.pos == game.player && item.kind == ItemKind::Food)
        );
        assert_eq!(response.turn, 1);
    }

    #[test]
    fn picked_up_items_are_kept_as_individual_records() {
        let mut game = Game::new(Some(1));
        game.player = Pos { x: 3, y: 2 };
        game.handle(request(Command::Pickup));
        assert_eq!(game.inventory_items.len(), 1);
        assert!(game.inventory_items[0].identified);
        assert!(!game.inventory_items[0].cursed);
    }

    #[test]
    fn cursed_weapon_is_unidentified_until_inspected() {
        let mut game = Game::new(Some(1));
        game.player = Pos { x: 9, y: 2 };
        game.handle(request(Command::Pickup));
        assert!(!game.inventory_items[0].identified);
        assert!(game.inventory_items[0].cursed);
        let inventory = game.handle(request(Command::Inventory));
        assert!(
            inventory
                .message
                .contains("weapon x1 (identified=false, cursed=unknown)")
        );
        assert!(!inventory.message.contains("bonus: -1"));
        let response = game.handle(request(Command::Identify {
            item: "weapon".into(),
        }));
        assert!(response.message.contains("cursed"));
        assert!(game.inventory_items[0].identified);
    }

    #[test]
    fn cursed_weapon_cannot_be_removed_after_equipping() {
        let mut game = Game::new(Some(1));
        game.player = Pos { x: 9, y: 2 };
        game.handle(request(Command::Pickup));
        game.handle(request(Command::Equip));
        let response = game.handle(request(Command::UnequipWeapon));
        assert_eq!(response.type_, "error");
        assert_eq!(game.weapon_bonus, -1);
        assert_eq!(game.turn, 2);
    }

    #[test]
    fn cursed_armor_cannot_be_removed_after_equipping() {
        let mut game = Game::new(Some(1));
        game.player = Pos { x: 10, y: 2 };
        game.handle(request(Command::Pickup));
        game.handle(request(Command::EquipArmor));
        let response = game.handle(request(Command::UnequipArmor));
        assert_eq!(response.type_, "error");
        assert_eq!(game.armor_bonus, -1);
        assert_eq!(game.turn, 2);
    }

    #[test]
    fn cursed_armor_is_identified_without_consuming_a_turn() {
        let mut game = Game::new(Some(1));
        game.player = Pos { x: 10, y: 2 };
        game.handle(request(Command::Pickup));
        let response = game.handle(request(Command::Identify {
            item: "armor".into(),
        }));
        assert!(response.message.contains("cursed"));
        assert_eq!(game.turn, 1);
        assert!(game.inventory_items[0].identified);
    }

    #[test]
    fn dropping_a_cursed_weapon_preserves_its_curse() {
        let mut game = Game::new(Some(1));
        game.player = Pos { x: 9, y: 2 };
        game.handle(request(Command::Pickup));
        game.handle(request(Command::Identify {
            item: "weapon".into(),
        }));
        game.handle(request(Command::Drop {
            item: "weapon".into(),
        }));
        assert!(game.items.iter().any(|item| {
            item.pos == game.player
                && item.kind == ItemKind::Weapon { bonus: -1 }
                && item.identified
        }));
    }

    #[test]
    fn dropping_a_cursed_armor_preserves_its_curse() {
        let mut game = Game::new(Some(1));
        game.player = Pos { x: 10, y: 2 };
        game.handle(request(Command::Pickup));
        game.handle(request(Command::Identify {
            item: "armor".into(),
        }));
        game.handle(request(Command::Drop {
            item: "armor".into(),
        }));
        assert!(game.items.iter().any(|item| {
            item.pos == game.player && item.kind == ItemKind::Armor { bonus: -1 } && item.identified
        }));
    }

    #[test]
    fn dropping_an_item_you_do_not_have_is_an_error() {
        let mut game = Game::new(Some(1));
        let response = game.handle(request(Command::Drop {
            item: "potion".into(),
        }));
        assert_eq!(response.type_, "error");
        assert_eq!(response.turn, 0);
    }

    #[test]
    fn identifying_without_an_item_is_an_error_without_consuming_a_turn() {
        let mut game = Game::new(Some(1));
        let response = game.handle(request(Command::Identify {
            item: "weapon".into(),
        }));
        assert_eq!(response.type_, "error");
        assert_eq!(response.turn, 0);
    }

    #[test]
    fn every_item_kind_counts_toward_inventory_limit() {
        let mut game = Game::new(Some(1));
        game.food = 17;
        game.potions = 1;
        game.scrolls = 1;
        game.amulets = 1;
        game.player = Pos { x: 4, y: 2 };
        let response = game.handle(request(Command::Pickup));
        assert_eq!(response.type_, "error");
        assert_eq!(response.error.as_ref().unwrap().code, "inventory_full");
        assert_eq!(response.turn, 0);
    }

    #[test]
    fn ground_enemies_cannot_cut_diagonal_corners() {
        let mut game = Game::new(Some(1));
        game.tiles[index(Pos { x: 3, y: 4 })] = Tile::Wall;
        game.tiles[index(Pos { x: 4, y: 3 })] = Tile::Wall;
        game.enemies = vec![Enemy {
            id: 1,
            pos: Pos { x: 4, y: 4 },
            kind: EnemyKind::Goblin,
            hp: 5,
            attack: 2,
            defense: 0,
        }];
        game.handle(request(Command::Wait));
        assert_eq!(game.enemies[0].pos, Pos { x: 4, y: 4 });
    }

    #[test]
    fn bats_can_fly_through_diagonal_corners() {
        let mut game = Game::new(Some(1));
        game.tiles[index(Pos { x: 3, y: 4 })] = Tile::Wall;
        game.tiles[index(Pos { x: 4, y: 3 })] = Tile::Wall;
        game.enemies = vec![Enemy {
            id: 1,
            pos: Pos { x: 4, y: 4 },
            kind: EnemyKind::Bat,
            hp: 3,
            attack: 1,
            defense: 0,
        }];
        game.handle(request(Command::Wait));
        assert_eq!(game.enemies[0].pos, Pos { x: 3, y: 3 });
    }

    #[test]
    fn returning_with_the_amulet_ends_in_victory() {
        let mut game = Game::new(Some(1));
        game.enemies.clear();
        game.player = game.stair;
        game.handle(request(Command::Wait));
        assert!(game.items.iter().any(|item| item.kind == ItemKind::Amulet));
        game.player = game.stair;
        game.handle(request(Command::Pickup));
        assert_eq!(game.amulets, 1);
        game.player = game.up_stair;
        let response = game.handle(request(Command::Wait));
        assert!(response.ended);
        assert_eq!(response.result.as_deref(), Some("victory"));
    }

    #[test]
    fn finished_game_reports_score() {
        let mut game = Game::new(Some(1));
        let response = game.handle(request(Command::Quit));
        assert_eq!(response.result.as_deref(), Some("quit"));
        assert_eq!(response.score, Some(220));
    }

    #[test]
    fn picking_up_food_adds_it_to_inventory() {
        let mut game = Game::new(Some(1));
        game.player = Pos { x: 3, y: 2 };
        game.hunger = 60;
        let response = game.handle(request(Command::Pickup));
        assert_eq!(response.turn, 1);
        assert_eq!(game.hunger, 59);
        assert_eq!(game.items.len(), 12);
        assert_eq!(game.food, 1);
    }

    #[test]
    fn eating_food_restores_hunger_and_consumes_turn() {
        let mut game = Game::new(Some(1));
        game.food = 1;
        game.hunger = 60;
        let response = game.handle(request(Command::Eat));
        assert_eq!(response.turn, 1);
        assert_eq!(game.hunger, 84);
        assert_eq!(game.food, 0);
    }

    #[test]
    fn picking_up_without_food_is_an_error() {
        let mut game = Game::new(Some(1));
        let response = game.handle(request(Command::Pickup));
        assert_eq!(response.type_, "error");
        assert_eq!(response.turn, 0);
    }

    #[test]
    fn eating_without_food_is_an_error() {
        let mut game = Game::new(Some(1));
        let response = game.handle(request(Command::Eat));
        assert_eq!(response.type_, "error");
        assert_eq!(response.turn, 0);
    }

    #[test]
    fn picking_up_and_equipping_a_weapon_updates_equipment() {
        let mut game = Game::new(Some(1));
        game.player = Pos { x: 4, y: 2 };
        game.handle(request(Command::Pickup));
        assert_eq!(game.weapons, 1);
        let response = game.handle(request(Command::Equip));
        assert_eq!(game.weapons, 0);
        assert_eq!(game.weapon_bonus, 1);
        assert!(response.message.contains("equip"));
        game.handle(request(Command::UnequipWeapon));
        assert_eq!(game.weapons, 1);
        assert_eq!(game.weapon_bonus, 0);
    }

    #[test]
    fn equipping_without_a_weapon_is_an_error() {
        let mut game = Game::new(Some(1));
        let response = game.handle(request(Command::Equip));
        assert_eq!(response.type_, "error");
        assert_eq!(response.turn, 0);
    }

    #[test]
    fn picking_up_and_equipping_armor_reduces_damage() {
        let mut game = Game::new(Some(1));
        game.player = Pos { x: 5, y: 2 };
        game.handle(request(Command::Pickup));
        let response = game.handle(request(Command::EquipArmor));
        assert_eq!(game.armor, 0);
        assert_eq!(game.armor_bonus, 1);
        assert!(response.message.contains("armor"));
        game.enemies = vec![Enemy {
            id: 1,
            pos: Pos { x: 6, y: 2 },
            kind: EnemyKind::Goblin,
            hp: 5,
            attack: 2,
            defense: 0,
        }];
        game.handle(request(Command::Wait));
        assert_eq!(game.hp, 19);
    }

    #[test]
    fn inventory_rejects_items_after_twenty_slots() {
        let mut game = Game::new(Some(1));
        game.food = 20;
        game.player = Pos { x: 3, y: 2 };
        let response = game.handle(request(Command::Pickup));
        assert_eq!(response.type_, "error");
        assert_eq!(response.turn, 0);
    }

    #[test]
    fn multiple_items_are_picked_up_one_at_a_time() {
        let mut game = Game::new(Some(1));
        game.player = Pos { x: 3, y: 2 };
        game.items.push(Item {
            pos: game.player,
            kind: ItemKind::Food,
            identified: true,
        });
        game.handle(request(Command::Pickup));
        assert_eq!(game.food, 1);
        assert_eq!(game.items.len(), 13);
    }

    #[test]
    fn identical_food_stacks_in_one_inventory_slot() {
        let mut game = Game::new(Some(1));
        game.player = Pos { x: 3, y: 2 };
        game.handle(request(Command::Pickup));
        game.items.push(Item {
            pos: game.player,
            kind: ItemKind::Food,
            identified: true,
        });
        game.handle(request(Command::Pickup));
        assert_eq!(game.food, 2);
        assert_eq!(game.inventory_items.len(), 1);
        assert_eq!(game.inventory_items[0].quantity, 2);
        assert_eq!(game.inventory_slots(), 1);
        game.handle(request(Command::Drop {
            item: "food".into(),
        }));
        assert_eq!(game.food, 1);
        assert_eq!(game.inventory_items[0].quantity, 1);
    }

    #[test]
    fn identical_potions_stack_and_are_consumed_one_at_a_time() {
        let mut game = Game::new(Some(1));
        game.player = Pos { x: 3, y: 2 };
        game.items.clear();
        game.items.push(Item {
            pos: game.player,
            kind: ItemKind::Potion,
            identified: true,
        });
        game.handle(request(Command::Pickup));
        game.items.push(Item {
            pos: game.player,
            kind: ItemKind::Potion,
            identified: true,
        });
        game.handle(request(Command::Pickup));
        assert_eq!(game.inventory_items.len(), 1);
        assert_eq!(game.inventory_items[0].quantity, 2);
        game.handle(request(Command::Drink));
        assert_eq!(game.potions, 1);
        assert_eq!(game.inventory_items[0].quantity, 1);
    }

    #[test]
    fn strength_potion_grants_temporary_attack_power() {
        let mut game = Game::new(Some(1));
        game.enemies.clear();
        game.player = Pos { x: 11, y: 2 };
        game.handle(request(Command::Pickup));
        assert_eq!(game.strength_potions, 1);
        game.handle(request(Command::DrinkStrength));
        assert_eq!(game.strength_potions, 0);
        assert_eq!(game.strength_turns, 9);
        assert!(game.inventory_items.is_empty());
    }

    #[test]
    fn protection_ring_updates_defense_and_can_be_removed() {
        let mut game = Game::new(Some(1));
        game.enemies.clear();
        game.player = Pos { x: 13, y: 2 };
        game.handle(request(Command::Pickup));
        game.handle(request(Command::EquipRing));
        assert_eq!(game.ring_bonus, 1);
        assert_eq!(game.rings, 0);
        game.handle(request(Command::UnequipRing));
        assert_eq!(game.ring_bonus, 0);
        assert_eq!(game.rings, 1);
    }

    #[test]
    fn striking_staff_updates_attack_and_can_be_removed() {
        let mut game = Game::new(Some(1));
        game.enemies.clear();
        game.player = Pos { x: 14, y: 2 };
        game.handle(request(Command::Pickup));
        game.handle(request(Command::EquipStaff));
        assert_eq!(game.staff_bonus, 3);
        assert_eq!(game.staffs, 0);
        game.handle(request(Command::UnequipStaff));
        assert_eq!(game.staff_bonus, 0);
        assert_eq!(game.staffs, 1);
    }

    #[test]
    fn cursed_ring_and_staff_are_identifiable_and_irremovable() {
        let mut game = Game::new(Some(1));
        game.enemies.clear();
        game.player = Pos { x: 15, y: 2 };
        game.handle(request(Command::Pickup));
        game.handle(request(Command::Identify {
            item: "ring".into(),
        }));
        game.handle(request(Command::EquipRing));
        let response = game.handle(request(Command::UnequipRing));
        assert_eq!(response.type_, "error");
        assert_eq!(game.ring_bonus, -1);

        game.player = Pos { x: 16, y: 2 };
        game.handle(request(Command::Pickup));
        game.handle(request(Command::Identify {
            item: "staff".into(),
        }));
        game.handle(request(Command::EquipStaff));
        let response = game.handle(request(Command::UnequipStaff));
        assert_eq!(response.type_, "error");
        assert_eq!(game.staff_bonus, -2);
    }

    #[test]
    fn teleport_scroll_moves_to_an_open_floor() {
        let mut game = Game::new(Some(1));
        game.items.clear();
        game.enemies.clear();
        game.player = Pos { x: 3, y: 2 };
        game.teleport_scrolls = 1;
        game.inventory_items.push(InventoryItem {
            kind: ItemKind::TeleportScroll,
            identified: true,
            cursed: false,
            quantity: 1,
        });
        let old_pos = game.player;
        let response = game.handle(request(Command::Teleport));
        assert_eq!(response.type_, "snapshot");
        assert_ne!(game.player, old_pos);
        assert!(matches!(game.tiles[index(game.player)], Tile::Floor));
        assert_eq!(game.teleport_scrolls, 0);
        assert_eq!(game.turn, 1);
    }

    #[test]
    fn identical_scrolls_stack_and_are_consumed_one_at_a_time() {
        let mut game = Game::new(Some(1));
        game.player = Pos { x: 3, y: 2 };
        game.items.clear();
        for _ in 0..2 {
            game.items.push(Item {
                pos: game.player,
                kind: ItemKind::MappingScroll,
                identified: true,
            });
            game.handle(request(Command::Pickup));
        }
        assert_eq!(game.inventory_items.len(), 1);
        assert_eq!(game.inventory_items[0].quantity, 2);
        game.handle(request(Command::ReadScroll));
        assert_eq!(game.scrolls, 1);
        assert_eq!(game.inventory_items[0].quantity, 1);
    }

    #[test]
    fn dropping_from_a_stack_returns_only_one_item() {
        let mut game = Game::new(Some(1));
        game.items.clear();
        game.player = Pos { x: 3, y: 2 };
        for _ in 0..2 {
            game.items.push(Item {
                pos: game.player,
                kind: ItemKind::Potion,
                identified: true,
            });
            game.handle(request(Command::Pickup));
        }
        game.handle(request(Command::Drop {
            item: "potion".into(),
        }));
        assert_eq!(game.potions, 1);
        assert_eq!(game.inventory_items[0].quantity, 1);
        assert_eq!(
            game.items
                .iter()
                .filter(|item| item.pos == game.player && item.kind == ItemKind::Potion)
                .count(),
            1
        );
    }

    #[test]
    fn a_large_consumable_stack_uses_one_inventory_slot() {
        let mut game = Game::new(Some(1));
        game.items.clear();
        game.enemies.clear();
        game.player = Pos { x: 3, y: 2 };
        for _ in 0..21 {
            game.items.push(Item {
                pos: game.player,
                kind: ItemKind::Food,
                identified: true,
            });
            game.handle(request(Command::Pickup));
        }
        assert_eq!(game.food, 21);
        assert_eq!(game.inventory_slots(), 1);
        assert_eq!(game.inventory_items[0].quantity, 21);
    }

    #[test]
    fn equipment_with_distinct_metadata_does_not_stack() {
        let mut game = Game::new(Some(1));
        game.items.clear();
        game.enemies.clear();
        game.player = Pos { x: 3, y: 2 };
        game.items.extend([
            Item {
                pos: game.player,
                kind: ItemKind::Weapon { bonus: 1 },
                identified: true,
            },
            Item {
                pos: game.player,
                kind: ItemKind::Weapon { bonus: -1 },
                identified: false,
            },
        ]);
        game.handle(request(Command::Pickup));
        game.handle(request(Command::Pickup));
        assert_eq!(game.inventory_items.len(), 2);
        assert_eq!(game.inventory_slots(), 2);
    }

    #[test]
    fn identical_equipment_and_amulets_stack() {
        let mut game = Game::new(Some(1));
        game.items.clear();
        game.enemies.clear();
        game.player = Pos { x: 3, y: 2 };
        for kind in [ItemKind::Weapon { bonus: 1 }, ItemKind::Amulet] {
            for _ in 0..2 {
                game.items.push(Item {
                    pos: game.player,
                    kind,
                    identified: true,
                });
                game.handle(request(Command::Pickup));
            }
        }
        assert_eq!(game.inventory_items.len(), 2);
        assert!(game.inventory_items.iter().all(|item| item.quantity == 2));
        assert_eq!(game.inventory_slots(), 2);
    }

    #[test]
    fn twenty_distinct_equipment_items_fill_the_inventory() {
        let mut game = Game::new(Some(1));
        game.items.clear();
        game.enemies.clear();
        game.player = Pos { x: 3, y: 2 };
        for bonus in 0..20 {
            game.inventory_items.push(InventoryItem {
                kind: ItemKind::Weapon { bonus },
                identified: true,
                cursed: false,
                quantity: 1,
            });
        }
        game.weapons = 20;
        game.items.push(Item {
            pos: game.player,
            kind: ItemKind::Weapon { bonus: 1 },
            identified: true,
        });
        let response = game.handle(request(Command::Pickup));
        assert_eq!(response.type_, "error");
        assert_eq!(response.turn, 0);
        assert_eq!(game.inventory_items.len(), 20);
    }

    #[test]
    fn full_inventory_keeps_the_item_on_the_floor() {
        let mut game = Game::new(Some(1));
        game.items.clear();
        game.enemies.clear();
        game.player = Pos { x: 3, y: 2 };
        for bonus in 0..20 {
            game.inventory_items.push(InventoryItem {
                kind: ItemKind::Weapon { bonus },
                identified: true,
                cursed: false,
                quantity: 1,
            });
        }
        game.weapons = 20;
        game.items.push(Item {
            pos: game.player,
            kind: ItemKind::Food,
            identified: true,
        });
        let response = game.handle(request(Command::Pickup));
        assert_eq!(response.type_, "error");
        assert_eq!(game.items.len(), 1);
        assert_eq!(game.items[0].kind, ItemKind::Food);
    }

    #[test]
    fn dropped_consumable_merges_back_into_existing_stack() {
        let mut game = Game::new(Some(1));
        game.items.clear();
        game.enemies.clear();
        game.player = Pos { x: 3, y: 2 };
        for _ in 0..2 {
            game.items.push(Item {
                pos: game.player,
                kind: ItemKind::Food,
                identified: true,
            });
            game.handle(request(Command::Pickup));
        }
        game.handle(request(Command::Drop {
            item: "food".into(),
        }));
        game.handle(request(Command::Pickup));
        assert_eq!(game.food, 2);
        assert_eq!(game.inventory_items.len(), 1);
        assert_eq!(game.inventory_items[0].quantity, 2);
    }

    #[test]
    fn every_consumed_turn_reduces_hunger() {
        let mut game = Game::new(Some(1));
        game.hunger = 1;
        game.handle(request(Command::Wait));
        assert_eq!(game.hunger, 0);
        assert_eq!(game.hp, 20);
    }

    #[test]
    fn starvation_damages_player() {
        let mut game = Game::new(Some(1));
        game.hunger = 0;
        game.handle(request(Command::Wait));
        assert_eq!(game.hp, 19);
    }

    #[test]
    fn defeating_an_enemy_awards_experience_and_levels_up() {
        let mut game = Game::new(Some(1));
        game.enemies = vec![Enemy {
            id: 99,
            pos: Pos { x: 3, y: 2 },
            kind: EnemyKind::Goblin,
            hp: 1,
            attack: 0,
            defense: 0,
        }];
        game.xp = 10;
        let response = game.handle(request(Command::Move { dx: 1, dy: 0 }));
        assert_eq!(game.xp, 0);
        assert_eq!(game.level, 2);
        assert_eq!(game.max_hp, 22);
        assert_eq!(game.defeated, 1);
        assert!(response.message.contains("level"));
    }

    #[test]
    fn generated_enemies_include_distinct_kinds() {
        let game = Game::new(Some(42));
        let kinds: std::collections::HashSet<_> =
            game.enemies.iter().map(|enemy| enemy.kind).collect();
        assert!(kinds.len() >= 2);
    }

    #[test]
    fn shooting_hits_the_first_enemy_in_a_straight_line() {
        let mut game = Game::new(Some(1));
        game.enemies = vec![Enemy {
            id: 7,
            pos: Pos { x: 5, y: 2 },
            kind: EnemyKind::Goblin,
            hp: 5,
            attack: 0,
            defense: 0,
        }];
        let response = game.handle(request(Command::Shoot { dx: 1, dy: 0 }));
        assert_eq!(game.arrows, 2);
        assert_eq!(game.enemies[0].hp, 2);
        assert!(response.message.contains("shoot"));
    }

    #[test]
    fn shooting_without_arrows_is_an_error() {
        let mut game = Game::new(Some(1));
        game.arrows = 0;
        let response = game.handle(request(Command::Shoot { dx: 1, dy: 0 }));
        assert_eq!(response.type_, "error");
        assert_eq!(response.turn, 0);
    }

    #[test]
    fn shooting_down_an_enemy_awards_experience() {
        let mut game = Game::new(Some(1));
        game.enemies = vec![Enemy {
            id: 7,
            pos: Pos { x: 5, y: 2 },
            kind: EnemyKind::Goblin,
            hp: 3,
            attack: 0,
            defense: 0,
        }];
        game.handle(request(Command::Shoot { dx: 1, dy: 0 }));
        assert_eq!(game.defeated, 1);
        assert_eq!(game.xp, 10);
    }

    #[test]
    fn stepping_on_a_trap_applies_poison() {
        let mut game = Game::new(Some(1));
        game.enemies.clear();
        game.player = Pos { x: 5, y: 2 };
        game.handle(request(Command::Move { dx: 1, dy: 0 }));
        assert_eq!(game.poison_turns, 3);
        assert_eq!(game.hp, 19);
    }

    #[test]
    fn poison_damages_on_later_turns_and_expires() {
        let mut game = Game::new(Some(1));
        game.enemies.clear();
        game.poison_turns = 1;
        game.hp = 10;
        game.player = Pos { x: 2, y: 2 };
        game.handle(request(Command::Move { dx: 1, dy: 0 }));
        assert_eq!(game.hp, 9);
        assert_eq!(game.poison_turns, 0);
    }

    #[test]
    fn search_reveals_an_adjacent_trap_without_consuming_a_turn() {
        let mut game = Game::new(Some(1));
        game.enemies.clear();
        game.player = Pos { x: 5, y: 2 };
        let response = game.handle(request(Command::Search));
        assert_eq!(game.turn, 0);
        assert!(game.traps[0].revealed);
        assert_eq!(response.map[2].chars().nth(6), Some('^'));
        assert!(response.message.contains("trap"));
    }

    #[test]
    fn search_reveals_a_trap_within_two_tiles_without_consuming_a_turn() {
        let mut game = Game::new(Some(1));
        game.enemies.clear();
        game.player = Pos { x: 4, y: 2 };
        let response = game.handle(request(Command::Search));
        assert_eq!(game.turn, 0);
        assert!(game.traps[0].revealed);
        assert_eq!(response.map[2].chars().nth(6), Some('^'));
    }

    #[test]
    fn fire_trap_deals_immediate_damage_without_poison() {
        let mut game = Game::new(Some(1));
        game.enemies.clear();
        game.player = Pos { x: 4, y: 3 };
        game.handle(request(Command::Move { dx: 1, dy: 0 }));
        assert_eq!(game.hp, 16);
        assert_eq!(game.poison_turns, 0);
        assert!(game.traps[1].triggered);
    }

    #[test]
    fn drinking_a_potion_heals_and_consumes_a_turn() {
        let mut game = Game::new(Some(1));
        game.enemies.clear();
        game.player = Pos { x: 7, y: 2 };
        game.handle(request(Command::Pickup));
        game.hp = 10;
        let response = game.handle(request(Command::Drink));
        assert_eq!(game.potions, 0);
        assert_eq!(game.hp, 15);
        assert_eq!(game.turn, 2);
        assert!(response.error.is_none());
    }
}
