// src/handlers/user_handlers.rs

use axum::response::IntoResponse;

pub async fn show_profile() -> impl IntoResponse {
    "User Profile Page"
}
