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

use crate::state::AppState;

/// 账户WebSocket处理器
pub async fn account_websocket(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| handle_account_socket(socket, state))
}

async fn handle_account_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    
    // TODO: 从JWT token中获取用户ID
    let user_id = Uuid::new_v4(); // 临时使用随机ID

    // 发送欢迎消息
    let welcome_msg = json!({
        "type": "welcome",
        "message": "Connected to account stream",
        "timestamp": chrono::Utc::now()
    });

    if sender.send(Message::Text(welcome_msg.to_string())).await.is_err() {
        return;
    }

    // 创建定时器发送账户更新
    let mut update_interval = interval(Duration::from_secs(10));

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
            
            // 定期发送账户更新
            _ = update_interval.tick() => {
                if let Err(e) = send_account_update(&state, user_id, &mut sender).await {
                    tracing::error!("Error sending account update: {}", e);
                    break;
                }
            }
        }
    }

    tracing::info!("Account WebSocket connection closed");
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
                "message": "Subscribed to account updates",
                "timestamp": chrono::Utc::now()
            });
            sender.send(Message::Text(response.to_string())).await?;
        }
        Some("unsubscribe") => {
            let response = json!({
                "type": "unsubscribed",
                "message": "Unsubscribed from account updates",
                "timestamp": chrono::Utc::now()
            });
            sender.send(Message::Text(response.to_string())).await?;
        }
        Some("get_account") => {
            match state.account_service.get_account(user_id, None).await {
                Ok(account) => {
                    let response = json!({
                        "type": "account",
                        "data": account,
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
        Some("get_balance") => {
            match state.account_service.get_balance(user_id, None).await {
                Ok(balance) => {
                    let response = json!({
                        "type": "balance",
                        "data": balance,
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

async fn send_account_update(
    state: &AppState,
    user_id: Uuid,
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 发送账户余额更新
    match state.account_service.get_balance(user_id, None).await {
        Ok(balance) => {
            let update = json!({
                "type": "account_update",
                "data": {
                    "balance": balance,
                    "timestamp": chrono::Utc::now()
                }
            });
            sender.send(Message::Text(update.to_string())).await?;
        }
        Err(e) => {
            tracing::error!("Failed to get account balance for WebSocket update: {}", e);
        }
    }

    Ok(())
}