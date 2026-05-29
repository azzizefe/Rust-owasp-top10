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
                let secret_bytes = state.session_secret.as_bytes();
                let (sign_key, enc_key) = owasp_core::crypto::derive_keys(secret_bytes);

                // HMAC imzasını doğrula
                if let Ok(encrypted) =
                    owasp_core::crypto::verify_cookie(&sign_key, &signed_encrypted)
                {
                    // AES-GCM şifresini çözerek ham session token'ı elde et
                    if let Ok(token) = owasp_core::crypto::decrypt_cookie(&enc_key, &encrypted) {
                        if let Ok(Some((_session, user))) =
                            owasp_core::session::get_session(&state.pool, &token).await
                        {
                            Some(user)
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
    };

    if let Some(user) = user_opt {
        request.extensions_mut().insert(user);
    }

    next.run(request).await
}
