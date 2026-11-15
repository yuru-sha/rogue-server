package game

import (
	"encoding/json"
	"strings"
	"testing"
)

func decodeSnapshot(t *testing.T, b []byte) Response {
	t.Helper()
	var r Response
	if err := json.Unmarshal(b, &r); err != nil {
		t.Fatal(err)
	}
	return r
}

func handle(t *testing.T, g *Game, command string) Response {
	t.Helper()
	return decodeSnapshot(t, g.HandleJSON([]byte(`{"version":1,"request_id":"test","command":`+command+`}`)))
}

func TestSameSeedInitialMap(t *testing.T) {
	seed := uint64(42)
	a, b := New(&seed), New(&seed)
	ra, rb := decodeSnapshot(t, a.InitialJSON()), decodeSnapshot(t, b.InitialJSON())
	if ra.Type != "snapshot" || len(ra.Map) != Height {
		t.Fatalf("unexpected initial response: %+v", ra)
	}
	for y := range ra.Map {
		if ra.Map[y] != rb.Map[y] {
			t.Fatalf("map differs at row %d", y)
		}
	}
	if ra.Seed != rb.Seed {
		t.Fatalf("seed differs: %d != %d", ra.Seed, rb.Seed)
	}
}

func TestHandleNestedWaitRequest(t *testing.T) {
	g := New(nil)
	r := decodeSnapshot(t, g.HandleJSON([]byte(`{"version":1,"request_id":"wait-1","command":{"type":"wait"}}`)))
	if r.Type != "snapshot" || r.RequestID != "wait-1" || r.Turn != 1 || r.Hunger != 99 {
		t.Fatalf("unexpected wait response: %+v", r)
	}
}

func TestInvalidMoveDoesNotConsumeTurn(t *testing.T) {
	g := New(nil)
	r := decodeSnapshot(t, g.HandleJSON([]byte(`{"version":1,"request_id":"wall","command":{"type":"move","dx":-1,"dy":0}}`)))
	if r.Type != "error" || r.Error == nil || r.Error.Code != "invalid_command" || r.Turn != 0 {
		t.Fatalf("unexpected invalid-move response: %+v", r)
	}
}

func TestSearchRevealsTrapWithoutConsumingTurn(t *testing.T) {
	g := New(nil)
	g.player = Pos{4, 2}
	r := decodeSnapshot(t, g.HandleJSON([]byte(`{"version":1,"request_id":"search","command":{"type":"search"}}`)))
	if r.Type != "snapshot" || r.Turn != 0 || !g.traps[0].Revealed {
		t.Fatalf("search should reveal nearby trap without advancing: %+v", r)
	}
}

func TestSnapshotPlacesItemsOnlyOnTheirMapRow(t *testing.T) {
	r := decodeSnapshot(t, New(nil).InitialJSON())
	if r.Map[2][3] != '!' {
		t.Fatalf("food missing from its row: %q", r.Map[2])
	}
	if r.Map[3][3] == '!' {
		t.Fatalf("food leaked onto another row: %q", r.Map[3])
	}
}

func TestEquipmentSlotRejectsStackingAndRestoresExactItem(t *testing.T) {
	g := New(nil)
	g.enemies = nil
	g.player = Pos{4, 2}
	g.items = []item{{Pos: g.player, Kind: "weapon", Bonus: 2, Identified: true}, {Pos: g.player, Kind: "weapon", Bonus: -1, Identified: false, Cursed: true}}
	handle(t, g, `{"type":"pickup"}`)
	handle(t, g, `{"type":"pickup"}`)
	handle(t, g, `{"type":"equip"}`)
	before := g.Turn
	rejected := handle(t, g, `{"type":"equip"}`)
	if rejected.Type != "error" || g.Turn != before || g.WeaponBonus != 2 || g.Weapons != 1 {
		t.Fatalf("occupied slot accepted another item: %+v bonus=%d count=%d", rejected, g.WeaponBonus, g.Weapons)
	}
	handle(t, g, `{"type":"unequip"}`)
	if g.WeaponBonus != 0 || g.Weapons != 2 || g.inventory[0].Bonus != -1 || !g.inventory[0].Cursed {
		t.Fatalf("unequip did not restore metadata and counts: bonus=%d count=%d inventory=%+v", g.WeaponBonus, g.Weapons, g.inventory)
	}
	handle(t, g, `{"type":"equip"}`)
	if g.WeaponBonus != -1 {
		t.Fatalf("re-equipping negative-bonus item lost its exact modifier: %d", g.WeaponBonus)
	}
	if got := handle(t, g, `{"type":"unequip"}`); got.Type != "error" || g.WeaponBonus != -1 {
		t.Fatalf("cursed re-equipped item was removable: %+v bonus=%d", got, g.WeaponBonus)
	}
}

