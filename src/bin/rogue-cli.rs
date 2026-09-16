use futures_util::{SinkExt, StreamExt};
use rogue_server::GameResponse;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().any(|arg| arg == "-h" || arg == "--help") {
        println!(
            "Usage: rogue-cli\n\nConnects to ws://127.0.0.1:8080/ws and accepts keyboard commands.\nSet ROGUE_WS to override the WebSocket endpoint."
        );
        return Ok(());
    }
    let endpoint = std::env::var("ROGUE_WS").unwrap_or_else(|_| "ws://127.0.0.1:8080/ws".into());
    let (mut socket, _) = connect_async(endpoint).await?;
    while let Some(message) = socket.next().await {
        let Message::Text(text) = message? else {
            break;
        };
        let response: GameResponse = serde_json::from_str(&text)?;
        print_snapshot(&response);
        if response.ended {
            break;
        }
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let key = input.trim().chars().next().unwrap_or('.');
        let drop_item = match key {
            '1' => Some("food"),
            '2' => Some("potion"),
            '3' => Some("scroll"),
            '4' => Some("amulet"),
            '5' => Some("weapon"),
            '6' => Some("armor"),
            '7' => Some("strength_potion"),
            '8' => Some("teleport_scroll"),
            '9' => Some("ring"),
            '0' => Some("staff"),
            _ => None,
        };
        let identify_item = match key {
            'I' => Some("weapon"),
            'A' => Some("armor"),
            'G' => Some("ring"),
            'D' => Some("staff"),
            _ => None,
        };
        let command = match key {
            'h' | 'a' => Some(("move", -1, 0)),
            'l' | 'd' => Some(("move", 1, 0)),
            'k' | 'w' => Some(("move", 0, -1)),
            'j' | 's' => Some(("move", 0, 1)),
            'y' => Some(("move", -1, -1)),
            'u' => Some(("move", 1, -1)),
            'b' => Some(("move", -1, 1)),
            'n' => Some(("move", 1, 1)),
            '.' => Some(("wait", 0, 0)),
            'g' => Some(("pickup", 0, 0)),
            'e' => Some(("eat", 0, 0)),
            'p' => Some(("drink", 0, 0)),
            'P' => Some(("drink_strength", 0, 0)),
            'r' => Some(("read_scroll", 0, 0)),
            'T' => Some(("teleport", 0, 0)),
            'i' => Some(("inventory", 0, 0)),
            'x' => Some(("equip", 0, 0)),
            'z' => Some(("equip_armor", 0, 0)),
            'v' => Some(("equip_ring", 0, 0)),
            'c' => Some(("equip_staff", 0, 0)),
            'U' => Some(("unequip", 0, 0)),
            'O' => Some(("unequip_armor", 0, 0)),
            'V' => Some(("unequip_ring", 0, 0)),
            'C' => Some(("unequip_staff", 0, 0)),
            'f' => Some(("shoot", 1, 0)),
            'F' => Some(("shoot", -1, 0)),
            'R' => Some(("shoot", 0, -1)),
            'B' => Some(("shoot", 0, 1)),
            'S' => Some(("search", 0, 0)),
            'q' => Some(("quit", 0, 0)),
            _ => None,
        };
        let Some((kind, dx, dy)) = command else {
            continue;
        };
        let body = if let Some(item) = identify_item {
            serde_json::json!({"version":1,"request_id":"cli","command":{"type":"identify","item":item}})
        } else if let Some(item) = drop_item {
            serde_json::json!({"version":1,"request_id":"cli","command":{"type":"drop","item":item}})
        } else if kind == "move" {
            serde_json::json!({"version":1,"request_id":"cli","command":{"type":kind,"dx":dx,"dy":dy}})
        } else {
            serde_json::json!({"version":1,"request_id":"cli","command":{"type":kind}})
        };
        socket.send(Message::Text(body.to_string().into())).await?;
    }
    Ok(())
}

fn print_snapshot(response: &GameResponse) {
    print!("\x1b[2J\x1b[H");
    println!(
        "Floor {}  HP {}/{}  hunger {}  poison {}  food {}  potions {}  strength potions {}  strength {}  scrolls {}  teleport scrolls {}  amulets {}  arrows {}  weapons {}  weapon+{}  armor {}  armor+{}  rings {}  ring+{}  staffs {}  staff+{}  level {}  XP {}  defeated {}  turn {}  score {:?}  seed {}",
        response.floor,
        response.hp,
        response.max_hp,
        response.hunger,
        response.poison_turns,
        response.food,
        response.potions,
        response.strength_potions,
        response.strength_turns,
        response.scrolls,
        response.teleport_scrolls,
        response.amulets,
        response.arrows,
        response.weapons,
        response.weapon_bonus,
        response.armor,
        response.armor_bonus,
        response.rings,
        response.ring_bonus,
        response.staffs,
        response.staff_bonus,
        response.level,
        response.xp,
        response.defeated,
        response.turn,
        response.score,
        response.seed
    );
    for row in &response.map {
        println!("{row}");
    }
    println!(
        "{}\n[hjklyubn] move  [.] wait  [g] pickup  [e] eat  [p/P] drink healing/strength potion  [r] read map scroll  [T] teleport  [i] inventory  [I/A/G/D] identify weapon/armor/ring/staff  [x/z/v/c] equip weapon/armor/ring/staff  [U/O/V/C] unequip weapon/armor/ring/staff  [1-0] drop food/potion/scroll/amulet/weapon/armor/strength/teleport scroll/ring/staff  [f/F/R/B] shoot east/west/north/south  [S] search  [q] quit",
        response.message
    );
}
