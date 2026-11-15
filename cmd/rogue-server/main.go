package main

import (
	"log/slog"
	"net"
	"net/http"
	"os"
	"strconv"
	"time"

	"github.com/gorilla/websocket"
	"rogue-server/internal/game"
)

var upgrader = websocket.Upgrader{CheckOrigin: func(*http.Request) bool { return true }}

func main() {
	slog.SetDefault(slog.New(slog.NewJSONHandler(os.Stderr, nil)))
	mux := http.NewServeMux()
	mux.HandleFunc("GET /health", func(w http.ResponseWriter, _ *http.Request) { _, _ = w.Write([]byte("ok")) })
	mux.HandleFunc("GET /ws", websocketHandler)
	address := os.Getenv("ROGUE_LISTEN")
	if address == "" {
		address = "127.0.0.1:8080"
	}
	listener, err := net.Listen("tcp", address)
	if err != nil {
		slog.Error("server_start_failed", "operation", "server_start", "outcome", "failed", "address", address, "error", err)
		os.Exit(1)
	}
	slog.Info("server_started", "operation", "server_start", "outcome", "started", "address", address)
	if err := http.Serve(listener, mux); err != nil {
		slog.Error("server_stopped", "operation", "server_start", "outcome", "failed", "error", err)
		os.Exit(1)
	}
}

func websocketHandler(w http.ResponseWriter, r *http.Request) {
	var seed *uint64
	if raw := r.URL.Query().Get("seed"); raw != "" {
		parsed, err := strconv.ParseUint(raw, 10, 64)
		if err != nil {
			slog.Warn("websocket_rejected", "operation", "websocket_connect", "outcome", "invalid_seed", "error", err)
			http.Error(w, "invalid seed", http.StatusBadRequest)
			return
		}
		seed = &parsed
	}
	conn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		slog.Warn("websocket_upgrade_failed", "operation", "websocket_connect", "outcome", "failed", "error", err)
		return
	}
	startedAt := time.Now()
	outcome := "client_disconnected"
	defer func() {
		slog.Info("websocket_disconnected", "operation", "websocket_connection", "outcome", outcome, "duration_ms", time.Since(startedAt).Milliseconds())
		if err := conn.Close(); err != nil {
			slog.Warn("websocket_close_failed", "operation", "websocket_close", "outcome", "failed", "error", err)
		}
	}()
	slog.Info("websocket_connected", "operation", "websocket_connect", "outcome", "connected")
	g := game.New(seed)
	if err := conn.WriteMessage(websocket.TextMessage, g.InitialJSON()); err != nil {
		outcome = "transport_error"
		slog.Warn("websocket_initial_snapshot_failed", "operation", "websocket_write", "outcome", "failed", "error", err)
		return
	}
	for {
		messageType, message, err := conn.ReadMessage()
		if err != nil {
			if !websocket.IsCloseError(err, websocket.CloseNormalClosure, websocket.CloseGoingAway) {
				outcome = "transport_error"
				slog.Warn("websocket_read_failed", "operation", "websocket_read", "outcome", "failed", "error", err)
			}
			return
		}
		if messageType != websocket.TextMessage {
			continue
		}
		response := g.HandleJSON(message)
		if err := conn.WriteMessage(websocket.TextMessage, response); err != nil {
			outcome = "transport_error"
			slog.Warn("websocket_write_failed", "operation", "websocket_write", "outcome", "failed", "error", err)
			return
		}
		if g.Ended {
			if err := conn.WriteControl(websocket.CloseMessage, websocket.FormatCloseMessage(websocket.CloseNormalClosure, ""), time.Now().Add(time.Second)); err != nil {
				outcome = "transport_error"
				slog.Warn("websocket_close_frame_failed", "operation", "websocket_close", "outcome", "failed", "error", err)
				return
			}
			outcome = "game_ended"
			return
		}
	}
}