func TestLethalStatusDamageEndsActionBeforeStairs(t *testing.T) {
	cases := []struct {
		name          string
		setup         func(*Game)
		expectedScore int
	}{
		{name: "poison", setup: func(g *Game) { g.HP, g.PoisonTurns = 1, 1 }, expectedScore: 199},
		{name: "fire trap", setup: func(g *Game) { g.HP = 4; g.traps = []trap{{Pos: g.player, Kind: "fire"}} }, expectedScore: 199},
		{name: "starvation", setup: func(g *Game) { g.HP, g.Hunger = 1, 0 }, expectedScore: 99},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			g := New(nil)
			g.enemies = nil
			g.player = Pos{4, 2}
			g.stair = g.player
			g.inventory = []stack{{Kind: "food", Qty: 1}}
			g.Food = 1
			tc.setup(g)
			response := handle(t, g, `{"type":"drop","item":"food"}`)
			if !response.Ended || response.Result == nil || *response.Result != "death" || response.Turn != 1 || response.Floor != 1 || response.Score == nil || *response.Score != tc.expectedScore {
				t.Fatalf("lethal damage did not terminate before transition with updated turn/score: %+v", response)
			}
		})
	}
}

func TestDropAndRepickupPreservesEquipmentMetadata(t *testing.T) {
	g := New(nil)
	g.enemies = nil
	g.player = Pos{9, 2}
	handle(t, g, `{"type":"pickup"}`)
	handle(t, g, `{"type":"identify","item":"weapon"}`)
	handle(t, g, `{"type":"drop","item":"weapon"}`)
	found := false
	for _, v := range g.items {
		if v.Pos == g.player && v.Kind == "weapon" {
			found = v.Bonus == -1 && v.Identified && v.Cursed
		}
	}
	if !found {
		t.Fatalf("drop lost item metadata: %+v", g.items)
	}
	handle(t, g, `{"type":"pickup"}`)
	if len(g.inventory) != 1 || g.inventory[0].Bonus != -1 || !g.inventory[0].Identified || !g.inventory[0].Cursed {
		t.Fatalf("re-pickup lost item metadata: %+v", g.inventory)
	}
}

func TestEveryEquipmentSlotRestoresExactStack(t *testing.T) {
	cases := []struct {
		kind, equip, unequip string
		bonus                int
		cursed               bool
	}{
		{"weapon", "equip", "unequip", 0, false},
		{"armor", "equip_armor", "unequip_armor", 2, false},
		{"ring", "equip_ring", "unequip_ring", -2, true},
		{"staff", "equip_staff", "unequip_staff", 5, false},
	}
	for _, tc := range cases {
		t.Run(tc.kind, func(t *testing.T) {
			g := New(nil)
			g.enemies = nil
			g.inventory = []stack{{Kind: tc.kind, Bonus: tc.bonus, Identified: true, Cursed: tc.cursed, Qty: 1}}
			g.count(tc.kind, 1)
			handle(t, g, `{"type":"`+tc.equip+`"}`)
			if *g.equippedSlot(tc.kind) == nil || (*g.equippedSlot(tc.kind)).Bonus != tc.bonus {
				t.Fatalf("equipped slot lost item record: %+v", *g.equippedSlot(tc.kind))
			}
			response := handle(t, g, `{"type":"`+tc.unequip+`"}`)
			if tc.cursed {
				if response.Type != "error" || *g.equippedSlot(tc.kind) == nil {
					t.Fatalf("cursed item was removable: %+v", response)
				}
				return
			}
			if response.Type != "snapshot" || *g.equippedSlot(tc.kind) != nil || len(g.inventory) != 1 || g.inventory[0].Bonus != tc.bonus || g.inventory[0].Identified != true || g.inventory[0].Cursed != tc.cursed {
				t.Fatalf("unequip did not restore exact record: response=%+v inventory=%+v", response, g.inventory)
			}
		})
	}
}

