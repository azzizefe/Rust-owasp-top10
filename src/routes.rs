// src/routes.rs

use axum::{
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use std::sync::Arc;

use crate::auth::AuthBackend;
use crate::config::AppMode;
use crate::handlers::{auth_handlers, post_handlers, user_handlers};

#[derive(Clone)]
pub struct AppState {
    pub auth: Arc<dyn AuthBackend>,
    pub pool: PgPool,
    pub mode: AppMode,
    pub session_secret: String,
}

pub fn create_router(state: AppState) -> Router {
    let mode = state.mode;

    // Temel router oluşturuluyor
    let mut router = Router::new()
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
        // Profil rotası
        .route("/profile/:id", get(user_handlers::show_profile))
        .route("/api/debug", get(user_handlers::show_debug))
        // Post rotaları
        .route("/search", get(post_handlers::search_posts))
        .route("/posts", post(post_handlers::create_post))
        .route("/fetch", get(post_handlers::fetch_url))
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
                    "default-src 'self'; script-src 'self'; object-src 'none'; base-uri 'self'",
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

async fn health_check() -> &'static str {
    "OK"
}
