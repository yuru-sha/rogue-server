package main

import (
	"bufio"
	"io"
	"net/http"
	"net/http/httptest"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/gorilla/websocket"
)

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

func TestReadCommandRetriesUnknownInputBeforeReturningValidCommand(t *testing.T) {
	var output strings.Builder
	command, err := readCommand(bufio.NewReader(strings.NewReader("?\n.\n")), &output)
	if err != nil {
		t.Fatal(err)
	}
	if command["type"] != "wait" || !strings.Contains(output.String(), "Unknown command") {
		t.Fatalf("input retry failed: command=%v output=%q", command, output.String())
	}
}

func TestReadCommandReturnsEOFWithoutSendingCommand(t *testing.T) {
	command, err := readCommand(bufio.NewReader(strings.NewReader("")), &strings.Builder{})
	if err == nil || command != nil {
		t.Fatalf("EOF should end input cleanly: command=%v err=%v", command, err)
	}
}

func TestRunRetriesInputAndHandlesWebSocketControls(t *testing.T) {
	requests := make(chan map[string]any, 1)
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		conn, err := (&websocket.Upgrader{}).Upgrade(w, r, nil)
		if err != nil {
			return
		}
		defer conn.Close()
		if err := conn.WriteMessage(websocket.BinaryMessage, []byte("ignored")); err != nil {
			return
		}
		if err := conn.WriteControl(websocket.PingMessage, []byte("probe"), time.Now().Add(time.Second)); err != nil {
			return
		}
		if err := conn.WriteJSON(map[string]any{"type": "snapshot", "ended": false, "map": []string{}}); err != nil {
			return
		}
		var request map[string]any
		if err := conn.ReadJSON(&request); err != nil {
			return
		}
		requests <- request
		_ = conn.WriteJSON(map[string]any{"type": "snapshot", "ended": true})
	}))
	defer server.Close()
	endpoint := "ws" + strings.TrimPrefix(server.URL, "http")
	conn, _, err := websocket.DefaultDialer.Dial(endpoint, nil)
	if err != nil {
		t.Fatal(err)
	}
	defer conn.Close()
	var output strings.Builder
	if err := run(conn, bufio.NewReader(strings.NewReader("?\n.\n")), &output); err != nil {
		t.Fatal(err)
	}
	request := <-requests
	command := request["command"].(map[string]any)
	if command["type"] != "wait" || !strings.Contains(output.String(), "Unknown command") {
		t.Fatalf("CLI did not retry and send the valid command: request=%v output=%q", request, output.String())
	}
}

func TestRunHandlesPingAndCloseWhileStdinIsIdle(t *testing.T) {
	pongSeen := make(chan struct{})
	var pongOnce sync.Once
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		conn, err := (&websocket.Upgrader{}).Upgrade(w, r, nil)
		if err != nil {
			return
		}
		defer conn.Close()
		conn.SetPongHandler(func(string) error {
			pongOnce.Do(func() { close(pongSeen) })
			return nil
		})
		go func() {
			for {
				if _, _, err := conn.ReadMessage(); err != nil {
					return
				}
			}
		}()
		if err := conn.WriteJSON(map[string]any{"type": "snapshot", "ended": false, "map": []string{}}); err != nil {
			return
		}
		if err := conn.WriteControl(websocket.PingMessage, []byte("probe"), time.Now().Add(time.Second)); err != nil {
			return
		}
		select {
		case <-pongSeen:
			_ = conn.WriteControl(websocket.CloseMessage, websocket.FormatCloseMessage(websocket.CloseNormalClosure, ""), time.Now().Add(time.Second))
		case <-time.After(2 * time.Second):
		}
	}))
	defer server.Close()
	endpoint := "ws" + strings.TrimPrefix(server.URL, "http")
	conn, _, err := websocket.DefaultDialer.Dial(endpoint, nil)
	if err != nil {
		t.Fatal(err)
	}
	defer conn.Close()
	reader, writer := io.Pipe()
	defer writer.Close()
	var output strings.Builder
	done := make(chan error, 1)
	go func() { done <- run(conn, bufio.NewReader(reader), &output) }()
	select {
	case err := <-done:
		if err != nil {
			t.Fatalf("CLI failed to handle Close with idle stdin: %v", err)
		}
	case <-time.After(3 * time.Second):
		t.Fatal("CLI remained blocked on stdin instead of processing WebSocket Ping/Close")
	}
	_ = reader.Close()
	select {
	case <-pongSeen:
	default:
		t.Fatal("CLI did not answer Ping while stdin was idle")
	}
}
