// src/error.rs

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use tracing::error;

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("unauthorized")]
    Unauthorized,

    #[error("forbidden")]
    Forbidden,

    #[error("not found")]
    NotFound,

    #[error("too many requests")]
    RateLimited,

    #[error("internal error: {0}")]
    Internal(String),
}

// sqlx::Error'ları otomatik ApiError::Internal'a dönüştür ve orijinal hatayı logla (OWASP A05 Koruması)
impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        error!("Veritabanı hatası oluştu: {:?}", err);
        ApiError::Internal(err.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let app_mode = std::env::var("APP_MODE").unwrap_or_default();
        let is_vulnerable = app_mode.to_lowercase() == "vulnerable";

        let (status, error_message) = match self {
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            ApiError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden".to_string()),
            ApiError::NotFound => (StatusCode::NOT_FOUND, "Not Found".to_string()),
            ApiError::RateLimited => (
                StatusCode::TOO_MANY_REQUESTS,
                "Too Many Requests".to_string(),
            ),
            ApiError::Internal(detailed_err) => {
                // A05 & A10: Güvenli modda (Secure) iç hata detayı ASLA sızmaz.
                // Zafiyetli modda (Vulnerable) ise stack trace / SQL detayı sızdırılır.
                let msg = if is_vulnerable {
                    format!("Internal Server Error: {}", detailed_err)
                } else {
                    "Internal Server Error".to_string()
                };
                (StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}
