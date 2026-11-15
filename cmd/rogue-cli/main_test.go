package main

import "testing"

func TestDropKeysSelectEverySupportedInventoryKind(t *testing.T) {
	cases := []struct {
		key  rune
		item string
	}{
		{'1', "food"}, {'2', "potion"}, {'3', "scroll"}, {'4', "amulet"}, {'5', "weapon"},
		{'6', "armor"}, {'7', "strength_potion"}, {'8', "teleport_scroll"}, {'9', "ring"}, {'0', "staff"},
	}
	for _, tc := range cases {
		got := commandForKey([]rune{tc.key})
		if got["type"] != "drop" || got["item"] != tc.item {
			t.Errorf("key %q produced %v; want drop %q", tc.key, got, tc.item)
		}
	}
}

func TestIdentifyAndShootKeysEncodeCommands(t *testing.T) {
	for key, item := range map[rune]string{'I': "weapon", 'A': "armor", 'G': "ring", 'D': "staff"} {
		got := commandForKey([]rune{key})
		if got["type"] != "identify" || got["item"] != item {
			t.Errorf("key %q produced %v; want identify %q", key, got, item)
		}
	}
	for key, direction := range map[rune][2]int{'f': {1, 0}, 'F': {-1, 0}, 'R': {0, -1}, 'B': {0, 1}} {
		got := commandForKey([]rune{key})
		if got["type"] != "shoot" || got["dx"] != direction[0] || got["dy"] != direction[1] {
			t.Errorf("key %q produced %v; want shoot %v", key, got, direction)
		}
	}
}

func TestMovementKeysPreserveEightDirections(t *testing.T) {
	cases := map[rune][2]int{
		'h': {-1, 0}, 'l': {1, 0}, 'k': {0, -1}, 'j': {0, 1},
		'y': {-1, -1}, 'u': {1, -1}, 'b': {-1, 1}, 'n': {1, 1},
	}
	for key, direction := range cases {
		got := commandForKey([]rune{key})
		if got["type"] != "move" || got["dx"] != direction[0] || got["dy"] != direction[1] {
			t.Errorf("key %q produced %v; want move %v", key, got, direction)
		}
	}
	if commandForKey(nil) != nil || commandForKey([]rune{'?'}) != nil {
		t.Fatal("unknown or empty input must not produce a request")
	}
}
