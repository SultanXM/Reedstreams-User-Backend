use axum::{
    extract::State,
    http::StatusCode,
    routing::{post, put},
    Json, Router,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    auth::create_token,
    models::{
        AuthResponse, ChangeUsernameRequest, CreateUserRequest, ForgotPasswordRequest,
        LoginRequest, ResetPasswordRequest,
    },
};

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/forgot-password", post(forgot_password))
        .route("/auth/reset-password", post(reset_password))
        .route("/auth/change-username", put(change_username))
        .with_state(pool)
}

async fn register(
    State(pool): State<PgPool>,
    Json(req): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), (StatusCode, String)> {
    
    let existing: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM users WHERE username = $1 OR email = $2"
    )
    .bind(&req.username)
    .bind(&req.email)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if existing.is_some() {
        return Err((StatusCode::CONFLICT, "Username or email already exists".to_string()));
    }

    let password_hash = hash(&req.password, DEFAULT_COST)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let user_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO users (id, username, email, password_hash) VALUES ($1, $2, $3, $4)"
    )
    .bind(&user_id)
    .bind(&req.username)
    .bind(&req.email)
    .bind(&password_hash)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sqlx::query(
        "INSERT INTO profiles (user_id) VALUES ($1)"
    )
    .bind(&user_id)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let token = create_token(&user_id.to_string())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            token,
            user_id: user_id.to_string(),
            username: req.username,
            is_admin: false,
        }),
    ))
}

async fn login(
    State(pool): State<PgPool>,
    Json(req): Json<LoginRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), (StatusCode, String)> {
    let user: (Uuid, String, String, bool, Option<chrono::NaiveDateTime>) = sqlx::query_as(
        "SELECT id, username, password_hash, is_admin, timeout_until FROM users WHERE username = $1"
    )
    .bind(&req.username)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()))?;

    
    if let Some(timeout) = user.4 {
        if timeout > Utc::now().naive_utc() {
            let mins_remaining = (timeout - Utc::now().naive_utc()).num_minutes();
            return Err((StatusCode::FORBIDDEN, format!("Account timed out. Try again in {} minutes", mins_remaining)));
        }
    }

    let valid = verify(&req.password, &user.2)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !valid {
        return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()));
    }

    let token = create_token(&user.0.to_string())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((
        StatusCode::OK,
        Json(AuthResponse {
            token,
            user_id: user.0.to_string(),
            username: user.1,
            is_admin: user.3,
        }),
    ))
}

async fn forgot_password(
    State(pool): State<PgPool>,
    Json(req): Json<ForgotPasswordRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {
    let user: Option<(Uuid, String)> = sqlx::query_as(
        "SELECT id, username FROM users WHERE email = $1"
    )
    .bind(&req.email)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some((user_id, username)) = user {
        let token = Uuid::new_v4().to_string();
        let expires_at = Utc::now() + Duration::hours(24);

        sqlx::query(
            "INSERT INTO password_resets (user_id, token, expires_at) VALUES ($1, $2, $3)"
        )
        .bind(&user_id)
        .bind(&token)
        .bind(&expires_at.naive_utc())
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        
        return Ok((
            StatusCode::OK,
            Json(serde_json::json!({
                "message": "Password reset token created",
                "email": req.email,
                "username": username,
                "token": token,
                "note": "In production, this would be sent via email. Use this token with /auth/reset-password"
            }))
        ));
    }

    
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "message": "If an account with this email exists, a reset link would be sent"
        }))
    ))
}

async fn reset_password(
    State(pool): State<PgPool>,
    Json(req): Json<ResetPasswordRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {
    let reset: Option<(Uuid, Uuid)> = sqlx::query_as(
        "SELECT id, user_id FROM password_resets WHERE token = $1 AND used = FALSE AND expires_at > NOW()"
    )
    .bind(&req.token)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some((reset_id, user_id)) = reset {
        let password_hash = hash(&req.new_password, DEFAULT_COST)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        sqlx::query(
            "UPDATE users SET password_hash = $1 WHERE id = $2"
        )
        .bind(&password_hash)
        .bind(&user_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        sqlx::query(
            "UPDATE password_resets SET used = TRUE WHERE id = $1"
        )
        .bind(&reset_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        return Ok((
            StatusCode::OK,
            Json(serde_json::json!({"message": "Password reset successful"}))
        ));
    }

    Err((StatusCode::BAD_REQUEST, "Invalid or expired token".to_string()))
}

async fn change_username(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    Json(req): Json<ChangeUsernameRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), (StatusCode, String)> {
    
    use crate::auth::verify_token;
    
    let auth_header = headers
        .get("authorization")
        .ok_or((StatusCode::UNAUTHORIZED, "Missing auth header".to_string()))?
        .to_str()
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid auth header".to_string()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid token format".to_string()))?;

    let claims = verify_token(token).map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".to_string()))?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid user ID".to_string()))?;
    
    
    let existing: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM users WHERE username = $1"
    )
    .bind(&req.new_username)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if existing.is_some() {
        return Err((StatusCode::CONFLICT, "Username already taken".to_string()));
    }

    
    let user: (Uuid, String, bool) = sqlx::query_as(
        "UPDATE users SET username = $1 RETURNING id, username, is_admin"
    )
    .bind(&req.new_username)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let new_token = create_token(&user.0.to_string())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((
        StatusCode::OK,
        Json(AuthResponse {
            token: new_token,
            user_id: user.0.to_string(),
            username: user.1,
            is_admin: user.2,
        }),
    ))
}
