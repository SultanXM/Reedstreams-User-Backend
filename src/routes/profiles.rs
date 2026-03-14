use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{get, put},
    Json, Router,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    auth::verify_token,
    models::{Profile, UpdateProfileRequest, UploadProfilePicRequest},
};

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/profile", get(get_my_profile))
        .route("/profile", put(update_profile))
        .route("/profile/:user_id", get(get_profile))
        .route("/profile/upload-pic", put(upload_profile_pic))
        .route("/profile/delete-pic", put(delete_profile_pic))
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

async fn get_my_profile(
    State(pool): State<PgPool>,
    headers: HeaderMap,
) -> Result<Json<Profile>, (StatusCode, String)> {
    let user_id = get_user_id_from_token(&headers)?;

    let profile: Profile = sqlx::query_as(
        "SELECT * FROM profiles WHERE user_id = $1"
    )
    .bind(&user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    Ok(Json(profile))
}

async fn get_profile(
    State(pool): State<PgPool>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Profile>, (StatusCode, String)> {
    let profile: Profile = sqlx::query_as(
        "SELECT * FROM profiles WHERE user_id = $1"
    )
    .bind(&user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    Ok(Json(profile))
}

async fn update_profile(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<Json<Profile>, (StatusCode, String)> {
    let user_id = get_user_id_from_token(&headers)?;

    sqlx::query(
        r#"
        UPDATE profiles 
        SET 
            theme = COALESCE($1, theme),
            avatar_url = COALESCE($2, avatar_url),
            description = COALESCE($3, description),
            name_color = COALESCE($4, name_color),
            name_glow = COALESCE($5, name_glow),
            updated_at = NOW()
        WHERE user_id = $6
        "#
    )
    .bind(&req.theme)
    .bind(&req.avatar_url)
    .bind(&req.description)
    .bind(&req.name_color)
    .bind(&req.name_glow)
    .bind(&user_id)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let profile: Profile = sqlx::query_as(
        "SELECT * FROM profiles WHERE user_id = $1"
    )
    .bind(&user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(profile))
}

async fn upload_profile_pic(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Json(req): Json<UploadProfilePicRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id = get_user_id_from_token(&headers)?;

    
    if !req.image_data.starts_with("data:image/") {
        return Err((StatusCode::BAD_REQUEST, "Invalid image format. Must be base64 data URL".to_string()));
    }

    
    let size_estimate = req.image_data.len() * 3 / 4;
    if size_estimate > 2 * 1024 * 1024 {
        return Err((StatusCode::BAD_REQUEST, "Image too large. Max 2MB".to_string()));
    }

    
    sqlx::query(
        "UPDATE profiles SET profile_pic_url = $1, updated_at = NOW() WHERE user_id = $2"
    )
    .bind(&req.image_data)
    .bind(&user_id)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "message": "Profile picture updated",
        "profile_pic_url": req.image_data
    })))
}

async fn delete_profile_pic(
    State(pool): State<PgPool>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id = get_user_id_from_token(&headers)?;

    sqlx::query(
        "UPDATE profiles SET profile_pic_url = NULL, updated_at = NOW() WHERE user_id = $1"
    )
    .bind(&user_id)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({"message": "Profile picture deleted"})))
}
