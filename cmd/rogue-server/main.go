package main

import (
	"context"
	"github.com/gorilla/websocket"
	"log/slog"
	"net"
	"net/http"
	"os"
	"os/signal"
	"rogue-server/internal/game"
	"strconv"
	"sync"
	"syscall"
	"time"
)

const defaultMaxSessions = 64
const defaultIdleTimeout = 5 * time.Minute

var sessionSlots = make(chan struct{}, defaultMaxSessions)
var activeSessions = struct {
	sync.Mutex
	connections map[*websocket.Conn]struct{}
}{connections: make(map[*websocket.Conn]struct{})}
var idleTimeout = defaultIdleTimeout

var upgrader = websocket.Upgrader{CheckOrigin: func(*http.Request) bool { return true }}

func main() {
	slog.SetDefault(slog.New(slog.NewJSONHandler(os.Stderr, nil)))
	configureSessionLimits()
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
	server := &http.Server{Handler: mux}
	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer stop()
	serveErr := make(chan error, 1)
	go func() { serveErr <- server.Serve(listener) }()
	slog.Info("server_started", "operation", "server_start", "outcome", "started", "address", address)
	select {
	case err := <-serveErr:
		if err != nil && err != http.ErrServerClosed {
			slog.Error("server_stopped", "operation", "server_start", "outcome", "failed", "error", err)
			os.Exit(1)
		}
	case <-ctx.Done():
		shutdownCtx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		err := server.Shutdown(shutdownCtx)
		cancel()
		closeActiveSessions()
		if err != nil {
			slog.Error("server_shutdown_failed", "operation", "server_shutdown", "outcome", "failed", "error", err)
			os.Exit(1)
		}
		<-serveErr
	}
}

func closeActiveSessions() {
	activeSessions.Lock()
	defer activeSessions.Unlock()
	for conn := range activeSessions.connections {
		_ = conn.Close()
	}
}

func configureSessionLimits() {
	if raw := os.Getenv("ROGUE_MAX_SESSIONS"); raw != "" {
		if n, err := strconv.Atoi(raw); err == nil && n > 0 {
			sessionSlots = make(chan struct{}, n)
		} else {
			slog.Warn("invalid_session_limit", "value", raw)
		}
	}
	if raw := os.Getenv("ROGUE_IDLE_TIMEOUT"); raw != "" {
		if d, err := time.ParseDuration(raw); err == nil && d > 0 {
			idleTimeout = d
		} else {
			slog.Warn("invalid_idle_timeout", "value", raw)
		}
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
	select {
	case sessionSlots <- struct{}{}:
	default:
		http.Error(w, "server is at session capacity", http.StatusServiceUnavailable)
		return
	}
	defer func() { <-sessionSlots }()

	conn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		slog.Warn("websocket_upgrade_failed", "operation", "websocket_connect", "outcome", "failed", "error", err)
		return
	}
	activeSessions.Lock()
	activeSessions.connections[conn] = struct{}{}
	activeSessions.Unlock()
	defer func() {
		activeSessions.Lock()
		delete(activeSessions.connections, conn)
		activeSessions.Unlock()
	}()
	startedAt := time.Now()
	outcome := "client_disconnected"
	if err := conn.SetReadDeadline(time.Now().Add(idleTimeout)); err != nil {
		outcome = "transport_error"
		_ = conn.Close()
		return
	}
	conn.SetPingHandler(func(data string) error {
		if err := conn.WriteControl(websocket.PongMessage, []byte(data), time.Now().Add(time.Second)); err != nil {
			return err
		}
		return conn.SetReadDeadline(time.Now().Add(idleTimeout))
	})
	conn.SetPongHandler(func(string) error {
		return conn.SetReadDeadline(time.Now().Add(idleTimeout))
	})
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
		if err := conn.SetReadDeadline(time.Now().Add(idleTimeout)); err != nil {
			outcome = "transport_error"
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