func TestUnequipAtInventoryCapacityKeepsEquippedItem(t *testing.T) {
	g := New(nil)
	g.enemies = nil
	for bonus := 2; bonus < 22; bonus++ {
		g.inventory = append(g.inventory, stack{Kind: "weapon", Bonus: bonus, Identified: true, Qty: 1})
	}
	equipped := stack{Kind: "weapon", Bonus: 1, Identified: true, Qty: 1}
	g.equippedWeapon = &equipped
	g.WeaponBonus = equipped.Bonus
	g.Weapons = uint32(len(g.inventory))
	response := handle(t, g, `{"type":"unequip"}`)
	if response.Type != "error" || response.Error == nil || response.Error.Code != "inventory_full" || g.equippedWeapon == nil || g.WeaponBonus != 1 || g.Weapons != 20 || g.Turn != 0 {
		t.Fatalf("full inventory lost or mutated equipment: response=%+v equipped=%+v count=%d turn=%d", response, g.equippedWeapon, g.Weapons, g.Turn)
	}
}

func TestRangedKillAwardsEnemySpecificExperience(t *testing.T) {
	g := New(nil)
	g.enemies = []enemy{{Pos: Pos{3, 2}, Kind: "bat", HP: 1, Attack: 1}}
	g.HandleJSON([]byte(`{"version":1,"request_id":"shoot","command":{"type":"shoot","dx":1,"dy":0}}`))
	if g.Defeated != 1 || g.XP != 8 {
		t.Fatalf("bat ranged kill should award its configured XP: defeated=%d xp=%d", g.Defeated, g.XP)
	}
}

func TestWaitHealsBeforeEnemyTurns(t *testing.T) {
	g := New(nil)
	g.enemies = nil
	g.HP = 10
	r := handle(t, g, `{"type":"wait"}`)
	if r.HP != 11 || r.Turn != 1 || r.Hunger != 99 {
		t.Fatalf("wait transition mismatch: hp=%d turn=%d hunger=%d", r.HP, r.Turn, r.Hunger)
	}
}

func TestInventoryStackConsumesAndDropsOneItem(t *testing.T) {
	g := New(nil)
	g.enemies = nil
	g.player = Pos{3, 2}
	handle(t, g, `{"type":"pickup"}`)
	g.items = append(g.items, item{Pos: g.player, Kind: "food", Identified: true})
	handle(t, g, `{"type":"pickup"}`)
	if g.Food != 2 || len(g.inventory) != 1 || g.inventory[0].Qty != 2 {
		t.Fatalf("food did not stack: food=%d stacks=%+v", g.Food, g.inventory)
	}
	handle(t, g, `{"type":"drop","item":"food"}`)
	if g.Food != 1 || len(g.inventory) != 1 || g.inventory[0].Qty != 1 {
		t.Fatalf("drop did not remove exactly one: food=%d stacks=%+v", g.Food, g.inventory)
	}
}

func TestCursedWeaponCanBeIdentifiedButNotRemoved(t *testing.T) {
	g := New(nil)
	g.enemies = nil
	g.player = Pos{9, 2}
	handle(t, g, `{"type":"pickup"}`)
	identified := handle(t, g, `{"type":"identify","item":"weapon"}`)
	if identified.Turn != 1 || !strings.Contains(identified.Message, "cursed") {
		t.Fatalf("identification should be free and report curse: %+v", identified)
	}
	equipped := handle(t, g, `{"type":"equip"}`)
	if equipped.Turn != 2 || g.WeaponBonus != -1 {
		t.Fatalf("cursed weapon did not equip: %+v bonus=%d", equipped, g.WeaponBonus)
	}
	removed := handle(t, g, `{"type":"unequip"}`)
	if removed.Type != "error" || g.Turn != 2 {
		t.Fatalf("cursed weapon removal should fail without a turn: %+v", removed)
	}
}

