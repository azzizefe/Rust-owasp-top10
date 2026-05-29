// crates/core/src/error.rs — Domain Error (HTTP bağımlılığı SIFIR)

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
