use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    auth::verify_token,
    models::SendMessageRequest,
};

#[derive(Debug, Deserialize)]
pub struct GetMessagesQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct ChatMessageWithUserData {
    pub id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub content: String,
    pub created_at: chrono::NaiveDateTime,
    pub memes: Option<Vec<String>>,
    pub profile_pic_url: Option<String>,
    pub name_color: Option<String>,
    pub name_glow: Option<i32>,
    pub badge: Option<String>,
}

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/chat/messages", get(get_messages))
        .route("/chat/send", post(send_message))
        .with_state(pool)
}

fn get_user_id_from_token(headers: &HeaderMap) -> Result<Uuid, (StatusCode, String)> {
    let auth_header = headers
        .get("authorization")
        .ok_or((StatusCode::UNAUTHORIZED, "Missing auth header".to_string()))?
        .to_str()
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid auth header".to_string()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid token format".to_string()))?;

    let claims = verify_token(token).map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".to_string()))?;

    Uuid::parse_str(&claims.sub).map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid user ID".to_string()))
}

async fn get_messages(
    State(pool): State<PgPool>,
    Query(query): Query<GetMessagesQuery>,
) -> Result<Json<Vec<ChatMessageWithUserData>>, (StatusCode, String)> {
    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0);

    let messages: Vec<ChatMessageWithUserData> = sqlx::query_as(
        r#"
        SELECT 
            m.id, 
            m.user_id, 
            m.username, 
            m.content, 
            m.created_at,
            p.memes,
            p.profile_pic_url,
            p.name_color,
            p.name_glow,
            p.badge
        FROM chat_messages m
        LEFT JOIN profiles p ON m.user_id = p.user_id
        ORDER BY m.created_at DESC 
        LIMIT $1 OFFSET $2
        "#
    )
    .bind(&limit)
    .bind(&offset)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(messages))
}

async fn send_message(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Json(req): Json<SendMessageRequest>,
) -> Result<(StatusCode, Json<ChatMessageWithUserData>), (StatusCode, String)> {
    let user_id = get_user_id_from_token(&headers)?;

    
    let user: (String,) = sqlx::query_as(
        "SELECT username FROM users WHERE id = $1"
    )
    .bind(&user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let message: ChatMessageWithUserData = sqlx::query_as(
        r#"
        INSERT INTO chat_messages (user_id, username, content) 
        VALUES ($1, $2, $3)
        RETURNING 
            id, 
            user_id, 
            username, 
            content, 
            created_at,
            (SELECT memes FROM profiles WHERE user_id = $1) as memes,
            (SELECT profile_pic_url FROM profiles WHERE user_id = $1) as profile_pic_url,
            (SELECT name_color FROM profiles WHERE user_id = $1) as name_color,
            (SELECT name_glow FROM profiles WHERE user_id = $1) as name_glow,
            (SELECT badge FROM profiles WHERE user_id = $1) as badge
        "#
    )
    .bind(&user_id)
    .bind(&user.0)
    .bind(&req.content)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(message)))
}
