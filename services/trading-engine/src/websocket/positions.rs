use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde_json::json;
use tokio::time::{interval, Duration};
use uuid::Uuid;

use crate::{state::AppState, models::PositionSummary};

/// 仓位WebSocket处理器
pub async fn positions_websocket(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| handle_positions_socket(socket, state))
}

async fn handle_positions_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    
    // TODO: 从JWT token中获取用户ID
    let user_id = Uuid::new_v4(); // 临时使用随机ID

    // 发送欢迎消息
    let welcome_msg = json!({
        "type": "welcome",
        "message": "Connected to positions stream",
        "timestamp": chrono::Utc::now()
    });

    if sender.send(Message::Text(welcome_msg.to_string())).await.is_err() {
        return;
    }

    // 创建定时器发送仓位更新
    let mut update_interval = interval(Duration::from_secs(3));

    loop {
        tokio::select! {
            // 处理客户端消息
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Err(e) = handle_client_message(&text, &state, user_id, &mut sender).await {
                            tracing::error!("Error handling client message: {}", e);
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        tracing::info!("WebSocket connection closed by client");
                        break;
                    }
                    Some(Err(e)) => {
                        tracing::error!("WebSocket error: {}", e);
                        break;
                    }
                    None => break,
                    _ => {}
                }
            }
            
            // 定期发送仓位更新
            _ = update_interval.tick() => {
                if let Err(e) = send_positions_update(&state, user_id, &mut sender).await {
                    tracing::error!("Error sending positions update: {}", e);
                    break;
                }
            }
        }
    }

    tracing::info!("Positions WebSocket connection closed");
}

async fn handle_client_message(
    text: &str,
    state: &AppState,
    user_id: Uuid,
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let request: serde_json::Value = serde_json::from_str(text)?;
    
    match request.get("type").and_then(|t| t.as_str()) {
        Some("subscribe") => {
            let response = json!({
                "type": "subscribed",
                "message": "Subscribed to positions updates",
                "timestamp": chrono::Utc::now()
            });
            sender.send(Message::Text(response.to_string())).await?;
        }
        Some("unsubscribe") => {
            let response = json!({
                "type": "unsubscribed",
                "message": "Unsubscribed from positions updates",
                "timestamp": chrono::Utc::now()
            });
            sender.send(Message::Text(response.to_string())).await?;
        }
        Some("get_positions") => {
            match state.position_service.list_positions(user_id, None, None).await {
                Ok(positions) => {
                    let summaries: Vec<PositionSummary> = positions.iter().map(|p| p.into()).collect();
                    let response = json!({
                        "type": "positions",
                        "data": summaries,
                        "timestamp": chrono::Utc::now()
                    });
                    sender.send(Message::Text(response.to_string())).await?;
                }
                Err(e) => {
                    let error_response = json!({
                        "type": "error",
                        "message": e.to_string(),
                        "timestamp": chrono::Utc::now()
                    });
                    sender.send(Message::Text(error_response.to_string())).await?;
                }
            }
        }
        _ => {
            let error_response = json!({
                "type": "error",
                "message": "Unknown message type",
                "timestamp": chrono::Utc::now()
            });
            sender.send(Message::Text(error_response.to_string())).await?;
        }
    }

    Ok(())
}

async fn send_positions_update(
    state: &AppState,
    user_id: Uuid,
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match state.position_service.list_positions(user_id, Some("OPEN".to_string()), None).await {
        Ok(positions) => {
            let summaries: Vec<PositionSummary> = positions.iter().map(|p| p.into()).collect();
            let update = json!({
                "type": "positions_update",
                "data": summaries,
                "timestamp": chrono::Utc::now()
            });
            sender.send(Message::Text(update.to_string())).await?;
        }
        Err(e) => {
            tracing::error!("Failed to get positions for WebSocket update: {}", e);
        }
    }

    Ok(())
}