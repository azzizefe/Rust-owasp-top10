// crates/core/src/auth/mod.rs

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use crate::config::AppMode;
use crate::error::ApiError;
use crate::models::{LoginForm, Post, RegisterForm, Session, User};

pub mod secure;
pub mod vulnerable;

#[async_trait]
pub trait AuthBackend: Send + Sync {
    async fn register(&self, form: RegisterForm) -> Result<i64, ApiError>;
    async fn login(&self, form: LoginForm) -> Result<Session, ApiError>;
    async fn find_user(&self, id: i64) -> Result<User, ApiError>;
    async fn create_post(&self, author_id: i64, content: &str) -> Result<i64, ApiError>;
    async fn search_posts(&self, query: &str) -> Result<Vec<(Post, String)>, ApiError>; // (Post, author_username) ikilisi
}

pub fn build(mode: &AppMode, pool: PgPool) -> Arc<dyn AuthBackend> {
    match mode {
        AppMode::Vulnerable => {
            tracing::warn!("⚠️ DİKKAT: Uygulama VULNERABLE (Zafiyetli) modda başlatılıyor!");
            Arc::new(vulnerable::VulnerableAuth::new(pool))
        }
        AppMode::Secure => {
            tracing::info!("🔒 GÜVENLİ: Uygulama SECURE (Zırhlandırılmış) modda başlatılıyor!");
            Arc::new(secure::SecureAuth::new(pool))
        }
    }
}
