// crates/web/src/middleware/auth.rs

use crate::handlers::auth_handlers::get_cookie;
use crate::routes::AppState;
use axum::{body::Body, extract::State, http::Request, middleware::Next, response::Response};
use owasp_core::config::AppMode;

pub async fn authenticate(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    let user_opt = match state.mode {
        AppMode::Vulnerable => {
            if let Some(b64) = get_cookie(request.headers(), "user_session") {
                use base64::Engine;
                if let Ok(decoded) =
                    base64::engine::general_purpose::STANDARD.decode(b64.as_bytes())
                {
                    if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&decoded) {
                        if let Some(id) = val.get("id").and_then(|i| i.as_i64()) {
                            state.auth.find_user(id).await.ok()
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }
        AppMode::Secure => {
            if let Some(signed_encrypted) = get_cookie(request.headers(), "session_token") {
                tracing::info!("🔑 AUTH: session_token çerezi bulundu!");
                let secret_bytes = state.session_secret.as_bytes();
                let (sign_key, enc_key) = owasp_core::crypto::derive_keys(secret_bytes);

                // HMAC imzasını doğrula
                match owasp_core::crypto::verify_cookie(&sign_key, &signed_encrypted) {
                    Ok(encrypted) => {
                        tracing::info!("🔑 AUTH: HMAC imzası başarıyla doğrulandı!");
                        // AES-GCM şifresini çözerek ham session token'ı elde et
                        match owasp_core::crypto::decrypt_cookie(&enc_key, &encrypted) {
                            Ok(token) => {
                                tracing::info!(
                                    "🔑 AUTH: AES-GCM şifresi başarıyla çözüldü! token={}",
                                    token
                                );
                                match owasp_core::session::get_session(&state.pool, &token).await {
                                    Ok(Some((_session, user))) => {
                                        tracing::info!("🔑 AUTH: Oturum veritabanından başarıyla çekildi! user={}", user.username);
                                        Some(user)
                                    }
                                    Ok(None) => {
                                        tracing::warn!("🔑 AUTH: Oturum veritabanında bulunamadı!");
                                        None
                                    }
                                    Err(e) => {
                                        tracing::error!(
                                            "🔑 AUTH: Oturum sorgusu DB hatası: {:?}",
                                            e
                                        );
                                        None
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::error!("🔑 AUTH: AES-GCM şifre çözme hatası: {:?}", e);
                                None
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("🔑 AUTH: HMAC imza doğrulama hatası: {:?}", e);
                        None
                    }
                }
            } else {
                tracing::warn!("🔑 AUTH: session_token çerezi request header'larında bulunamadı!");
                None
            }
        }
    };

    if let Some(user) = user_opt {
        request.extensions_mut().insert(user);
    }

    next.run(request).await
}
