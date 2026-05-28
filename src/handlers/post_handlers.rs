// src/handlers/post_handlers.rs

use axum::response::IntoResponse;

pub async fn create_post() -> impl IntoResponse {
    "Create Post Action"
}

pub async fn search_posts() -> impl IntoResponse {
    "Search Posts Action"
}
