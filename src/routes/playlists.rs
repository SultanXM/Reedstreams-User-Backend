use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{delete, get, post, put},
    Json, Router,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    auth::verify_token,
    models::{CreatePlaylistRequest, Playlist, UpdatePlaylistRequest},
};

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/playlists", get(get_my_playlists))
        .route("/playlists", post(create_playlist))
        .route("/playlists/:id", get(get_playlist))
        .route("/playlists/:id", put(update_playlist))
        .route("/playlists/:id", delete(delete_playlist))
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

async fn get_my_playlists(
    State(pool): State<PgPool>,
    headers: HeaderMap,
) -> Result<Json<Vec<Playlist>>, (StatusCode, String)> {
    let user_id = get_user_id_from_token(&headers)?;

    let playlists: Vec<Playlist> = sqlx::query_as(
        "SELECT * FROM playlists WHERE user_id = $1 ORDER BY created_at DESC"
    )
    .bind(&user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(playlists))
}

async fn get_playlist(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<Playlist>, (StatusCode, String)> {
    let user_id = get_user_id_from_token(&headers)?;

    let playlist: Playlist = sqlx::query_as(
        "SELECT * FROM playlists WHERE id = $1 AND user_id = $2"
    )
    .bind(&id)
    .bind(&user_id)
    .fetch_one(&pool)
    .await
    .map_err(|_| (StatusCode::NOT_FOUND, "Playlist not found".to_string()))?;

    Ok(Json(playlist))
}

async fn create_playlist(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Json(req): Json<CreatePlaylistRequest>,
) -> Result<(StatusCode, Json<Playlist>), (StatusCode, String)> {
    let user_id = get_user_id_from_token(&headers)?;
    let playlist_id = Uuid::new_v4();

    sqlx::query_as::<_, Playlist>(
        r#"
        INSERT INTO playlists (id, user_id, name, description, matches)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#
    )
    .bind(&playlist_id)
    .bind(&user_id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.matches)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
    .map(|p| (StatusCode::CREATED, Json(p)))
}

async fn update_playlist(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdatePlaylistRequest>,
) -> Result<Json<Playlist>, (StatusCode, String)> {
    let user_id = get_user_id_from_token(&headers)?;

    sqlx::query(
        r#"
        UPDATE playlists 
        SET 
            name = COALESCE($1, name),
            description = COALESCE($2, description),
            matches = COALESCE($3, matches),
            updated_at = NOW()
        WHERE id = $4 AND user_id = $5
        "#
    )
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.matches)
    .bind(&id)
    .bind(&user_id)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let playlist: Playlist = sqlx::query_as(
        "SELECT * FROM playlists WHERE id = $1 AND user_id = $2"
    )
    .bind(&id)
    .bind(&user_id)
    .fetch_one(&pool)
    .await
    .map_err(|_| (StatusCode::NOT_FOUND, "Playlist not found".to_string()))?;

    Ok(Json(playlist))
}

async fn delete_playlist(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = get_user_id_from_token(&headers)?;

    let result = sqlx::query(
        "DELETE FROM playlists WHERE id = $1 AND user_id = $2"
    )
    .bind(&id)
    .bind(&user_id)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Playlist not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}
