// src/auth/vulnerable.rs

use async_trait::async_trait;
use sqlx::PgPool;

use crate::auth::AuthBackend;
use crate::error::ApiError;
use crate::models::{LoginForm, Post, RegisterForm, Session, User};

pub struct VulnerableAuth {
    pool: PgPool,
}

impl VulnerableAuth {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuthBackend for VulnerableAuth {
    async fn register(&self, form: RegisterForm) -> Result<i64, ApiError> {
        // Zafiyetli kayıt implementasyonu (ilerleyen fazda doldurulacak)
        Ok(1)
    }

    async fn login(&self, form: LoginForm) -> Result<Session, ApiError> {
        // Zafiyetli giriş implementasyonu (ilerleyen fazda doldurulacak)
        Err(ApiError::Unauthorized)
    }

    async fn find_user(&self, id: i64) -> Result<User, ApiError> {
        // Zafiyetli profil arama implementasyonu (ilerleyen fazda doldurulacak)
        Err(ApiError::NotFound)
    }

    async fn create_post(&self, author_id: i64, content: &str) -> Result<i64, ApiError> {
        // Zafiyetli post oluşturma implementasyonu (ilerleyen fazda doldurulacak)
        Ok(1)
    }

    async fn search_posts(&self, query: &str) -> Result<Vec<(Post, String)>, ApiError> {
        // Zafiyetli post arama implementasyonu (ilerleyen fazda doldurulacak)
        Ok(vec![])
    }
}
