use axum::Router;
use sqlx::PgPool;

mod users;
mod profiles;
mod playlists;
mod chat;
mod admin;
mod views;
mod default_source;
pub mod ws_views;

pub use ws_views::AppState;

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

pub fn views_routes(pool: PgPool) -> Router {
    views::views_routes(pool)
}

pub fn default_source_routes(pool: PgPool) -> Router {
    default_source::router(pool)
}

pub fn ws_views_routes(state: AppState) -> Router {
    ws_views::ws_views_routes(state)
}
