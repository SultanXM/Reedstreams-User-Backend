use axum::Router;
use sqlx::PgPool;

mod users;
mod profiles;
mod playlists;
mod chat;
mod admin;

pub fn user_routes(pool: PgPool) -> Router {
    users::router(pool)
}

pub fn profile_routes(pool: PgPool) -> Router {
    profiles::router(pool)
}

pub fn playlist_routes(pool: PgPool) -> Router {
    playlists::router(pool)
}

pub fn chat_routes(pool: PgPool) -> Router {
    chat::router(pool)
}

pub fn admin_routes(pool: PgPool) -> Router {
    admin::router(pool)
}
