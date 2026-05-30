// crates/web/src/handlers/auth_handlers.rs

use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Redirect},
    Form,
};

use crate::routes::AppState;
use crate::templates::{LoginTemplate, RegisterTemplate};
use owasp_core::config::AppMode;
use owasp_core::models::{LoginForm, RegisterForm};

pub async fn show_register() -> impl IntoResponse {
    RegisterTemplate { error: None }
}

pub async fn register(
    State(state): State<AppState>,
    Form(form): Form<RegisterForm>,
) -> impl IntoResponse {
    tracing::debug!("📥 REGISTER IST EGI ALINDI: username={}", form.username);
    // SECURE MOD: Girdi doğrulama ve temizleme (OWASP A03:2026)
    if state.mode == AppMode::Secure {
        if let Err(err_msg) = form.validate() {
            tracing::warn!("⚠️ GÜVENLİK: Register form validation failed: {}", err_msg);
            return RegisterTemplate {
                error: Some(err_msg),
            }
            .into_response();
        }
    }

    match state.auth.register(form).await {
        Ok(_) => Redirect::to("/login").into_response(),
        Err(e) => {
            tracing::error!(
                "❌ GÜVENLİK/KAYIT HATASI: Register failed with error: {:?}",
                e
            );
            // Vulnerable modda veritabanı hata detayları sızar (OWASP A10:2026)
            let err_msg = if state.mode == AppMode::Vulnerable {
                format!("Kayıt hatası: {:?}", e)
            } else {
                "Kayıt işlemi sırasında bir hata oluştu.".to_string()
            };
            RegisterTemplate {
                error: Some(err_msg),
            }
            .into_response()
        }
    }
}

pub async fn show_login() -> impl IntoResponse {
    LoginTemplate { error: None }
}

use tracing::warn;

pub async fn login(
    State(state): State<AppState>,
    Form(form): Form<LoginForm>,
) -> impl IntoResponse {
    tracing::debug!("📥 LOGIN IST EGI ALINDI: username={}", form.username);
    match state.auth.login(form.clone()).await {
        Ok(session) => {
            // Giriş başarılı, çerez (cookie) set ediliyor
            let cookie_str = match state.mode {
                AppMode::Vulnerable => {
                    // ⚠️ VULNERABLE — Bütünlük İhlali (OWASP A08:2026)
                    // Kullanıcı yetki objesini imzasız, şifresiz, düz bir Base64 cookie olarak saklar!
                    // Saldırgan bunu decode edip "role": "admin" yazarak admin yetkisi elde edebilir.
                    let cookie_json = serde_json::json!({
                        "id": session.user_id,
                        "role": "user"
                    });

                    // Base64 encode
                    use base64::Engine;
                    let b64 = base64::engine::general_purpose::STANDARD
                        .encode(cookie_json.to_string().as_bytes());

                    format!("user_session={}; Path=/; HttpOnly; SameSite=Lax", b64)
                }
                AppMode::Secure => {
                    // SECURE MOD: Çerez şifreleme ve imzalama (AEAD Tamper-Proof Cookie)
                    let secret_bytes = state.session_secret.as_bytes();
                    let (sign_key, enc_key) = owasp_core::crypto::derive_keys(secret_bytes);

                    let encrypted = owasp_core::crypto::encrypt_cookie(&enc_key, &session.token);
                    let signed_encrypted = owasp_core::crypto::sign_cookie(&sign_key, &encrypted);

                    let secure_attr = if state.cookie_secure { "; Secure" } else { "" };
                    format!(
                        "session_token={}; Path=/; HttpOnly{}; SameSite=Strict",
                        signed_encrypted, secure_attr
                    )
                }
            };

            let mut headers = HeaderMap::new();
            headers.insert(
                header::SET_COOKIE,
                header::HeaderValue::from_str(&cookie_str)
                    .unwrap_or_else(|_| header::HeaderValue::from_static("")),
            );

            (StatusCode::SEE_OTHER, headers, Redirect::to("/search")).into_response()
        }
        Err(e) => {
            tracing::error!("❌ GÜVENLİK/LOGIN HATASI: Login failed with error: {:?}", e);
            // ⚠️ VULNERABLE — Farklı hata mesajları vererek kullanıcı hesaplarını ifşa eder (User Enumeration - OWASP A07:2026)
            let err_msg = match state.mode {
                AppMode::Vulnerable => match e {
                    owasp_core::error::ApiError::Unauthorized => "Şifre yanlış!".to_string(),
                    owasp_core::error::ApiError::NotFound => {
                        "Kullanıcı adı bulunamadı!".to_string()
                    }
                    _ => format!("Giriş hatası: {:?}", e), // Hata sızıntısı
                },
                AppMode::Secure => {
                    // SECURE MOD: Genel hata mesajı verilerek enumeration engellenir
                    "Kullanıcı adı veya şifre hatalı.".to_string()
                }
            };

            if state.mode == AppMode::Secure {
                // 🛡️ SECURITY AUDIT LOG: Başarısız login denemesi (OWASP A09:2026)
                warn!(
                    target: "security_audit",
                    event = "login_failed",
                    username = %form.username,
                    error = ?e,
                    "🔒 GÜVENLİK AUDİT: Başarısız giriş denemesi!"
                );

                (
                    StatusCode::UNAUTHORIZED,
                    LoginTemplate {
                        error: Some(err_msg),
                    },
                )
                    .into_response()
            } else {
                LoginTemplate {
                    error: Some(err_msg),
                }
                .into_response()
            }
        }
    }
}

pub async fn logout(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let mut response_headers = HeaderMap::new();

    if state.mode == AppMode::Secure {
        // Çerezi temizle ve veritabanı oturumunu sil
        if let Some(token) = get_cookie(&headers, "session_token") {
            let _ = owasp_core::session::delete_session(&state.pool, &token).await;
        }
        response_headers.insert(
            header::SET_COOKIE,
            header::HeaderValue::from_static(
                "session_token=; Path=/; Max-Age=0; HttpOnly; Secure; SameSite=Strict",
            ),
        );
    } else {
        response_headers.insert(
            header::SET_COOKIE,
            header::HeaderValue::from_static(
                "user_session=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax",
            ),
        );
    }

    (
        StatusCode::SEE_OTHER,
        response_headers,
        Redirect::to("/login"),
    )
        .into_response()
}

// Cookie ayrıştırma yardımcısı
pub fn get_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| {
            s.split(';')
                .map(|pair| pair.trim())
                .find(|pair| pair.starts_with(name))
                .and_then(|pair| pair.split('=').nth(1))
                .map(|val| val.to_string())
        })
}
