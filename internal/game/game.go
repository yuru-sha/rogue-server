package game

import (
	"encoding/json"
	"fmt"
	"strings"
)

const (
	Width  = 80
	Height = 24
)

type Pos struct {
	X, Y int `json:"-"`
}
type tile uint8

const (
	wall tile = iota
	floor
	up
	down
)

type item struct {
	Pos        Pos
	Kind       string
	Bonus      int
	Identified bool
	Cursed     bool
}
type enemy struct {
	Pos                 Pos
	Kind                string
	HP, Attack, Defense int
	ID                  int
}
type trap struct {
	Pos                 Pos
	Kind                string
	Triggered, Revealed bool
}
type stack struct {
	Kind               string
	Bonus              int
	Identified, Cursed bool
	Qty                uint32
}
type Game struct {
	Seed                                                                               uint64 `json:"-"`
	tiles                                                                              []tile
	known                                                                              []bool
	player                                                                             Pos
	stair, upStair                                                                     Pos
	FloorNo                                                                            uint8
	floorStates                                                                        [2]*floorState
	enemies                                                                            []enemy
	items                                                                              []item
	traps                                                                              []trap
	rng                                                                                uint64
	Turn                                                                               uint64
	Ended                                                                              bool
	Result                                                                             *string
	Score                                                                              *int
	HP, MaxHP, Hunger                                                                  int
	Food, Weapons, Potions, StrengthPotions, Amulets, Scrolls, TeleportScrolls, Arrows uint32
	WeaponBonus, ArmorBonus, RingBonus, StaffBonus                                     int
	equippedWeapon, equippedArmor, equippedRing, equippedStaff                         *stack
	StrengthTurns                                                                      uint32
	Armor, Rings, Staffs                                                               uint32
	PoisonTurns                                                                        uint32
	Level, XP, Defeated                                                                uint32
	inventory                                                                          []stack
}
type floorState struct {
	tiles          []tile
	known          []bool
	stair, upStair Pos
	enemies        []enemy
	items          []item
	traps          []trap
}
type ErrorBody struct {
	Code    string `json:"code"`
	Message string `json:"message"`
}
type Response struct {
	Type            string     `json:"type"`
	Version         uint8      `json:"version"`
	RequestID       string     `json:"request_id"`
	Turn            uint64     `json:"turn"`
	Ended           bool       `json:"ended"`
	Result          *string    `json:"result"`
	Score           *int       `json:"score"`
	Floor           uint8      `json:"floor"`
	HP              int        `json:"hp"`
	MaxHP           int        `json:"max_hp"`
	Hunger          int        `json:"hunger"`
	Food            uint32     `json:"food"`
	Weapons         uint32     `json:"weapons"`
	WeaponBonus     int        `json:"weapon_bonus"`
	StrengthTurns   uint32     `json:"strength_turns"`
	Armor           uint32     `json:"armor"`
	ArmorBonus      int        `json:"armor_bonus"`
	RingBonus       int        `json:"ring_bonus"`
	Rings           uint32     `json:"rings"`
	StaffBonus      int        `json:"staff_bonus"`
	Staffs          uint32     `json:"staffs"`
	Potions         uint32     `json:"potions"`
	StrengthPotions uint32     `json:"strength_potions"`
	Amulets         uint32     `json:"amulets"`
	Scrolls         uint32     `json:"scrolls"`
	TeleportScrolls uint32     `json:"teleport_scrolls"`
	Arrows          uint32     `json:"arrows"`
	PoisonTurns     uint32     `json:"poison_turns"`
	Level           uint32     `json:"level"`
	XP              uint32     `json:"xp"`
	Defeated        uint32     `json:"defeated"`
	Seed            uint64     `json:"seed"`
	Map             []string   `json:"map"`
	Message         string     `json:"message"`
	Error           *ErrorBody `json:"error"`
}

