package main

import (
	"encoding/json"
	"net"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/gorilla/websocket"
)

func testWebSocket(t *testing.T) (*httptest.Server, *websocket.Conn) {
	return testWebSocketWithHandler(t, http.HandlerFunc(websocketHandler))
}

func testWebSocketWithHandler(t *testing.T, handler http.Handler) (*httptest.Server, *websocket.Conn) {
	t.Helper()
	server := httptest.NewServer(handler)
	t.Cleanup(server.Close)
	endpoint := "ws" + strings.TrimPrefix(server.URL, "http")
	conn, response, err := websocket.DefaultDialer.Dial(endpoint, nil)
	if err != nil {
		if response != nil {
			t.Fatalf("websocket dial failed: %v (HTTP %d)", err, response.StatusCode)
		}
		t.Fatal(err)
	}
	t.Cleanup(func() { _ = conn.Close() })
	if _, _, err := conn.ReadMessage(); err != nil {
		t.Fatalf("reading initial snapshot: %v", err)
	}
	return server, conn
}

func TestWebSocketPingAndBinaryFramesDoNotAbandonGame(t *testing.T) {
	_, conn := testWebSocket(t)
	if err := conn.WriteMessage(websocket.BinaryMessage, []byte("ignored")); err != nil {
		t.Fatal(err)
	}
	if err := conn.WriteControl(websocket.PingMessage, []byte("probe"), time.Now().Add(time.Second)); err != nil {
		t.Fatal(err)
	}
	if err := conn.WriteJSON(map[string]any{"version": 1, "request_id": "wait", "command": map[string]any{"type": "wait"}}); err != nil {
		t.Fatal(err)
	}
	_, data, err := conn.ReadMessage()
	if err != nil {
		t.Fatalf("session did not survive ping/binary frames: %v", err)
	}
	var response struct {
		Type string `json:"type"`
		Turn uint64 `json:"turn"`
	}
	if err := json.Unmarshal(data, &response); err != nil || response.Type != "snapshot" || response.Turn != 1 {
		t.Fatalf("unexpected command response: %s (error %v)", data, err)
	}
}

func TestWebSocketIdleTimeoutReleasesSession(t *testing.T) {
	previous := idleTimeout
	idleTimeout = 40 * time.Millisecond
	t.Cleanup(func() { idleTimeout = previous })
	_, conn := testWebSocket(t)
	_ = conn.SetReadDeadline(time.Now().Add(time.Second))
	if _, _, err := conn.ReadMessage(); err == nil {
		t.Fatal("idle connection remained open")
	}
	waitForNoSessions(t)
}

func TestWebSocketCapacityRejectsBeforeUpgrade(t *testing.T) {
	previous := sessionSlots
	sessionSlots = make(chan struct{}, 1)
	sessionSlots <- struct{}{}
	t.Cleanup(func() { sessionSlots = previous })
	server := httptest.NewServer(http.HandlerFunc(websocketHandler))
	defer server.Close()
	endpoint := "ws" + strings.TrimPrefix(server.URL, "http")
	conn, response, err := websocket.DefaultDialer.Dial(endpoint, nil)
	if conn != nil {
		_ = conn.Close()
		t.Fatal("over-capacity connection was upgraded")
	}
	if err == nil || response == nil || response.StatusCode != http.StatusServiceUnavailable {
		t.Fatalf("over-capacity response = %v, error %v; want HTTP 503", response, err)
	}
}

func TestWebSocketShutdownClosesActiveSession(t *testing.T) {
	_, conn := testWebSocket(t)
	closeActiveSessions()
	_ = conn.SetReadDeadline(time.Now().Add(time.Second))
	if _, _, err := conn.ReadMessage(); err == nil {
		t.Fatal("active connection remained open after shutdown")
	}
	waitForNoSessions(t)
}

func waitForNoSessions(t *testing.T) {
	waitForNoSessionsWithin(t, time.Second)
}

func waitForNoSessionsWithin(t *testing.T, timeout time.Duration) {
	t.Helper()
	deadline := time.Now().Add(timeout)
	for len(sessionSlots) != 0 && time.Now().Before(deadline) {
		time.Sleep(time.Millisecond)
	}
	if len(sessionSlots) != 0 {
		t.Fatal("session slot was not released")
	}
}

func TestWebSocketGracefulCloseReleasesSession(t *testing.T) {
	_, conn := testWebSocket(t)
	if err := conn.WriteControl(websocket.CloseMessage, websocket.FormatCloseMessage(websocket.CloseNormalClosure, ""), time.Now().Add(time.Second)); err != nil {
		t.Fatal(err)
	}
	waitForNoSessions(t)
}

func TestWebSocketDisconnectDuringResponseReleasesSession(t *testing.T) {
	_, conn := testWebSocket(t)
	if err := conn.WriteJSON(map[string]any{"version": 1, "request_id": "wait", "command": map[string]any{"type": "wait"}}); err != nil {
		t.Fatal(err)
	}
	if err := conn.UnderlyingConn().Close(); err != nil {
		t.Fatal(err)
	}
	waitForNoSessions(t)
}

func TestSlowWebSocketReaderCannotHoldSessionPastWriteDeadline(t *testing.T) {
	handler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		websocketHandlerWithWriteTimeout(w, r, 100*time.Millisecond)
	})
	_, conn := testWebSocketWithHandler(t, handler)
	tcp, ok := conn.UnderlyingConn().(*net.TCPConn)
	if !ok {
		t.Fatal("WebSocket transport is not TCP")
	}
	if err := tcp.SetReadBuffer(1024); err != nil {
		t.Fatal(err)
	}
	sendDone := make(chan struct{})
	go func() {
		defer close(sendDone)
		for range 10000 {
			if err := conn.WriteJSON(map[string]any{"version": 1, "request_id": "wait", "command": map[string]any{"type": "wait"}}); err != nil {
				return
			}
		}
	}()
	waitForNoSessionsWithin(t, 5*time.Second)
	_ = conn.Close()
	select {
	case <-sendDone:
	case <-time.After(time.Second):
		t.Fatal("client writer did not stop after the connection was closed")
	}
}
