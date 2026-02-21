//! WebSocket handler for real-time streaming chat via gateway.
//!
//! Protocol:
//! → Client sends: {"type":"chat","content":"...","provider":"openai","stream":true}
//! ← Server sends: {"type":"chat_start","request_id":"..."}
//! ← Server sends: {"type":"chat_chunk","request_id":"...","content":"token","index":0}
//! ← Server sends: {"type":"chat_done","request_id":"...","total_tokens":42}
//! OR
//! ← Server sends: {"type":"chat_response","content":"full response"} (non-streaming)

use axum::{
    extract::{State, ws::{Message, WebSocket, WebSocketUpgrade}},
    response::IntoResponse,
};
use std::sync::Arc;
use super::server::AppState;

/// WebSocket upgrade handler.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handle a WebSocket connection.
async fn handle_socket(mut socket: WebSocket, _state: Arc<AppState>) {
    tracing::info!("WebSocket client connected");

    // Send welcome
    let welcome = serde_json::json!({
        "type": "connected",
        "message": "BizClaw Gateway — WebSocket connected",
        "version": env!("CARGO_PKG_VERSION"),
        "capabilities": ["chat", "stream", "ping"],
    });
    if send_json(&mut socket, &welcome).await.is_err() {
        return;
    }

    // Connection state
    let mut request_counter: u64 = 0;

    // Message loop
    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(Message::Text(text)) => {
                let json = match serde_json::from_str::<serde_json::Value>(&text) {
                    Ok(j) => j,
                    Err(e) => {
                        send_error(&mut socket, &format!("Invalid JSON: {e}")).await;
                        continue;
                    }
                };

                let msg_type = json["type"].as_str().unwrap_or("unknown");

                match msg_type {
                    "chat" => {
                        request_counter += 1;
                        let request_id = format!("req_{request_counter}");
                        let content = json["content"].as_str().unwrap_or("").to_string();
                        let stream = json["stream"].as_bool().unwrap_or(false);
                        let provider = json["provider"].as_str().unwrap_or("default").to_string();

                        tracing::debug!("Chat: provider={provider}, stream={stream}, len={}", content.len());

                        if stream {
                            // Streaming response — send token-by-token
                            let start = serde_json::json!({
                                "type": "chat_start",
                                "request_id": &request_id,
                                "provider": &provider,
                            });
                            if send_json(&mut socket, &start).await.is_err() { break; }

                            // Simulate streaming (TODO: wire to actual agent)
                            let words: Vec<&str> = content.split_whitespace().collect();
                            let response_text = if words.is_empty() {
                                "Hello! I'm BizClaw, your AI assistant.".to_string()
                            } else {
                                format!("Tôi đã nhận được tin nhắn với {} từ. Hệ thống đang hoạt động.", words.len())
                            };

                            let tokens: Vec<&str> = response_text.split_whitespace().collect();
                            for (i, token) in tokens.iter().enumerate() {
                                let chunk = serde_json::json!({
                                    "type": "chat_chunk",
                                    "request_id": &request_id,
                                    "content": format!("{token} "),
                                    "index": i,
                                });
                                if send_json(&mut socket, &chunk).await.is_err() { break; }

                                // Simulate typing delay
                                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                            }

                            let done = serde_json::json!({
                                "type": "chat_done",
                                "request_id": &request_id,
                                "total_tokens": tokens.len(),
                                "full_content": response_text,
                            });
                            if send_json(&mut socket, &done).await.is_err() { break; }

                        } else {
                            // Non-streaming response
                            let response = serde_json::json!({
                                "type": "chat_response",
                                "request_id": &request_id,
                                "content": format!("[gateway] Echo: {content}"),
                                "provider": &provider,
                            });
                            if send_json(&mut socket, &response).await.is_err() { break; }
                        }
                    }

                    "ping" => {
                        let pong = serde_json::json!({
                            "type": "pong",
                            "timestamp": chrono::Utc::now().timestamp_millis(),
                        });
                        let _ = send_json(&mut socket, &pong).await;
                    }

                    "status" => {
                        let status = serde_json::json!({
                            "type": "status",
                            "requests_processed": request_counter,
                            "uptime_secs": _state.start_time.elapsed().as_secs(),
                        });
                        let _ = send_json(&mut socket, &status).await;
                    }

                    _ => {
                        send_error(&mut socket, &format!("Unknown message type: {msg_type}")).await;
                    }
                }
            }
            Ok(Message::Ping(data)) => {
                let _ = socket.send(Message::Pong(data)).await;
            }
            Ok(Message::Close(_)) => {
                tracing::info!("WebSocket client disconnected (close frame)");
                break;
            }
            Err(e) => {
                tracing::error!("WebSocket error: {e}");
                break;
            }
            _ => {}
        }
    }

    tracing::info!("WebSocket connection closed (total requests: {request_counter})");
}

/// Send a JSON value as a WS text message.
async fn send_json(socket: &mut WebSocket, value: &serde_json::Value) -> Result<(), ()> {
    socket.send(Message::Text(value.to_string().into()))
        .await
        .map_err(|e| {
            tracing::error!("WS send failed: {e}");
        })
}

/// Send an error message.
async fn send_error(socket: &mut WebSocket, message: &str) {
    let error = serde_json::json!({
        "type": "error",
        "message": message,
    });
    let _ = send_json(socket, &error).await;
}
