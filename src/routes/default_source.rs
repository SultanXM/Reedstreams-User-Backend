use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DefaultSourceSetting {
    pub id: i32,
    pub source_name: String,
    pub is_default: bool,
    pub priority: i32,
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDefaultSourceRequest {
    pub source_name: String,
    pub is_default: bool,
    pub priority: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct DefaultSourceResponse {
    pub default_source: Option<String>,
    pub all_sources: Vec<DefaultSourceSetting>,
}

#[derive(Debug, Deserialize)]
pub struct SyncSourcesRequest {
    pub sources: Vec<String>,
}

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/admin/default-source", get(get_default_source))
        .route("/admin/default-source", put(update_default_source))
        .route("/admin/default-source/list", get(list_all_sources))
        .route("/admin/default-source/sync", post(sync_sources))
        .with_state(pool)
}

async fn get_default_source(
    State(pool): State<PgPool>,
) -> Result<Json<DefaultSourceResponse>, (StatusCode, String)> {
    // Get all active sources
    let all_sources: Vec<DefaultSourceSetting> = sqlx::query_as(
        r#"
        SELECT id, source_name, is_default, priority, is_active
        FROM default_source_settings
        WHERE is_active = TRUE
        ORDER BY priority ASC
        "#
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Get the current default source
    let default_source: Option<(String,)> = sqlx::query_as(
        r#"
        SELECT source_name
        FROM default_source_settings
        WHERE is_default = TRUE AND is_active = TRUE
        LIMIT 1
        "#
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(DefaultSourceResponse {
        default_source: default_source.map(|s| s.0),
        all_sources,
    }))
}

async fn update_default_source(
    State(pool): State<PgPool>,
    Json(req): Json<UpdateDefaultSourceRequest>,
) -> Result<Json<DefaultSourceSetting>, (StatusCode, String)> {
    // First, unset all defaults
    sqlx::query(
        r#"
        UPDATE default_source_settings
        SET is_default = FALSE, updated_at = NOW()
        WHERE is_default = TRUE
        "#
    )
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Set the new default
    let updated_source: DefaultSourceSetting = sqlx::query_as(
        r#"
        UPDATE default_source_settings
        SET 
            is_default = TRUE,
            priority = COALESCE($1, priority),
            is_active = COALESCE($2, is_active),
            updated_at = NOW()
        WHERE source_name = $3
        RETURNING id, source_name, is_default, priority, is_active
        "#
    )
    .bind(req.priority)
    .bind(req.is_active.unwrap_or(true))
    .bind(&req.source_name)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        if e.to_string().contains("no rows returned") {
            (StatusCode::NOT_FOUND, "Source not found".to_string())
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    })?;

    Ok(Json(updated_source))
}

async fn list_all_sources(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<DefaultSourceSetting>>, (StatusCode, String)> {
    let sources: Vec<DefaultSourceSetting> = sqlx::query_as(
        r#"
        SELECT id, source_name, is_default, priority, is_active
        FROM default_source_settings
        ORDER BY priority ASC
        "#
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(sources))
}

async fn sync_sources(
    State(pool): State<PgPool>,
    Json(req): Json<SyncSourcesRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // Delete all existing sources
    sqlx::query("DELETE FROM default_source_settings")
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Insert new sources from request
    let mut priority = 1;
    for source_name in req.sources {
        sqlx::query(
            r#"
            INSERT INTO default_source_settings (source_name, is_default, priority, is_active)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (source_name) DO NOTHING
            "#
        )
        .bind(&source_name)
        .bind(priority == 1) // First source becomes default
        .bind(priority)
        .bind(true)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        priority += 1;
    }

    Ok(Json(serde_json::json!({
        "message": "Sources synchronized successfully",
        "count": priority - 1
    })))
}
