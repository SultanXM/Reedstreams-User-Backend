use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub is_admin: bool,
    pub timeout_until: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: String,
    pub username: String,
    pub is_admin: bool,
}

#[derive(Debug, Deserialize)]
pub struct ChangeUsernameRequest {
    pub new_username: String,
}

#[derive(Debug, Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct TimeoutUserRequest {
    pub user_id: String,
    pub minutes: i64,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Profile {
    pub user_id: Uuid,
    pub tags: Option<Vec<String>>,
    pub memes: Option<Vec<String>>,
    pub theme: Option<String>,
    pub avatar_url: Option<String>,
    pub profile_pic_url: Option<String>,
    pub name_color: Option<String>,
    pub name_glow: Option<i32>,
    pub description: Option<String>,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub theme: Option<String>,
    pub avatar_url: Option<String>,
    pub description: Option<String>,
    pub name_color: Option<String>,
    pub name_glow: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UploadProfilePicRequest {
    pub image_data: String,
}

#[derive(Debug, Deserialize)]
pub struct AdminUpdateUserRequest {
    pub tags: Option<Vec<String>>,
    pub memes: Option<Vec<String>>,
    pub name_color: Option<String>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Playlist {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub matches: Vec<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreatePlaylistRequest {
    pub name: String,
    pub description: Option<String>,
    pub matches: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePlaylistRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub matches: Option<Vec<String>>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub content: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
}

#[derive(Debug, FromRow)]
pub struct PasswordReset {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub expires_at: NaiveDateTime,
    pub used: bool,
}
