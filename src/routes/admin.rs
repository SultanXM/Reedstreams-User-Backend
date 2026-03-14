use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::{Duration, Utc};
use serde::Serialize;
use sqlx::FromRow;
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{AdminUpdateUserRequest, TimeoutUserRequest};

#[derive(Debug, FromRow, Serialize)]
pub struct UserWithProfile {
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub is_admin: bool,
    pub timeout_until: Option<chrono::NaiveDateTime>,
    pub tags: Option<Vec<String>>,
    pub memes: Option<Vec<String>>,
    pub name_color: Option<String>,
    pub name_glow: Option<i32>,
    pub profile_pic_url: Option<String>,
    pub badge: Option<String>,
}

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/admin/users", get(list_users))
        .route("/admin/users/:user_id/tags", put(update_user_tags))
        .route("/admin/users/:user_id/profile", put(update_user_profile))
        .route("/admin/users/:user_id/timeout", post(timeout_user))
        .route("/admin/users/:user_id/unban", post(unban_user))
        .route("/admin/messages/:id", delete(delete_message))
        .route("/admin/chat/clear", delete(clear_all_chat))
        .with_state(pool)
}

async fn list_users(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<UserWithProfile>>, (StatusCode, String)> {
    let users: Vec<UserWithProfile> = sqlx::query_as(
        r#"
        SELECT 
            u.id::text as user_id,
            u.username,
            u.email,
            u.is_admin,
            u.timeout_until,
            p.tags,
            p.memes,
            p.name_color,
            p.name_glow,
            p.profile_pic_url,
            p.badge
        FROM users u
        LEFT JOIN profiles p ON u.id = p.user_id
        ORDER BY u.created_at DESC
        "#
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(users))
}

async fn update_user_tags(
    State(pool): State<PgPool>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<AdminUpdateUserRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    
    let exists: (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM profiles WHERE user_id = $1)"
    )
    .bind(&user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if exists.0 {
        
        sqlx::query(
            r#"
            UPDATE profiles 
            SET 
                tags = COALESCE($1, tags),
                memes = COALESCE($2, memes),
                name_color = COALESCE($3, name_color),
                updated_at = NOW()
            WHERE user_id = $4
            "#
        )
        .bind(&req.tags)
        .bind(&req.memes)
        .bind(&req.name_color)
        .bind(&user_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    } else {
        
        sqlx::query(
            r#"
            INSERT INTO profiles (user_id, tags, memes, name_color)
            VALUES ($1, $2, $3, $4)
            "#
        )
        .bind(&user_id)
        .bind(&req.tags)
        .bind(&req.memes)
        .bind(&req.name_color)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    Ok(Json(serde_json::json!({"message": "User updated"})))
}

async fn timeout_user(
    State(pool): State<PgPool>,
    Json(req): Json<TimeoutUserRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id = Uuid::parse_str(&req.user_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid user ID".to_string()))?;

    let timeout_until = Utc::now() + Duration::minutes(req.minutes);

    sqlx::query(
        "UPDATE users SET timeout_until = $1 WHERE id = $2"
    )
    .bind(&timeout_until.naive_utc())
    .bind(&user_id)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "message": format!("User timed out for {} minutes", req.minutes),
        "timeout_until": timeout_until
    })))
}

async fn unban_user(
    State(pool): State<PgPool>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    sqlx::query(
        "UPDATE users SET timeout_until = NULL WHERE id = $1"
    )
    .bind(&user_id)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({"message": "User unbanned"})))
}

async fn delete_message(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = sqlx::query(
        "DELETE FROM chat_messages WHERE id = $1"
    )
    .bind(&id)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Message not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

async fn update_user_profile(
    State(pool): State<PgPool>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<AdminUpdateUserRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    
    let exists: (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM profiles WHERE user_id = $1)"
    )
    .bind(&user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if exists.0 {
        sqlx::query(
            r#"
            UPDATE profiles 
            SET 
                tags = COALESCE($1, tags),
                memes = COALESCE($2, memes),
                name_color = COALESCE($3, name_color),
                name_glow = COALESCE($4, name_glow),
                badge = COALESCE($5, badge),
                updated_at = NOW()
            WHERE user_id = $6
            "#
        )
        .bind(&req.tags)
        .bind(&req.memes)
        .bind(&req.name_color)
        .bind(&req.name_glow)
        .bind(&req.badge)
        .bind(&user_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    } else {
        sqlx::query(
            r#"
            INSERT INTO profiles (user_id, tags, memes, name_color, name_glow, badge)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#
        )
        .bind(&user_id)
        .bind(&req.tags)
        .bind(&req.memes)
        .bind(&req.name_color)
        .bind(&req.name_glow)
        .bind(&req.badge)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    Ok(Json(serde_json::json!({"message": "User updated"})))
}

async fn clear_all_chat(
    State(pool): State<PgPool>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    sqlx::query("DELETE FROM chat_messages")
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({"message": "All chat messages cleared"})))
}