func TestStrengthPotionExpiresAfterTenConsumedTurns(t *testing.T) {
	g := New(nil)
	g.enemies = nil
	g.StrengthPotions = 1
	r := handle(t, g, `{"type":"drink_strength"}`)
	if r.StrengthTurns != 9 || r.Turn != 1 || r.StrengthPotions != 0 {
		t.Fatalf("strength duration mismatch after drinking turn: %+v", r)
	}
	for range 9 {
		handle(t, g, `{"type":"wait"}`)
	}
	if g.StrengthTurns != 0 {
		t.Fatalf("strength effect did not expire: %d turns remain", g.StrengthTurns)
	}
}

func TestMappingScrollRevealsWholeFloor(t *testing.T) {
	g := New(nil)
	g.enemies = nil
	g.Scrolls = 1
	r := handle(t, g, `{"type":"read_scroll"}`)
	for i, known := range g.known {
		if !known {
			t.Fatalf("tile %d remains hidden", i)
		}
	}
	if r.Turn != 1 || r.Scrolls != 0 {
		t.Fatalf("mapping scroll transition mismatch: %+v", r)
	}
}

func TestTeleportAvoidsOccupiedCells(t *testing.T) {
	g := New(nil)
	g.enemies = nil
	g.items = nil
	g.TeleportScrolls = 1
	g.addStack(item{Kind: "teleport_scroll", Identified: true})
	origin := g.player
	r := handle(t, g, `{"type":"teleport"}`)
	if r.Turn != 1 || r.TeleportScrolls != 0 || g.player == origin || g.tiles[idx(g.player)] != floor {
		t.Fatalf("teleport failed to choose a different floor tile: %+v player=%+v", r, g.player)
	}
}

func TestPoisonTrapAppliesImmediateAndDelayedDamage(t *testing.T) {
	g := New(nil)
	g.enemies = nil
	g.player = Pos{5, 2}
	r := handle(t, g, `{"type":"move","dx":1,"dy":0}`)
	if r.HP != 19 || r.PoisonTurns != 3 {
		t.Fatalf("poison trap did not apply immediate damage and status: %+v", r)
	}
	r = handle(t, g, `{"type":"wait"}`)
	if r.HP != 19 || r.PoisonTurns != 2 {
		t.Fatalf("poison did not damage on the next consumed turn: %+v", r)
	}
}

func TestFireTrapDealsImmediateDamage(t *testing.T) {
	g := New(nil)
	g.enemies = nil
	g.player = Pos{4, 3}
	r := handle(t, g, `{"type":"move","dx":1,"dy":0}`)
	if r.HP != 16 || r.PoisonTurns != 0 {
		t.Fatalf("fire trap effect mismatch: %+v", r)
	}
}

func TestTrollActsEveryOtherTurnAndRegenerates(t *testing.T) {
	g := New(nil)
	g.HP = 10
	g.enemies = []enemy{{Pos: Pos{3, 2}, Kind: "troll", HP: 5, Attack: 3, Defense: 1}}
	handle(t, g, `{"type":"wait"}`)
	if g.HP != 11 || g.enemies[0].HP != 5 {
		t.Fatalf("troll acted on its skipped turn: hp=%d troll_hp=%d", g.HP, g.enemies[0].HP)
	}
	handle(t, g, `{"type":"wait"}`)
	if g.HP != 10 || g.enemies[0].HP != 6 {
		t.Fatalf("troll did not regenerate and attack on its action turn: hp=%d troll_hp=%d", g.HP, g.enemies[0].HP)
	}
}

func TestWoundedBatRetreats(t *testing.T) {
	g := New(nil)
	g.enemies = []enemy{{Pos: Pos{4, 2}, Kind: "bat", HP: 1, Attack: 1}}
	handle(t, g, `{"type":"wait"}`)
	if g.enemies[0].Pos != (Pos{5, 2}) {
		t.Fatalf("wounded bat did not retreat: %+v", g.enemies[0].Pos)
	}
}

