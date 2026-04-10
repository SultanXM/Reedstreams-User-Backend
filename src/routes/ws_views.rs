use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use sqlx::PgPool;

use crate::ws_state::ActiveViewers;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub viewers: ActiveViewers,
}

pub fn ws_views_routes(state: AppState) -> Router {
    Router::new()
        .route("/ws/views/:match_id", get(viewer_ws_handler))
        .with_state(state)
}

async fn viewer_ws_handler(
    ws: WebSocketUpgrade,
    Path(match_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, match_id, state))
}

async fn handle_ws(socket: WebSocket, match_id: String, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Increment viewer count on connect
    let initial_count = state.viewers.increment(&match_id);

    // Send initial count to this client
    let msg = serde_json::json!({
        "type": "view_count",
        "match_id": match_id,
        "count": initial_count,
    });
    if sender
        .send(Message::Text(msg.to_string()))
        .await
        .is_err()
    {
        // Connection already dead
        state.viewers.decrement(&match_id);
        return;
    }

    // Subscribe to future count updates for this match
    let (_current, mut rx) = state.viewers.subscribe(&match_id);
    let match_id_clone = match_id.clone();
    let viewers_clone = state.viewers.clone();

    // Task: forward broadcast updates to this WebSocket client
    let send_task = tokio::spawn(async move {
        while let Ok(count) = rx.recv().await {
            let msg = serde_json::json!({
                "type": "view_count",
                "match_id": match_id_clone,
                "count": count,
            });
            if sender.send(Message::Text(msg.to_string())).await.is_err() {
                break;
            }
        }
    });

    // Task: handle incoming messages from client (we just keep the connection alive)
    let recv_task = tokio::spawn(async move {
        while let Some(_msg) = receiver.next().await {}
    });

    // Wait for either task to finish (client disconnect or send error)
    tokio::select! {
        _ = send_task => {}
        _ = recv_task => {}
    }

    // Decrement viewer count on disconnect
    viewers_clone.decrement(&match_id);
}
