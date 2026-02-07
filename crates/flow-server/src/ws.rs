use crate::state::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::warn;

/// GET /api/ws — WebSocket endpoint for bidirectional communication
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_ws(socket, state))
}

async fn handle_ws(socket: WebSocket, state: Arc<AppState>) {
    let mut rx = state.tx.subscribe();
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Send initial connected message
    let connected = serde_json::json!({"type": "connected"});
    if ws_sender
        .send(Message::Text(connected.to_string()))
        .await
        .is_err()
    {
        return;
    }

    // Reader task: handle incoming messages from the client
    let state_clone = state.clone();
    let reader = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_receiver.next().await {
            match msg {
                Message::Text(text) => {
                    // Parse incoming commands from agents
                    if let Ok(cmd) = serde_json::from_str::<serde_json::Value>(&text) {
                        let cmd_type = cmd.get("type").and_then(|v| v.as_str()).unwrap_or("");
                        match cmd_type {
                            "agent-status" => {
                                // Broadcast agent status update to all clients
                                let payload = serde_json::json!({
                                    "type": "agent-status",
                                    "agent": cmd.get("agent"),
                                    "status": cmd.get("status"),
                                    "task": cmd.get("task"),
                                });
                                let _ = state_clone.tx.send(payload.to_string());
                            }
                            "ping" => {
                                // Client keepalive, no action needed
                            }
                            _ => {
                                warn!("Unknown WS command: {cmd_type}");
                            }
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    // Writer task: forward broadcast events to the WebSocket client
    let writer = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(data) => {
                    if ws_sender.send(Message::Text(data)).await.is_err() {
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    warn!("WS client lagged by {n} messages");
                }
                Err(_) => break,
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = reader => {},
        _ = writer => {},
    }
}
