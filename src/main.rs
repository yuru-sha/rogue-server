use axum::{
    Router,
    extract::{
        Query,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
    routing::get,
};
use rogue_server::{CommandRequest, Game, GameResponse};
use serde::Deserialize;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = app();
    let listener = tokio::net::TcpListener::bind(bind_address()).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

fn app() -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/ws", get(ws))
}

fn bind_address() -> String {
    std::env::var("ROGUE_LISTEN").unwrap_or_else(|_| "127.0.0.1:8080".into())
}
#[derive(Deserialize)]
struct SeedQuery {
    seed: Option<u64>,
}

async fn ws(upgrade: WebSocketUpgrade, Query(query): Query<SeedQuery>) -> impl IntoResponse {
    upgrade.on_upgrade(move |socket| game_session(socket, query.seed))
}

async fn game_session(mut socket: WebSocket, seed: Option<u64>) {
    let mut game = Game::new(seed);
    send(&mut socket, GameResponse::snapshot(&game)).await;
    while let Some(Ok(Message::Text(text))) = socket.recv().await {
        let response = match serde_json::from_str::<CommandRequest>(&text) {
            Ok(request) => game.handle(request),
            Err(error) => GameResponse::error("invalid_json", error.to_string()),
        };
        let ended = response.ended;
        send(&mut socket, response).await;
        if ended {
            let _ = socket.send(Message::Close(None)).await;
            break;
        }
    }
}

async fn send(socket: &mut WebSocket, response: GameResponse) {
    let Ok(text) = serde_json::to_string(&response) else {
        return;
    };
    let _ = socket.send(Message::Text(text.into())).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::{SinkExt, StreamExt};
    use serde_json::json;
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use tokio_tungstenite::{connect_async, tungstenite::Message};

    #[tokio::test]
    async fn websocket_sends_seeded_snapshot_and_handles_a_command() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, app()).await.unwrap();
        });

        let url = format!("ws://{address}/ws?seed=42");
        let (mut socket, _) = connect_async(url).await.unwrap();
        let initial = socket.next().await.unwrap().unwrap().into_text().unwrap();
        let initial: GameResponse = serde_json::from_str(&initial).unwrap();
        assert_eq!(initial.seed, 42);
        assert_eq!(initial.turn, 0);

        socket
            .send(Message::Text(
                json!({
                    "version": 1,
                    "request_id": "integration-1",
                    "command": {"type": "wait"}
                })
                .to_string()
                .into(),
            ))
            .await
            .unwrap();
        let response = socket.next().await.unwrap().unwrap().into_text().unwrap();
        let response: GameResponse = serde_json::from_str(&response).unwrap();
        assert_eq!(response.request_id, "integration-1");
        assert_eq!(response.turn, 1);

        server.abort();
    }

    #[tokio::test]
    async fn websocket_rejects_invalid_input_without_consuming_a_turn() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, app()).await.unwrap();
        });

        let url = format!("ws://{address}/ws");
        let (mut socket, _) = connect_async(url).await.unwrap();
        socket.next().await.unwrap().unwrap();
        socket
            .send(Message::Text("not-json".to_owned().into()))
            .await
            .unwrap();
        let response = socket.next().await.unwrap().unwrap().into_text().unwrap();
        let response: GameResponse = serde_json::from_str(&response).unwrap();
        assert_eq!(response.type_, "error");
        assert_eq!(response.error.as_ref().unwrap().code, "invalid_json");
        assert_eq!(response.turn, 0);

        socket
            .send(Message::Text(
                json!({
                    "version": 1,
                    "request_id": "after-error",
                    "command": {"type": "move", "dx": 0, "dy": 0}
                })
                .to_string()
                .into(),
            ))
            .await
            .unwrap();
        let response = socket.next().await.unwrap().unwrap().into_text().unwrap();
        let response: GameResponse = serde_json::from_str(&response).unwrap();
        assert_eq!(response.type_, "error");
        assert_eq!(response.request_id, "after-error");
        assert_eq!(response.turn, 0);

        server.abort();
    }

    #[tokio::test]
    async fn websocket_closes_after_quit() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, app()).await.unwrap();
        });

        let url = format!("ws://{address}/ws");
        let (mut socket, _) = connect_async(url).await.unwrap();
        socket.next().await.unwrap().unwrap();
        socket
            .send(Message::Text(
                json!({
                    "version": 1,
                    "request_id": "quit",
                    "command": {"type": "quit"}
                })
                .to_string()
                .into(),
            ))
            .await
            .unwrap();
        let response = socket.next().await.unwrap().unwrap().into_text().unwrap();
        let response: GameResponse = serde_json::from_str(&response).unwrap();
        assert!(response.ended);
        assert_eq!(response.result.as_deref(), Some("quit"));
        assert!(matches!(
            socket.next().await,
            Some(Ok(Message::Close(_))) | None
        ));

        server.abort();
    }

    #[tokio::test]
    async fn websocket_connections_have_independent_games() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, app()).await.unwrap();
        });

        let url = format!("ws://{address}/ws?seed=42");
        let (mut first, _) = connect_async(&url).await.unwrap();
        let (mut second, _) = connect_async(&url).await.unwrap();
        first.next().await.unwrap().unwrap();
        let second_initial = second.next().await.unwrap().unwrap().into_text().unwrap();
        let second_initial: GameResponse = serde_json::from_str(&second_initial).unwrap();
        first
            .send(Message::Text(
                json!({
                    "version": 1,
                    "request_id": "first",
                    "command": {"type": "wait"}
                })
                .to_string()
                .into(),
            ))
            .await
            .unwrap();
        let first_response = first.next().await.unwrap().unwrap().into_text().unwrap();
        let first_response: GameResponse = serde_json::from_str(&first_response).unwrap();
        assert_eq!(first_response.turn, 1);
        assert_eq!(second_initial.turn, 0);

        server.abort();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn health_endpoint_returns_ok() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, app()).await.unwrap();
        });

        let mut connection = TcpStream::connect(address).unwrap();
        connection
            .write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
            .unwrap();
        let mut body = String::new();
        connection.read_to_string(&mut body).unwrap();
        assert!(body.starts_with("HTTP/1.1 200 OK"));
        assert!(body.ends_with("ok"));

        server.abort();
    }
}
