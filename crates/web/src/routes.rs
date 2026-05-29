// crates/web/src/routes.rs

use axum::{
    routing::{get, post},
    response::IntoResponse,
    Router,
};
use sqlx::PgPool;
use std::sync::Arc;

use crate::handlers::{auth_handlers, post_handlers, user_handlers};
use owasp_core::auth::AuthBackend;
use owasp_core::config::AppMode;

#[derive(Clone)]
pub struct AppState {
    pub auth: Arc<dyn AuthBackend>,
    pub pool: PgPool,
    pub mode: AppMode,
    pub session_secret: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
}

use crate::middleware::auth::authenticate;
use crate::middleware::role_guard::{require_admin, require_user};
use axum::middleware::from_fn_with_state;

pub fn create_router(state: AppState) -> Router {
    let mode = state.mode;

    // Temel router oluşturuluyor
    let mut router = Router::new()
        // Ana sayfa yönlendirmesi
        .route(
            "/",
            get(|| async { axum::response::Redirect::to("/login") }),
        )
        // Sağlık kontrolü
        .route("/health", get(health_check))
        // Kimlik doğrulama rotaları
        .route(
            "/register",
            get(auth_handlers::show_register).post(auth_handlers::register),
        )
        .route(
            "/login",
            get(auth_handlers::show_login).post(auth_handlers::login),
        )
        .route("/logout", post(auth_handlers::logout))
        // Profil rotası (Giriş zorunlu)
        .route(
            "/profile/:id",
            get(user_handlers::show_profile)
                .route_layer(from_fn_with_state(state.clone(), require_user)),
        )
        // Admin debug rotası (Sadece Admin yetkilidir)
        .route(
            "/api/debug",
            get(user_handlers::show_debug)
                .route_layer(from_fn_with_state(state.clone(), require_admin)),
        )
        // Post rotaları
        .route(
            "/search",
            get(post_handlers::search_posts)
                .route_layer(from_fn_with_state(state.clone(), require_user)),
        )
        .route(
            "/posts",
            post(post_handlers::create_post)
                .route_layer(from_fn_with_state(state.clone(), require_user)),
        )
        .route("/fetch", get(post_handlers::fetch_url))
        // Tüm isteklerde önce oturum çözme katmanı (Authenticate) çalışır
        .layer(from_fn_with_state(state.clone(), authenticate))
        // En dışta isteklerin Correlation ID ile etiketlenmesi sağlanır
        .layer(axum::middleware::from_fn(crate::middleware::request_id::correlation_id))
        .with_state(state);

    // SECURE MOD: Güvenlik başlıkları, rate limiting ve body limit korumaları (OWASP A05 & A06:2026)
    if mode == AppMode::Secure {
        use axum::http::header;
        use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
        use tower_http::{limit::RequestBodyLimitLayer, set_header::SetResponseHeaderLayer};

        // 1. IP Başına Hız Sınırı (Rate Limiting - Brute-Force Koruması - OWASP A07:2026)
        // Her 2 saniyede 1 istek limiti, maksimum 5 istek birikimi (burst)
        let governor_config = Box::leak(Box::new(
            GovernorConfigBuilder::default()
                .per_second(2)
                .burst_size(5)
                .finish()
                .unwrap(),
        ));

        // 2. Güvenlik Header Katmanları (Security Headers - OWASP A06:2026)
        router = router
            .layer(GovernorLayer {
                config: governor_config,
            })
            .layer(RequestBodyLimitLayer::new(64 * 1024)) // 64KB büyük istek boyutu DoS engeli (OWASP A10:2026)
            .layer(SetResponseHeaderLayer::overriding(
                header::CONTENT_SECURITY_POLICY,
                header::HeaderValue::from_static(
                    "default-src 'self'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src 'self' https://fonts.gstatic.com; script-src 'self'; object-src 'none'; base-uri 'self'",
                ),
            ))
            .layer(SetResponseHeaderLayer::overriding(
                header::X_FRAME_OPTIONS,
                header::HeaderValue::from_static("DENY"),
            ))
            .layer(SetResponseHeaderLayer::overriding(
                header::X_CONTENT_TYPE_OPTIONS,
                header::HeaderValue::from_static("nosniff"),
            ))
            .layer(SetResponseHeaderLayer::overriding(
                header::REFERRER_POLICY,
                header::HeaderValue::from_static("no-referrer"),
            ));
    }

    router
}

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde_json::json;

async fn health_check(State(state): State<AppState>) -> impl axum::response::IntoResponse {
    let db_check = sqlx::query("SELECT 1")
        .execute(&state.pool)
        .await;

    let uptime = (chrono::Utc::now() - state.start_time).num_seconds();

    match db_check {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "status": "healthy",
                "db": "connected",
                "uptime_secs": uptime
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Health Check DB Hatası: {:?}", e);
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "status": "unhealthy",
                    "db": "disconnected",
                    "uptime_secs": uptime
                })),
            )
                .into_response()
        }
    }
}
