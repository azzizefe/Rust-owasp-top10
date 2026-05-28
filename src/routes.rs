// src/routes.rs

use axum::{
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use std::sync::Arc;

use crate::auth::AuthBackend;
use crate::config::AppMode;
use crate::handlers::{auth_handlers, post_handlers, user_handlers};

#[derive(Clone)]
pub struct AppState {
    pub auth: Arc<dyn AuthBackend>,
    pub pool: PgPool,
    pub mode: AppMode,
    pub session_secret: String,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Sağlık kontrolü
        .route("/health", get(health_check))
        
        // Kimlik doğrulama rotaları
        .route("/register", get(auth_handlers::show_register).post(auth_handlers::register))
        .route("/login", get(auth_handlers::show_login).post(auth_handlers::login))
        .route("/logout", post(auth_handlers::logout))
        
        // Profil rotası
        .route("/profile/:id", get(user_handlers::show_profile))
        .route("/api/debug", get(user_handlers::show_debug))
        
        // Post rotaları
        .route("/search", get(post_handlers::search_posts))
        .route("/posts", post(post_handlers::create_post))
        .route("/fetch", get(post_handlers::fetch_url))
        
        .with_state(state)
}

async fn health_check() -> &'static str {
    "OK"
}
