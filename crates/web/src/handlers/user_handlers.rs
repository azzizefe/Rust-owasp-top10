// crates/web/src/handlers/user_handlers.rs

use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use tracing::warn;

use crate::error_response::AppError;
use crate::routes::AppState;
use crate::templates::ProfileTemplate;
use owasp_core::config::AppMode;
use owasp_core::error::ApiError;

use crate::extractors::OptionalUser;

pub async fn show_profile(
    State(state): State<AppState>,
    OptionalUser(current_user): OptionalUser,
    Path(profile_id): Path<i64>,
) -> Result<ProfileTemplate, AppError> {
    // SECURE MOD: Erişim Kontrolü (Broken Access Control Engeli - OWASP A01:2026)
    if state.mode == AppMode::Secure {
        match &current_user {
            None => return Err(ApiError::Unauthorized.into()),
            Some(user) => {
                // Sadece kendisi veya 'admin' rolündeki biri başkasının profilini görebilir
                if user.id != profile_id && user.role != "admin" {
                    warn!(
                        target: "security_audit",
                        event = "idor_blocked",
                        attacker_id = %user.id,
                        target_id = %profile_id,
                        "🔒 GÜVENLİK İHLALİ ENGELLENDİ: Kullanıcı {} başkasının ({}) profiline erişmeye çalıştı!",
                        user.id,
                        profile_id
                    );
                    return Err(ApiError::Forbidden.into());
                }
            }
        }
    }

    // ⚠️ VULNERABLE — IDOR: Giriş kontrolü olmaksızın, profile_id parametresine göre profil gösterilir!
    // Saldırgan herhangi bir id vererek herkesin profilini gezebilir.
    match state.auth.find_user(profile_id).await {
        Ok(profile_user) => Ok(ProfileTemplate {
            user: profile_user,
            current_user,
            is_vulnerable: state.mode == AppMode::Vulnerable,
        }),
        Err(_) => Err(ApiError::NotFound.into()),
    }
}

// ⚠️ VULNERABLE — Güvenlik Yapılandırma Hatası (Security Misconfiguration - OWASP A06:2026)
// Zafiyetli modda tüm hassas sistem verilerini (DATABASE_URL, SECRET_KEY vb.) dışarı sızdıran debug endpoint'i.
pub async fn show_debug(State(state): State<AppState>) -> impl IntoResponse {
    use axum::http::StatusCode;

    // ⚠️ VULNERABLE: Tüm gizli anahtarlar ve veritabanı şifreleri JSON olarak dışarı sızıyor!
    let debug_info = serde_json::json!({
        "status": "vulnerable_debug_active",
        "database_url": std::env::var("DATABASE_URL").unwrap_or_default(), // Hassas bağlantı şifresi sızıyor!
        "session_secret": state.session_secret,
        "app_mode": format!("{:?}", state.mode),
        "rust_version": "1.95.0",
        "active_connections": state.pool.size(),
    });

    (StatusCode::OK, axum::Json(debug_info)).into_response()
}