func TestAmuletReturnEndsInVictory(t *testing.T) {
	g := New(nil)
	g.enemies = nil
	g.player = g.stair
	handle(t, g, `{"type":"wait"}`)
	if g.FloorNo != 2 {
		t.Fatalf("did not descend: floor=%d", g.FloorNo)
	}
	g.player = g.stair
	handle(t, g, `{"type":"pickup"}`)
	if g.Amulets != 1 {
		t.Fatal("did not pick up the amulet")
	}
	g.player = g.upStair
	r := handle(t, g, `{"type":"wait"}`)
	if r.Floor != 1 || !r.Ended || r.Result == nil || *r.Result != "victory" || r.Score == nil {
		t.Fatalf("returning with the amulet should win: %+v", r)
	}
}

func TestSeedZeroIsReportedWithoutChangingRNGFallback(t *testing.T) {
	seed := uint64(0)
	r := decodeSnapshot(t, New(&seed).InitialJSON())
	if r.Seed != 0 {
		t.Fatalf("snapshot must report supplied seed 0, got %d", r.Seed)
	}
}

func TestUnsupportedCommandIsRejectedWithoutTurn(t *testing.T) {
	g := New(nil)
	r := handle(t, g, `{"type":"fly"}`)
	if r.Type != "error" || r.Error == nil || r.Error.Code != "invalid_json" || g.Turn != 0 {
		t.Fatalf("unsupported command should be rejected without changing state: %+v", r)
	}
}

func TestFullInventoryLeavesGroundItemUntouched(t *testing.T) {
	g := New(nil)
	g.enemies = nil
	g.player = Pos{3, 2}
	g.Food = 20
	r := handle(t, g, `{"type":"pickup"}`)
	if r.Type != "error" || r.Error == nil || r.Error.Code != "inventory_full" || len(g.items) == 0 || g.items[0].Pos != g.player || g.Turn != 0 {
		t.Fatalf("full inventory should preserve item and turn: %+v", r)
	}
}

func TestCursedArmorKeepsNegativeDefenseAndCannotBeRemoved(t *testing.T) {
	g := New(nil)
	g.enemies = nil
	g.player = Pos{10, 2}
	handle(t, g, `{"type":"pickup"}`)
	handle(t, g, `{"type":"identify","item":"armor"}`)
	handle(t, g, `{"type":"equip_armor"}`)
	r := handle(t, g, `{"type":"unequip_armor"}`)
	if g.ArmorBonus != -1 || r.Type != "error" || g.Turn != 2 {
		t.Fatalf("cursed armor must retain negative defense and resist removal: bonus=%d response=%+v", g.ArmorBonus, r)
	}
}

func TestEnemiesDoNotActThroughBlockingWall(t *testing.T) {
	g := New(nil)
	g.HP = 10
	g.tiles[idx(Pos{3, 2})] = wall
	g.enemies = []enemy{{Pos: Pos{4, 2}, Kind: "goblin", HP: 5, Attack: 2}}
	r := handle(t, g, `{"type":"wait"}`)
	if r.HP != 11 || g.enemies[0].Pos != (Pos{4, 2}) {
		t.Fatalf("enemy crossed or attacked through a wall: hp=%d enemy=%+v", r.HP, g.enemies[0])
	}
}

func TestFloorWorldStateSurvivesRoundTrip(t *testing.T) {
	g := New(nil)
	g.enemies = nil
	g.player = g.stair
	handle(t, g, `{"type":"wait"}`)
	g.items = nil
	g.player = g.upStair
	handle(t, g, `{"type":"wait"}`)
	if g.FloorNo != 1 {
		t.Fatalf("did not ascend: floor=%d", g.FloorNo)
	}
	g.player = g.stair
	handle(t, g, `{"type":"wait"}`)
	if g.FloorNo != 2 || len(g.items) != 0 {
		t.Fatalf("floor 2 state was not restored: floor=%d items=%d", g.FloorNo, len(g.items))
	}
}
