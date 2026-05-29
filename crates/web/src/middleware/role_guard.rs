// crates/web/src/middleware/role_guard.rs

use crate::error_response::AppError;
use crate::routes::AppState;
use axum::{body::Body, http::Request, middleware::Next, response::Response};
use owasp_core::config::AppMode;
use owasp_core::error::ApiError;
use owasp_core::models::User;
use tracing::warn;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequireRole {
    Admin,
    User,
}

pub async fn require_role(
    role: RequireRole,
    state: AppState,
    request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    if state.mode == AppMode::Secure {
        let user = request
            .extensions()
            .get::<User>()
            .ok_or(ApiError::Unauthorized)?;

        match role {
            RequireRole::Admin => {
                if user.role != "admin" {
                    warn!(
                        target: "security_audit",
                        event = "unauthorized_role_access",
                        attacker_id = %user.id,
                        required_role = "admin",
                        "🔒 GÜVENLİK İHLALİ ENGELLENDİ: Yetkisiz Admin paneli erişimi! Kullanıcı ID: {}",
                        user.id
                    );
                    return Err(ApiError::Forbidden.into());
                }
            }
            RequireRole::User => {
                // Her login olmuş kullanıcı erişebilir
            }
        }
    }

    Ok(next.run(request).await)
}

use axum::extract::State;

pub async fn require_admin(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    require_role(RequireRole::Admin, state, request, next).await
}

pub async fn require_user(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    require_role(RequireRole::User, state, request, next).await
}
