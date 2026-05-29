// crates/core/src/auth/vulnerable.rs

use async_trait::async_trait;
use sqlx::PgPool;
use tracing::warn;

use crate::auth::AuthBackend;
use crate::error::ApiError;
use crate::models::{LoginForm, Post, RegisterForm, Session, User};

// ⚠️ VULNERABLE — OWASP A04:2026 — eğitim amaçlı, ASLA production'da kullanma. Güvenli sürüm: secure.rs
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
    // ⚠️ VULNERABLE — Parolaları veritabanında PLAIN TEXT (şifresiz) tutuyor (OWASP A02:2026 & A07:2026)
    // ⚠️ VULNERABLE — SQL enjeksiyonuna tamamen açık format! makrosu kullanımı (OWASP A04:2026)
    async fn register(&self, form: RegisterForm) -> Result<i64, ApiError> {
        warn!("⚠️ VULNERABLE REGISTER TETİKLENDİ - Plaintext Parola Kaydediliyor");

        let query = format!(
            "INSERT INTO users (username, password_hash, email, role) VALUES ('{}', '{}', '{}', 'user') RETURNING id",
            form.username, form.password, form.email
        );

        let row: (i64,) = sqlx::query_as(&query).fetch_one(&self.pool).await?;

        Ok(row.0)
    }

    // ⚠️ VULNERABLE — Girdileri doğrudan SQL string'e gömerek SQLi'ye zemin hazırlar (OWASP A04:2026)
    // Payload Örneği: username = "' OR '1'='1' --"  veya  "' OR '1'='1' LIMIT 1 --"
    async fn login(&self, form: LoginForm) -> Result<Session, ApiError> {
        warn!("⚠️ VULNERABLE LOGIN TETİKLENDİ - SQL Injection ve Plaintext Kontrolü");

        let query = format!(
            "SELECT * FROM users WHERE username = '{}' AND password_hash = '{}'",
            form.username, form.password
        );

        // Zafiyetli modda veritabanı hataları unwrap/detaylı sızıntıyla panikletebilir (OWASP A10:2026)
        let user = sqlx::query_as::<_, User>(&query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?; // Hata detayları sızıyor!

        // Kriptografik olmayan zayıf rastgele token oluşturma (OWASP A02:2026)
        let token = format!("vulnerable_session_{}_{}", user.id, rand::random::<u32>());
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(24);

        let session = sqlx::query_as::<_, Session>(
            "INSERT INTO sessions (token, user_id, expires_at) VALUES ($1, $2, $3) RETURNING *",
        )
        .bind(&token)
        .bind(user.id)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(session)
    }

    // ⚠️ VULNERABLE — Yetki ve sahiplik kontrolü olmaksızın doğrudan profil sorgulama (IDOR - OWASP A01:2026)
    async fn find_user(&self, id: i64) -> Result<User, ApiError> {
        warn!("⚠️ VULNERABLE PROFILE FETCH TETİKLENDİ - IDOR Zafiyeti");

        let query = format!("SELECT * FROM users WHERE id = {}", id);

        let user = sqlx::query_as::<_, User>(&query)
            .fetch_one(&self.pool)
            .await?;

        Ok(user)
    }

    // ⚠️ VULNERABLE — Gönderileri XSS sanitasyonundan geçirmeden kaydeder (OWASP A04:2026)
    async fn create_post(&self, author_id: i64, content: &str) -> Result<i64, ApiError> {
        warn!("⚠️ VULNERABLE POST CREATE TETİKLENDİ - Stored XSS Yüzeyi");

        // SQL enjeksiyonuna da açık!
        let query = format!(
            "INSERT INTO posts (author_id, content) VALUES ({}, '{}') RETURNING id",
            author_id, content
        );

        let row: (i64,) = sqlx::query_as(&query).fetch_one(&self.pool).await?;

        Ok(row.0)
    }

    // ⚠️ VULNERABLE — SQL enjeksiyonlu gönderi arama (OWASP A04:2026)
    async fn search_posts(&self, query: &str) -> Result<Vec<(Post, String)>, ApiError> {
        warn!("⚠️ VULNERABLE POST SEARCH TETİKLENDİ - SQL Injection ve Reflected XSS");

        let raw_query = format!(
            "SELECT p.*, u.username FROM posts p JOIN users u ON p.author_id = u.id WHERE p.content LIKE '%{}%'",
            query
        );

        let rows = sqlx::query_as::<_, (i64, i64, String, chrono::DateTime<chrono::Utc>, String)>(
            &raw_query,
        )
        .fetch_all(&self.pool)
        .await?;

        let result = rows
            .into_iter()
            .map(|row| {
                let post = Post {
                    id: row.0,
                    author_id: row.1,
                    content: row.2,
                    created_at: row.3,
                };
                let username = row.4;
                (post, username)
            })
            .collect();

        Ok(result)
    }
}
