// src/handlers/user_handlers.rs

use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::IntoResponse,
};
use tracing::warn;

use crate::config::AppMode;
use crate::error::ApiError;
use crate::handlers::auth_handlers::get_cookie;
use crate::models::User;
use crate::routes::AppState;
use crate::templates::ProfileTemplate;

// İstekte bulunan aktif kullanıcıyı çözen yardımcı fonksiyon
pub async fn resolve_current_user(state: &AppState, headers: &HeaderMap) -> Option<User> {
    match state.mode {
        AppMode::Vulnerable => {
            if let Some(b64) = get_cookie(headers, "user_session") {
                use base64::Engine;
                if let Ok(decoded) =
                    base64::engine::general_purpose::STANDARD.decode(b64.as_bytes())
                {
                    if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&decoded) {
                        if let Some(id) = val.get("id").and_then(|i| i.as_i64()) {
                            // Zafiyetli modda veritabanından sorgulayıp dönüyoruz
                            return state.auth.find_user(id).await.ok();
                        }
                    }
                }
            }
            None
        }
        AppMode::Secure => {
            if let Some(token) = get_cookie(headers, "session_token") {
                if let Ok(Some((_session, user))) =
                    crate::session::get_session(&state.pool, &token).await
                {
                    return Some(user);
                }
            }
            None
        }
    }
}

pub async fn show_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(profile_id): Path<i64>,
) -> impl IntoResponse {
    let current_user = resolve_current_user(&state, &headers).await;

    // SECURE MOD: Erişim Kontrolü (Broken Access Control Engeli - OWASP A01:2026)
    if state.mode == AppMode::Secure {
        match &current_user {
            None => return Err(ApiError::Unauthorized),
            Some(user) => {
                // Sadece kendisi veya 'admin' rolündeki biri başkasının profilini görebilir
                if user.id != profile_id && user.role != "admin" {
                    warn!("🔒 GÜVENLİK İHLALİ ENGELENDİ: Kullanıcı {} başkasının ({}) profiline erişmeye çalıştı!", user.id, profile_id);
                    return Err(ApiError::Forbidden);
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
        Err(_) => Err(ApiError::NotFound),
    }
}

// ⚠️ VULNERABLE — Güvenlik Yapılandırma Hatası (Security Misconfiguration - OWASP A06:2026)
// Zafiyetli modda tüm hassas sistem verilerini (DATABASE_URL, SECRET_KEY vb.) dışarı sızdıran debug endpoint'i.
pub async fn show_debug(State(state): State<AppState>) -> impl IntoResponse {
    use axum::http::StatusCode;

    if state.mode == AppMode::Secure {
        // SECURE MOD: Debug endpoint'i tamamen kapatılır veya yetkilendirilir.
        return (
            StatusCode::FORBIDDEN,
            "🔒 GÜVENLİ: Yetkisiz debug erişimi engellendi.",
        )
            .into_response();
    }

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
