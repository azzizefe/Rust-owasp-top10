// src/handlers/auth_handlers.rs

use axum::{
    extract::State,
    http::{header, HeaderMap, Response, StatusCode},
    response::{IntoResponse, Redirect},
    Form,
};
use tracing::warn;

use crate::config::AppMode;
use crate::models::{LoginForm, RegisterForm};
use crate::routes::AppState;
use crate::templates::{LoginTemplate, RegisterTemplate};

pub async fn show_register() -> impl IntoResponse {
    RegisterTemplate { error: None }
}

pub async fn register(
    State(state): State<AppState>,
    Form(form): Form<RegisterForm>,
) -> impl IntoResponse {
    // SECURE MOD: Girdi doğrulama ve temizleme (OWASP A03:2026)
    if state.mode == AppMode::Secure {
        if let Err(err_msg) = form.validate() {
            return RegisterTemplate { error: Some(err_msg) }.into_response();
        }
    }

    match state.auth.register(form).await {
        Ok(_) => Redirect::to("/login").into_response(),
        Err(e) => {
            // Vulnerable modda veritabanı hata detayları sızar (OWASP A10:2026)
            let err_msg = if state.mode == AppMode::Vulnerable {
                format!("Kayıt hatası: {:?}", e)
            } else {
                "Kayıt işlemi sırasında bir hata oluştu.".to_string()
            };
            RegisterTemplate { error: Some(err_msg) }.into_response()
        }
    }
}

pub async fn show_login() -> impl IntoResponse {
    LoginTemplate { error: None }
}

pub async fn login(
    State(state): State<AppState>,
    Form(form): Form<LoginForm>,
) -> impl IntoResponse {
    match state.auth.login(form).await {
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
                    let b64 = base64::engine::general_purpose::STANDARD.encode(
                        cookie_json.to_string().as_bytes()
                    );
                    
                    format!(
                        "user_session={}; Path=/; HttpOnly; SameSite=Lax",
                        b64
                    )
                }
                AppMode::Secure => {
                    // SECURE MOD: Güvenli, kriptografik rastgele DB session token'ı (OWASP A02:2026)
                    format!(
                        "session_token={}; Path=/; HttpOnly; Secure; SameSite=Strict",
                        session.token
                    )
                }
            };

            let mut headers = HeaderMap::new();
            headers.insert(
                header::SET_COOKIE,
                header::HeaderValue::from_str(&cookie_str).unwrap_or_else(|_| {
                    header::HeaderValue::from_static("")
                }),
            );

            (StatusCode::SEE_OTHER, headers, Redirect::to("/search")).into_response()
        }
        Err(e) => {
            // ⚠️ VULNERABLE — Farklı hata mesajları vererek kullanıcı hesaplarını ifşa eder (User Enumeration - OWASP A07:2026)
            let err_msg = match state.mode {
                AppMode::Vulnerable => match e {
                    crate::error::ApiError::Unauthorized => "Şifre yanlış!".to_string(),
                    crate::error::ApiError::NotFound => "Kullanıcı adı bulunamadı!".to_string(),
                    _ => format!("Giriş hatası: {:?}", e), // Hata sızıntısı
                },
                AppMode::Secure => {
                    // SECURE MOD: Genel hata mesajı verilerek enumeration engellenir
                    "Kullanıcı adı veya şifre hatalı.".to_string()
                }
            };
            LoginTemplate { error: Some(err_msg) }.into_response()
        }
    }
}

pub async fn logout(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let mut response_headers = HeaderMap::new();

    if state.mode == AppMode::Secure {
        // Çerezi temizle ve veritabanı oturumunu sil
        if let Some(token) = get_cookie(&headers, "session_token") {
            let _ = crate::session::delete_session(&state.pool, &token).await;
        }
        response_headers.insert(
            header::SET_COOKIE,
            header::HeaderValue::from_static("session_token=; Path=/; Max-Age=0; HttpOnly; Secure; SameSite=Strict"),
        );
    } else {
        response_headers.insert(
            header::SET_COOKIE,
            header::HeaderValue::from_static("user_session=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax"),
        );
    }

    (StatusCode::SEE_OTHER, response_headers, Redirect::to("/login")).into_response()
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
