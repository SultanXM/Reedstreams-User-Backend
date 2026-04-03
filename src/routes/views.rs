use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use sqlx::PgPool;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TrackViewRequest {
    pub match_id: String,
}

#[derive(serde::Serialize)]
pub struct ViewResponse {
    pub views: i32,
}

pub fn views_routes(pool: PgPool) -> Router {
    Router::new()
        .nest("/views", Router::new()
            .route("/track", post(track_view))
            .route("/all", get(get_all_views))
            .route("/:match_id", get(get_views))
        )
        .with_state(pool)
}

#[derive(serde::Serialize, sqlx::FromRow)]
pub struct AllViewsResponse {
    pub match_id: String,
    pub views: i32,
}

async fn get_all_views(
    State(pool): State<PgPool>,
) -> Json<Vec<AllViewsResponse>> {
    let views = sqlx::query_as::<_, AllViewsResponse>(
        "SELECT id as match_id, COALESCE(views, 0) as views FROM matches"
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    Json(views)
}

fn get_client_ip(headers: &axum::http::HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.to_string())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "unknown".to_string())
}

async fn track_view(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<TrackViewRequest>,
) -> Json<ViewResponse> {
    let ip = get_client_ip(&headers);

    // Ensure match exists in matches table
    let _ = sqlx::query("INSERT INTO matches (id, views) VALUES ($1, 0) ON CONFLICT (id) DO NOTHING")
        .bind(&payload.match_id)
        .execute(&pool)
        .await;

    // Check if this IP already viewed this match
    let existing = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM match_views WHERE match_id = $1 AND ip_address = $2"
    )
    .bind(&payload.match_id)
    .bind(&ip)
    .fetch_one(&pool)
    .await
    .unwrap_or(0);

    if existing == 0 {
        // New view - insert and increment count
        let _ = sqlx::query("INSERT INTO match_views (match_id, ip_address) VALUES ($1, $2)")
            .bind(&payload.match_id)
            .bind(&ip)
            .execute(&pool)
            .await;

        let _ = sqlx::query("UPDATE matches SET views = COALESCE(views, 0) + 1 WHERE id = $1")
            .bind(&payload.match_id)
            .execute(&pool)
            .await;
    }

    // Get current view count
    let views = sqlx::query_scalar::<_, i32>("SELECT COALESCE(views, 0) FROM matches WHERE id = $1")
        .bind(&payload.match_id)
        .fetch_one(&pool)
        .await
        .unwrap_or(0);

    Json(ViewResponse { views })
}

async fn get_views(
    State(pool): State<PgPool>,
    Path(match_id): Path<String>,
) -> Json<ViewResponse> {
    let views = sqlx::query_scalar::<_, i32>("SELECT COALESCE(views, 0) FROM matches WHERE id = $1")
        .bind(&match_id)
        .fetch_one(&pool)
        .await
        .unwrap_or(0);

    Json(ViewResponse { views })
}
