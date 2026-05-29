// crates/web/src/error_response.rs — HTTP'ye özgü IntoResponse implementasyonu
// Orphan rule nedeniyle newtype wrapper kullanılır: AppError(ApiError)

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use owasp_core::error::ApiError;
use serde_json::json;

/// Web katmanında ApiError'ı HTTP response'a dönüştüren wrapper
pub struct AppError(pub ApiError);

impl From<ApiError> for AppError {
    fn from(e: ApiError) -> Self {
        AppError(e)
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError(ApiError::from(e))
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let app_mode = std::env::var("APP_MODE").unwrap_or_default();
        let is_vulnerable = app_mode.to_lowercase() == "vulnerable";

        let (status, error_message) = match self.0 {
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