func New(seed *uint64) *Game {
	seedValue := uint64(1)
	if seed != nil {
		seedValue = *seed
	}
	rngSeed := seedValue
	if rngSeed == 0 {
		rngSeed = 1
	}
	g := &Game{Seed: seedValue, rng: rngSeed, tiles: make([]tile, Width*Height), known: make([]bool, Width*Height), player: Pos{2, 2}, stair: Pos{54, 18}, upStair: Pos{3, 2}, HP: 20, MaxHP: 20, Hunger: 100, FloorNo: 1, Arrows: 3, Level: 1}
	rooms := [][4]int{{2, 2, 12, 7}, {22, 2, 14, 7}, {46, 2, 14, 7}, {10, 14, 14, 7}, {38, 14, 18, 7}}
	for _, r := range rooms {
		for y := r[1]; y < r[1]+r[3]; y++ {
			for x := r[0]; x < r[0]+r[2]; x++ {
				g.tiles[idx(Pos{x, y})] = floor
			}
		}
	}
	for i := 0; i < len(rooms)-1; i++ {
		a, b := rooms[i], rooms[i+1]
		x, y := a[0]+a[2]/2, a[1]+a[3]/2
		tx := b[0] + 2
		for x != tx {
			g.tiles[idx(Pos{x, y})] = floor
			if x < tx {
				x++
			} else {
				x--
			}
		}
		ty := b[1] + b[3]/2
		for y != ty {
			g.tiles[idx(Pos{tx, y})] = floor
			if y < ty {
				y++
			} else {
				y--
			}
		}
	}
	g.tiles[idx(g.stair)] = down
	for len(g.enemies) < 5 {
		p := Pos{8 + int(g.rand(60)), 3 + int(g.rand(16))}
		if g.tiles[idx(p)] != floor || p == g.player || p == g.stair || abs(p.X-g.player.X)+abs(p.Y-g.player.Y) <= 4 || g.enemyAt(p) >= 0 {
			continue
		}
		k := []string{"goblin", "bat", "troll"}[g.rand(3)]
		hp, atk, def := 5, 2, 0
		if k == "bat" {
			hp, atk = 3, 1
		}
		if k == "troll" {
			hp, atk, def = 12, 3, 1
		}
		g.enemies = append(g.enemies, enemy{p, k, hp, atk, def, len(g.enemies)})
	}
	for _, v := range []struct {
		x, y  int
		k     string
		b     int
		known bool
	}{{3, 2, "food", 0, true}, {4, 2, "weapon", 1, true}, {5, 2, "armor", 1, true}, {7, 2, "potion", 0, true}, {11, 2, "strength_potion", 0, true}, {8, 2, "scroll", 0, true}, {12, 2, "teleport_scroll", 0, true}, {13, 2, "ring", 1, true}, {14, 2, "staff", 3, true}, {15, 2, "ring", -1, false}, {16, 2, "staff", -2, false}, {9, 2, "weapon", -1, false}, {10, 2, "armor", -1, false}} {
		g.items = append(g.items, item{Pos{v.x, v.y}, v.k, v.b, v.known, v.b < 0})
	}
	g.traps = []trap{{Pos{6, 2}, "poison", false, false}, {Pos{5, 3}, "fire", false, false}}
	g.reveal()
	return g
}
func (g *Game) rand(n uint64) uint64 { g.rng = g.rng*6364136223846793005 + 1; return g.rng % n }
func idx(p Pos) int                  { return p.Y*Width + p.X }
func abs(n int) int {
	if n < 0 {
		return -n
	}
	return n
}
func (g *Game) enemyAt(p Pos) int {
	for i, e := range g.enemies {
		if e.Pos == p {
			return i
		}
	}
	return -1
}
func (g *Game) reveal() {
	for y := 0; y < Height; y++ {
		for x := 0; x < Width; x++ {
			if abs(x-g.player.X) <= 6 && abs(y-g.player.Y) <= 6 {
				g.known[idx(Pos{x, y})] = true
			}
		}
	}
}
func (g *Game) InitialJSON() []byte { return marshal(g.snapshot("Welcome to the dungeon.", "server")) }
func (g *Game) HandleJSON(b []byte) []byte {
	var envelope struct {
		Version   *uint8          `json:"version"`
		RequestID *string         `json:"request_id"`
		Command   json.RawMessage `json:"command"`
	}
	if err := json.Unmarshal(b, &envelope); err != nil || envelope.Version == nil || envelope.RequestID == nil || len(envelope.Command) == 0 {
		return marshal(errorResponse("invalid_json", "invalid JSON", "server"))
	}
	rid := *envelope.RequestID
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(envelope.Command, &raw); err != nil || raw == nil {
		return marshal(errorResponse("invalid_json", "invalid JSON", rid))
	}
	var typ string
	if err := json.Unmarshal(raw["type"], &typ); err != nil || typ == "" {
		return marshal(errorResponse("invalid_json", "invalid JSON", rid))
	}
	getInt := func(key string) (int, bool) {
		value, ok := raw[key]
		if !ok || string(value) == "null" {
			return 0, false
		}
		var result int
		if err := json.Unmarshal(value, &result); err != nil {
			return 0, false
		}
		return result, true
	}
	getStr := func(key string) (string, bool) {
		value, ok := raw[key]
		if !ok || string(value) == "null" {
			return "", false
		}
		var result string
		if err := json.Unmarshal(value, &result); err != nil {
			return "", false
		}
		return result, true
	}
	dx, dy, itemName := 0, 0, ""
	switch typ {
	case "move", "shoot":
		var ok bool
		if dx, ok = getInt("dx"); !ok {
			return marshal(errorResponse("invalid_json", "invalid JSON", rid))
		}
		if dy, ok = getInt("dy"); !ok {
			return marshal(errorResponse("invalid_json", "invalid JSON", rid))
		}
	case "drop", "identify":
		var ok bool
		if itemName, ok = getStr("item"); !ok {
			return marshal(errorResponse("invalid_json", "invalid JSON", rid))
		}
	case "wait", "pickup", "eat", "drink", "drink_strength", "read_scroll", "teleport", "inventory", "equip", "equip_armor", "equip_ring", "equip_staff", "unequip", "unequip_armor", "unequip_ring", "unequip_staff", "search", "quit":
	default:
		return marshal(errorResponse("invalid_json", "invalid JSON", rid))
	}
	if *envelope.Version != 1 {
		return marshal(errorResponse("unsupported_version", "version must be 1", rid))
	}
	if g.Ended {
		return marshal(errorResponse("game_ended", "game is already over", rid))
	}
	msg := ""
	consumed := false
	fail := func(m string) []byte { return marshal(errorResponse("invalid_command", m, rid)) }
	switch typ {
	case "move":
		if abs(dx) > 1 || abs(dy) > 1 || dx == 0 && dy == 0 {
			return fail("unsupported command")
		}
		consumed = g.move(dx, dy, &msg)
		if !consumed {
			return fail(msg)
		}
	case "wait":
		g.HP = min(g.HP+1, g.MaxHP)
		msg = "You wait."
		consumed = true
	case "pickup":
		i := -1
		for j, v := range g.items {
			if v.Pos == g.player {
				i = j
				break
			}
		}
		if i < 0 {
			return fail("There is nothing to pick up here.")
		}
		if g.slots() >= 20 {
			return marshal(errorResponse("inventory_full", "Your inventory is full.", rid))
		}
		v := g.items[i]
		g.addStack(v)
		g.count(v.Kind, 1)
		g.items = append(g.items[:i], g.items[i+1:]...)
		msg = "You pick up an item."
		consumed = true
	case "eat":
		if g.Food == 0 {
			return fail("You have no food.")
		}
		g.Food--
		g.remove("food")
		g.Hunger = min(100, g.Hunger+25)
		msg = "You eat the food."
		consumed = true
	case "drink":
		if g.Potions == 0 {
			return fail("You have no potion.")
		}
		g.Potions--
		g.remove("potion")
		g.HP = min(g.MaxHP, g.HP+5)
		msg = "You drink a healing potion."
		consumed = true
	case "drink_strength":
		if g.StrengthPotions == 0 {
			return fail("You have no strength potion.")
		}
		g.StrengthPotions--
		g.remove("strength_potion")
		g.StrengthTurns = 10
		msg = "You drink a strength potion."
		consumed = true
	case "read_scroll":
		if g.Scrolls == 0 {
			return fail("You have no mapping scroll.")
		}
		g.Scrolls--
		g.remove("scroll")
		for i := range g.known {
			g.known[i] = true
		}
		msg = "The dungeon map is revealed."
		consumed = true
	case "inventory":
		msg = g.inventoryMessage()
	case "search":
		found := false
		for i := range g.traps {
			t := &g.traps[i]
			if abs(t.Pos.X-g.player.X) <= 2 && abs(t.Pos.Y-g.player.Y) <= 2 && !t.Triggered {
				t.Revealed = true
				found = true
			}
		}
		if found {
			msg = "You find a trap."
		} else {
			msg = "You find nothing."
		}
	case "quit":
		g.end("quit")
	case "shoot":
		if (dx == 0) == (dy == 0) || abs(dx) > 1 || abs(dy) > 1 {
			return fail("unsupported command")
		}
		if g.Arrows == 0 {
			return fail("You have no arrows.")
		}
		g.Arrows--
		g.shoot(dx, dy, &msg)
		consumed = true
	case "teleport":
		if g.TeleportScrolls == 0 {
			return fail("You have no teleport scroll.")
		}
		if p, ok := g.randomFloor(); ok {
			g.TeleportScrolls--
			g.remove("teleport_scroll")
			g.player = p
			msg = "You teleport to another part of the dungeon."
			consumed = true
		} else {
			return fail("There is no safe teleport destination.")
		}
	case "identify":
		name := itemName
		i := -1
		for j, s := range g.inventory {
			if !s.Identified && s.Kind == name {
				i = j
				break
			}
		}
		if i < 0 {
			return fail("There is no unidentified item of that kind.")
		}
		g.inventory[i].Identified = true
		if g.inventory[i].Cursed {
			msg = "You identify a cursed " + name + "."
		} else {
			msg = "You identify the " + name + "."
		}
	case "drop":
		name := itemName
		dropped, ok := g.take(name)
		if !ok {
			return fail("You do not have that item.")
		}
		g.items = append(g.items, item{g.player, dropped.Kind, dropped.Bonus, dropped.Identified, dropped.Cursed})
		msg = "You drop an item."
		consumed = true
	case "equip", "equip_armor", "equip_ring", "equip_staff":
		name := "weapon"
		if typ == "equip_armor" {
			name = "armor"
		}
		if typ == "equip_ring" {
			name = "ring"
		}
		if typ == "equip_staff" {
			name = "staff"
		}
		slot := g.equippedSlot(name)
		if *slot != nil {
			return fail("You already have a " + name + " equipped.")
		}
		i := -1
		for j, s := range g.inventory {
			if s.Kind == name {
				i = j
				break
			}
		}
		if i < 0 {
			return fail("You have no " + name + " to equip.")
		}
		s := g.inventory[i]
		s.Qty = 1
		*slot = &s
		if g.inventory[i].Qty > 1 {
			g.inventory[i].Qty--
		} else {
			g.inventory = append(g.inventory[:i], g.inventory[i+1:]...)
		}
		g.decrementCount(name)
		g.setEquipmentBonus(name, s.Bonus)
		msg = "You equip the " + name + "."
		consumed = true
	case "unequip", "unequip_armor", "unequip_ring", "unequip_staff":
		name := "weapon"
		if typ == "unequip_armor" {
			name = "armor"
		}
		if typ == "unequip_ring" {
			name = "ring"
		}
		if typ == "unequip_staff" {
			name = "staff"
		}
		slot := g.equippedSlot(name)
		if *slot == nil {
			return fail("You have no equipped " + name + ".")
		}
		if (*slot).Cursed {
			return fail("The cursed " + name + " cannot be removed.")
		}
		equipped := **slot
		canMerge := false
		for _, carried := range g.inventory {
			if carried.Kind == equipped.Kind && carried.Bonus == equipped.Bonus && carried.Identified == equipped.Identified && carried.Cursed == equipped.Cursed {
				canMerge = true
				break
			}
		}
		if g.slots() >= 20 && !canMerge {
			return marshal(errorResponse("inventory_full", "Your inventory is full.", rid))
		}
		g.addInventoryStack(equipped)
		g.count(name, 1)
		g.setEquipmentBonus(name, 0)
		*slot = nil
		msg = "You remove the " + name + "."
		consumed = true
	default:
		return fail("unsupported command")
	}
	if consumed && !g.Ended {
		g.Turn++
		g.poison(&msg)
		if !g.Ended {
			g.trigger(&msg)
		}
		if !g.Ended {
			g.hunger(&msg)
		}
		if !g.Ended {
			transition := g.transition(&msg)
			g.enemyTurn(&msg)
			if !g.Ended {
				if g.StrengthTurns > 0 {
					g.StrengthTurns--
				}
				if transition && g.FloorNo == 1 && g.Amulets > 0 {
					g.end("victory")
					msg = "You return to the surface with the amulet."
				}
			}
		}
	}
	g.reveal()
	return marshal(g.snapshot(msg, rid))
}
func marshal(v any) []byte { b, _ := json.Marshal(v); return b }
func errorResponse(code, msg, rid string) Response {
	return Response{Type: "error", Version: 1, RequestID: rid, Map: []string{}, Error: &ErrorBody{code, msg}}
}
func (g *Game) snapshot(msg, rid string) Response {
	rows := make([]string, Height)
	for y := 0; y < Height; y++ {
		r := make([]byte, Width)
		for x := 0; x < Width; x++ {
			p := Pos{x, y}
			if !g.known[idx(p)] {
				r[x] = ' '
				continue
			}
			c := byte('#')
			switch g.tiles[idx(p)] {
			case floor:
				c = '.'
			case up:
				c = '<'
			case down:
				c = '>'
			}
			if abs(x-g.player.X) > 6 || abs(y-g.player.Y) > 6 {
				if c >= 'A' && c <= 'Z' {
					c += 32
				}
			}
			r[x] = c
		}
		for _, it := range g.items {
			if it.Pos.Y != y {
				continue
			}
			if abs(it.Pos.X-g.player.X) <= 6 && abs(it.Pos.Y-g.player.Y) <= 6 {
				r[it.Pos.X] = itemSymbol(it.Kind)
			}
		}
		for _, e := range g.enemies {
			if e.Pos.Y != y {
				continue
			}
			if abs(e.Pos.X-g.player.X) <= 6 && abs(e.Pos.Y-g.player.Y) <= 6 {
				c := byte('g')
				if e.Kind == "bat" {
					c = 'b'
				}
				if e.Kind == "troll" {
					c = 'T'
				}
				r[e.Pos.X] = c
			}
		}
		for _, t := range g.traps {
			if t.Pos.Y != y {
				continue
			}
			if t.Revealed && !t.Triggered && g.known[idx(t.Pos)] {
				r[t.Pos.X] = '^'
			}
		}
		if y == g.player.Y {
			r[g.player.X] = '@'
		}
		rows[y] = string(r)
	}
	return Response{Type: "snapshot", Version: 1, RequestID: rid, Turn: g.Turn, Ended: g.Ended, Result: g.Result, Score: g.Score, Floor: g.FloorNo, HP: g.HP, MaxHP: g.MaxHP, Hunger: g.Hunger, Food: g.Food, Weapons: g.Weapons, WeaponBonus: g.WeaponBonus, StrengthTurns: g.StrengthTurns, Armor: g.Armor, ArmorBonus: g.ArmorBonus, RingBonus: g.RingBonus, Rings: g.Rings, StaffBonus: g.StaffBonus, Staffs: g.Staffs, Potions: g.Potions, StrengthPotions: g.StrengthPotions, Amulets: g.Amulets, Scrolls: g.Scrolls, TeleportScrolls: g.TeleportScrolls, Arrows: g.Arrows, PoisonTurns: g.PoisonTurns, Level: g.Level, XP: g.XP, Defeated: g.Defeated, Seed: g.Seed, Map: rows, Message: msg}
}
func itemSymbol(kind string) byte {
	switch kind {
	case "food":
		return '!'
	case "weapon":
		return ')'
	case "armor":
		return '['
	case "ring":
		return '='
	case "staff":
		return '/'
	case "potion", "strength_potion":
		return '?'
	case "amulet":
		return ','
	case "scroll", "teleport_scroll":
		return '~'
	default:
		return ' '
	}
}
func (g *Game) count(k string, n uint32) {
	switch k {
	case "food":
		g.Food += n
	case "weapon":
		g.Weapons += n
	case "armor":
		g.Armor += n
	case "ring":
		g.Rings += n
	case "staff":
		g.Staffs += n
	case "potion":
		g.Potions += n
	case "strength_potion":
		g.StrengthPotions += n
	case "amulet":
		g.Amulets += n
	case "scroll":
		g.Scrolls += n
	case "teleport_scroll":
		g.TeleportScrolls += n
	}
}
func (g *Game) decrementCount(k string) {
	switch k {
	case "food":
		g.Food--
	case "weapon":
		g.Weapons--
	case "armor":
		g.Armor--
	case "ring":
		g.Rings--
	case "staff":
		g.Staffs--
	case "potion":
		g.Potions--
	case "strength_potion":
		g.StrengthPotions--
	case "amulet":
		g.Amulets--
	case "scroll":
		g.Scrolls--
	case "teleport_scroll":
		g.TeleportScrolls--
	}
}
func (g *Game) equippedSlot(kind string) **stack {
	switch kind {
	case "weapon":
		return &g.equippedWeapon
	case "armor":
		return &g.equippedArmor
	case "ring":
		return &g.equippedRing
	default:
		return &g.equippedStaff
	}
}
func (g *Game) setEquipmentBonus(kind string, bonus int) {
	switch kind {
	case "weapon":
		g.WeaponBonus = bonus
	case "armor":
		g.ArmorBonus = bonus
	case "ring":
		g.RingBonus = bonus
	case "staff":
		g.StaffBonus = bonus
	}
}
func (g *Game) addStack(v item) {
	g.addInventoryStack(stack{Kind: v.Kind, Bonus: v.Bonus, Identified: v.Identified, Cursed: v.Cursed, Qty: 1})
}
func (g *Game) addInventoryStack(v stack) {
	for i := range g.inventory {
		s := &g.inventory[i]
		if s.Kind == v.Kind && s.Bonus == v.Bonus && s.Identified == v.Identified && s.Cursed == v.Cursed {
			s.Qty += v.Qty
			return
		}
	}
	g.inventory = append(g.inventory, v)
}
func (g *Game) slots() int {
	if len(g.inventory) > 0 {
		return len(g.inventory)
	}
	return int(g.Food + g.Weapons + g.Armor + g.Rings + g.Staffs + g.Potions + g.StrengthPotions + g.Amulets + g.Scrolls + g.TeleportScrolls)
}
func (g *Game) remove(k string) {
	for i := range g.inventory {
		if g.inventory[i].Kind == k {
			if g.inventory[i].Qty > 1 {
				g.inventory[i].Qty--
			} else {
				g.inventory = append(g.inventory[:i], g.inventory[i+1:]...)
			}
			return
		}
	}
}
func (g *Game) inventoryMessage() string {
	items := []string{}
	for _, s := range g.inventory {
		c := "unknown"
		if s.Identified {
			c = fmt.Sprint(s.Cursed)
		}
		name := s.Kind
		name = strings.ReplaceAll(name, "_", " ")
		items = append(items, fmt.Sprintf("%s x%d (identified=%t, cursed=%s)", name, s.Qty, s.Identified, c))
	}
	return fmt.Sprintf("Inventory: food %d potions %d strength potions %d scrolls %d teleport scrolls %d amulets %d weapons %d armor %d rings %d staffs %d. Items: %s.", g.Food, g.Potions, g.StrengthPotions, g.Scrolls, g.TeleportScrolls, g.Amulets, g.Weapons, g.Armor, g.Rings, g.Staffs, strings.Join(items, ", "))
}
func (g *Game) move(dx, dy int, msg *string) bool {
	p := Pos{g.player.X + dx, g.player.Y + dy}
	if p.X <= 0 || p.X >= Width-1 || p.Y <= 0 || p.Y >= Height-1 || g.tiles[idx(p)] == wall || dx != 0 && dy != 0 && (g.tiles[idx(Pos{p.X, g.player.Y})] == wall || g.tiles[idx(Pos{g.player.X, p.Y})] == wall) {
		*msg = "You cannot move there."
		return false
	}
	if i := g.enemyAt(p); i >= 0 {
		d := max(1, 4+int(g.rand(4))+2+g.WeaponBonus+g.StaffBonus+boost(g.StrengthTurns)-g.enemies[i].Defense)
		g.enemies[i].HP -= d
		*msg = fmt.Sprintf("You hit the enemy for %d damage.", d)
		if g.enemies[i].HP <= 0 {
			xp := enemyXP(g.enemies[i].Kind)
			g.enemies = append(g.enemies[:i], g.enemies[i+1:]...)
			g.Defeated++
			g.XP += xp
			t := g.Level * 20
			if g.Level < 12 && g.XP >= t {
				g.XP -= t
				g.Level++
				g.MaxHP += 2
				g.HP = g.MaxHP
				*msg = fmt.Sprintf("You defeated the enemy and reached level %d.", g.Level)
			} else {
				*msg = "You defeated the enemy."
			}
		}
		return true
	}
	g.player = p
	return true
}
func enemyXP(kind string) uint32 {
	switch kind {
	case "bat":
		return 8
	case "troll":
		return 20
	default:
		return 10
	}
}
func boost(n uint32) int {
	if n > 0 {
		return 2
	}
	return 0
}
func (g *Game) shoot(dx, dy int, msg *string) {
	p := g.player
	for {
		p.X += dx
		p.Y += dy
		if p.X <= 0 || p.X >= Width-1 || p.Y <= 0 || p.Y >= Height-1 || g.tiles[idx(p)] == wall {
			*msg = "You shoot into the darkness."
			return
		}
		if i := g.enemyAt(p); i >= 0 {
			d := max(1, 3+g.WeaponBonus+g.StaffBonus+boost(g.StrengthTurns)-g.enemies[i].Defense)
			g.enemies[i].HP -= d
			*msg = fmt.Sprintf("You shoot the enemy for %d damage.", d)
			if g.enemies[i].HP <= 0 {
				kind := g.enemies[i].Kind
				g.enemies = append(g.enemies[:i], g.enemies[i+1:]...)
				g.Defeated++
				g.XP += enemyXP(kind)
			}
			return
		}
	}
}
func (g *Game) canSee(p Pos) bool {
	dx, dy := g.player.X-p.X, g.player.Y-p.Y
	if abs(dx) > 8 || abs(dy) > 8 || dx != 0 && dy != 0 && abs(dx) != abs(dy) {
		return false
	}
	sx, sy := sign(dx), sign(dy)
	for p != g.player {
		p.X += sx
		p.Y += sy
		if p != g.player && g.tiles[idx(p)] == wall {
			return false
		}
	}
	return true
}
func sign(n int) int {
	if n < 0 {
		return -1
	}
	if n > 0 {
		return 1
	}
	return 0
}
func (g *Game) enemyTurn(msg *string) {
	occupied := make([]Pos, len(g.enemies))
	for i, e := range g.enemies {
		occupied[i] = e.Pos
	}
	for i := range g.enemies {
		e := &g.enemies[i]
		if !g.canSee(e.Pos) || e.Kind == "troll" && g.Turn%2 == 1 {
			continue
		}
		if e.Kind == "troll" && e.HP < 12 {
			e.HP++
		}
		retreat := e.Kind == "bat" && e.HP <= 1
		dx, dy := sign(g.player.X-e.Pos.X), sign(g.player.Y-e.Pos.Y)
		if retreat {
			dx, dy = -dx, -dy
		}
		if abs(e.Pos.X-g.player.X) <= 1 && abs(e.Pos.Y-g.player.Y) <= 1 {
			g.HP -= max(1, e.Attack-1-g.ArmorBonus-g.RingBonus)
			*msg = "The enemy hits you."
			if g.HP <= 0 {
				g.end("death")
				return
			}
			continue
		}
		n := Pos{e.Pos.X + dx, e.Pos.Y + dy}
		if g.tiles[idx(n)] != wall && n != g.player && (e.Kind == "bat" || dx == 0 || dy == 0 || g.tiles[idx(Pos{e.Pos.X + dx, e.Pos.Y})] != wall && g.tiles[idx(Pos{e.Pos.X, e.Pos.Y + dy})] != wall) {
			free := true
			for j, p := range occupied {
				if j != i && p == n {
					free = false
				}
			}
			if free {
				e.Pos = n
				occupied[i] = n
			}
		}
	}
}
func (g *Game) poison(msg *string) {
	if g.PoisonTurns > 0 {
		g.PoisonTurns--
		g.HP--
		*msg = "Poison harms you."
		if g.HP <= 0 {
			g.end("death")
		}
	}
}
func (g *Game) trigger(msg *string) {
	for i := range g.traps {
		t := &g.traps[i]
		if t.Pos == g.player && !t.Triggered {
			t.Triggered = true
			if t.Kind == "poison" {
				g.PoisonTurns = 3
				g.HP--
				*msg = "A poison trap wounds you."
			} else {
				g.HP -= 4
				*msg = "A fire trap burns you."
			}
			if g.HP <= 0 {
				g.end("death")
			}
			return
		}
	}
}
func (g *Game) hunger(msg *string) {
	if g.Hunger > 0 {
		g.Hunger--
	} else {
		g.HP--
		*msg = "Starvation hurts you."
		if g.HP <= 0 {
			g.end("death")
		}
	}
}
func (g *Game) end(r string) {
	g.Ended = true
	g.Result = &r
	s := int(g.FloorNo)*100 + int(g.Defeated)*100 + max(0, g.HP) + g.Hunger - int(g.Turn)
	if r == "victory" {
		s += 1000
	}
	if s < 0 {
		s = 0
	}
	g.Score = &s
}
func (g *Game) capture() *floorState {
	return &floorState{append([]tile(nil), g.tiles...), append([]bool(nil), g.known...), g.stair, g.upStair, append([]enemy(nil), g.enemies...), append([]item(nil), g.items...), append([]trap(nil), g.traps...)}
}
func (g *Game) restore(s *floorState) {
	g.tiles = s.tiles
	g.known = s.known
	g.stair = s.stair
	g.upStair = s.upStair
	g.enemies = s.enemies
	g.items = s.items
	g.traps = s.traps
}
func (g *Game) transition(msg *string) bool {
	if g.FloorNo == 1 && g.player == g.stair {
		g.floorStates[0] = g.capture()
		n := g.floorStates[1]
		if n == nil {
			n = g.capture()
			n.tiles[idx(n.upStair)] = up
			n.items = append(n.items, item{n.stair, "amulet", 0, true, false})
		}
		g.restore(n)
		g.FloorNo = 2
		g.tiles[idx(g.upStair)] = up
		g.player = g.upStair
		*msg = "You descend to floor 2."
		return true
	}
	if g.FloorNo == 2 && g.player == g.upStair {
		g.floorStates[1] = g.capture()
		g.restore(g.floorStates[0])
		g.FloorNo = 1
		g.player = Pos{g.stair.X - 1, g.stair.Y}
		*msg = "You ascend to floor 1."
		return true
	}
	return false
}
func (g *Game) randomFloor() (Pos, bool) {
	for range Width * Height {
		p := Pos{1 + int(g.rand(Width-2)), 1 + int(g.rand(Height-2))}
		if g.tiles[idx(p)] != floor || p == g.player || g.enemyAt(p) >= 0 {
			continue
		}
		occupied := false
		for _, v := range g.items {
			if v.Pos == p {
				occupied = true
				break
			}
		}
		if !occupied {
			return p, true
		}
	}
	return Pos{}, false
}
func (g *Game) take(name string) (stack, bool) {
	for i, s := range g.inventory {
		if s.Kind != name {
			continue
		}
		out := s
		out.Qty = 1
		if s.Qty > 1 {
			g.inventory[i].Qty--
		} else {
			g.inventory = append(g.inventory[:i], g.inventory[i+1:]...)
		}
		g.decrementCount(name)
		return out, true
	}
	return stack{}, false
}
