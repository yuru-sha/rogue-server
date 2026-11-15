package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"os"

	"github.com/gorilla/websocket"
)

func main() {
	flag.Usage = printUsage
	help := flag.Bool("help", false, "show usage")
	flag.Parse()
	if *help {
		printUsage()
		return
	}
	endpoint := os.Getenv("ROGUE_WS")
	if endpoint == "" {
		endpoint = "ws://127.0.0.1:8080/ws"
	}
	conn, _, err := websocket.DefaultDialer.Dial(endpoint, nil)
	if err != nil {
		fatal(err)
	}
	defer conn.Close()
	for {
		_, data, err := conn.ReadMessage()
		if err != nil {
			fatal(err)
		}
		var response map[string]any
		if err := json.Unmarshal(data, &response); err != nil {
			fatal(err)
		}
		printSnapshot(response)
		if response["ended"] == true {
			return
		}
		var line string
		if _, err := fmt.Fscanln(os.Stdin, &line); err != nil {
			return
		}
		command := commandForKey([]rune(line))
		if command == nil {
			continue
		}
		request := map[string]any{"version": 1, "request_id": "cli", "command": command}
		if err := conn.WriteJSON(request); err != nil {
			fatal(err)
		}
	}
}

func printSnapshot(response map[string]any) {
	fmt.Print("\x1b[2J\x1b[H")
	if rows, ok := response["map"].([]any); ok {
		for _, row := range rows {
			fmt.Println(row)
		}
	}
	fmt.Printf("Floor %v  HP %v/%v  hunger %v  poison %v  food %v  potions %v  strength potions %v  strength %v  scrolls %v  teleport scrolls %v  amulets %v  arrows %v  weapons %v  weapon+%v  armor %v  armor+%v  rings %v  ring+%v  staffs %v  staff+%v  level %v  XP %v  defeated %v  turn %v  score %v  seed %v\n",
		response["floor"], response["hp"], response["max_hp"], response["hunger"], response["poison_turns"],
		response["food"], response["potions"], response["strength_potions"], response["strength_turns"],
		response["scrolls"], response["teleport_scrolls"], response["amulets"], response["arrows"],
		response["weapons"], response["weapon_bonus"], response["armor"], response["armor_bonus"],
		response["rings"], response["ring_bonus"], response["staffs"], response["staff_bonus"],
		response["level"], response["xp"], response["defeated"], response["turn"], displayScore(response["score"]), response["seed"])
	fmt.Println(response["message"])
	fmt.Println("[hjklyubn] move  [.] wait  [g] pickup  [e] eat  [p/P] drink  [r] map  [T] teleport  [i] inventory  [x/z/v/c] equip  [U/O/V/C] unequip  [1-0] drop  [f/F/R/B] shoot  [S] search  [q] quit")
}
func printUsage() {
	fmt.Println("Usage: rogue-cli\n\nConnects to ws://127.0.0.1:8080/ws and accepts keyboard commands.\nSet ROGUE_WS to override the WebSocket endpoint.")
}

func displayScore(score any) string {
	value, ok := score.(float64)
	if !ok {
		return "None"
	}
	return fmt.Sprintf("Some(%d)", int(value))
}

func commandForKey(keys []rune) map[string]any {
	if len(keys) == 0 {
		return nil
	}
	key := keys[0]
	dropItems := []string{"food", "potion", "scroll", "amulet", "weapon", "armor", "strength_potion", "teleport_scroll", "ring", "staff"}
	if key >= '1' && key <= '9' {
		return map[string]any{"type": "drop", "item": dropItems[key-'1']}
	}
	if key == '0' {
		return map[string]any{"type": "drop", "item": "staff"}
	}
	identify := map[rune]string{'I': "weapon", 'A': "armor", 'G': "ring", 'D': "staff"}
	if item, ok := identify[key]; ok {
		return map[string]any{"type": "identify", "item": item}
	}
	moves := map[rune][2]int{'h': {-1, 0}, 'a': {-1, 0}, 'l': {1, 0}, 'd': {1, 0}, 'k': {0, -1}, 'w': {0, -1}, 'j': {0, 1}, 's': {0, 1}, 'y': {-1, -1}, 'u': {1, -1}, 'b': {-1, 1}, 'n': {1, 1}}
	if delta, ok := moves[key]; ok {
		return map[string]any{"type": "move", "dx": delta[0], "dy": delta[1]}
	}
	commands := map[rune]string{'.': "wait", 'g': "pickup", 'e': "eat", 'p': "drink", 'P': "drink_strength", 'r': "read_scroll", 'T': "teleport", 'i': "inventory", 'x': "equip", 'z': "equip_armor", 'v': "equip_ring", 'c': "equip_staff", 'U': "unequip", 'O': "unequip_armor", 'V': "unequip_ring", 'C': "unequip_staff", 'S': "search", 'q': "quit"}
	if command, ok := commands[key]; ok {
		return map[string]any{"type": command}
	}
	shots := map[rune][2]int{'f': {1, 0}, 'F': {-1, 0}, 'R': {0, -1}, 'B': {0, 1}}
	if delta, ok := shots[key]; ok {
		return map[string]any{"type": "shoot", "dx": delta[0], "dy": delta[1]}
	}
	return nil
}

func fatal(err error) {
	fmt.Fprintln(os.Stderr, err)
	os.Exit(1)
}
