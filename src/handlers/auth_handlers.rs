// src/handlers/auth_handlers.rs

use axum::response::IntoResponse;

pub async fn show_register() -> impl IntoResponse {
    "Register Page"
}

pub async fn register() -> impl IntoResponse {
    "Register Action"
}

pub async fn show_login() -> impl IntoResponse {
    "Login Page"
}

pub async fn login() -> impl IntoResponse {
    "Login Action"
}

pub async fn logout() -> impl IntoResponse {
    "Logout Action"
}
