// src/auth/secure.rs

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, PasswordVerifier, SaltString},
    Argon2, PasswordHash,
};
use async_trait::async_trait;
use sqlx::PgPool;
use tracing::warn;

use crate::auth::AuthBackend;
use crate::error::ApiError;
use crate::models::{LoginForm, Post, RegisterForm, Session, User};

// ✅ SECURE — OWASP A03 & A07:2026 Uyumlu Zırhlandırılmış Auth Yapısı
pub struct SecureAuth {
    pool: PgPool,
}

impl SecureAuth {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuthBackend for SecureAuth {
    // ✅ SECURE: Güçlü Argon2id Parola Hashleme (OWASP A07:2026) ve Parameterized SQLi Koruması (OWASP A04:2026)
    async fn register(&self, form: RegisterForm) -> Result<i64, ApiError> {
        let password = form.password.clone();
        
        // Argon2id ile güçlü, salt'lı hash üretimini spawn_blocking ile asenkron havuzda yapıyoruz
        // Bu sayede CPU-intensive hash işlemi asenkron executor'ı bloke etmez!
        let password_hash = tokio::task::spawn_blocking(move || {
            let salt = SaltString::generate(&mut OsRng);
            Argon2::default()
                .hash_password(password.as_bytes(), &salt)
                .map(|h| h.to_string())
                .map_err(|e| ApiError::Internal(e.to_string()))
        })
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))??;

        // Parametreli sorgu ($1, $2, $3) girdiyi SQL yapısından ayırır (SQLi Engeli)
        let row = sqlx::query!(
            "INSERT INTO users (username, password_hash, email, role) VALUES ($1, $2, $3, 'user') RETURNING id",
            form.username,
            password_hash,
            form.email
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.id)
    }

    // ✅ SECURE: Parametreli SQL sorgusu + Zamanlama Analizi (Timing Attack) Koruması (OWASP A07:2026)
    async fn login(&self, form: LoginForm) -> Result<Session, ApiError> {
        // Parametreli sorgu ile kullanıcı araması (SQLi engeli)
        let user_opt = sqlx::query_as::<_, User>(
            "SELECT id, username, password_hash, email, role, created_at FROM users WHERE username = $1"
        )
        .bind(&form.username)
        .fetch_optional(&self.pool)
        .await?;

        // Zamanlama Saldırısı (Timing Attack) Koruması:
        // Eğer kullanıcı yoksa bile sahte bir Argon2id doğrulaması çalıştırarak
        // saldırganın geçerli/geçersiz kullanıcı analizini yapmasını engelliyoruz!
        if user_opt.is_none() {
            let password = form.password.clone();
            let _ = tokio::task::spawn_blocking(move || {
                let dummy_hash = "$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$dGVzdHBhc3N3b3JkZHVtbXloYXNo";
                if let Ok(parsed_hash) = PasswordHash::new(dummy_hash) {
                    let _ = Argon2::default().verify_password(password.as_bytes(), &parsed_hash);
                }
            })
            .await;
            
            warn!("🔒 GÜVENLİK AUDIT: Oturum açma başarısız (Kullanıcı bulunamadı) - IP/Rate-limit izlemesi etkin.");
            return Err(ApiError::Unauthorized);
        }

        let user = user_opt.expect("Kullanıcı varlığı doğrulandı");
        let password = form.password.clone();
        let password_hash_str = user.password_hash.clone();

        // Parola hash doğrulaması spawn_blocking içinde yapılarak ana asenkron kanal rahatlatılır
        let verify_res = tokio::task::spawn_blocking(move || {
            let parsed_hash = PasswordHash::new(&password_hash_str)
                .map_err(|e| {
                    tracing::error!("Parola hash parse hatası: {:?}", e);
                    ApiError::Unauthorized
                })?;

            Argon2::default()
                .verify_password(password.as_bytes(), &parsed_hash)
                .map_err(|_| ApiError::Unauthorized)
        })
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

        if verify_res.is_err() {
            warn!("🔒 GÜVENLİK AUDIT: Yanlış şifre denemesi. Kullanıcı: {}", user.username);
            return Err(ApiError::Unauthorized);
        }

        // Güvenli, kriptografik rastgele oturum token'ı oluştur (OWASP A02:2026)
        let session = crate::session::create_session(&self.pool, user.id, 2).await?;
        Ok(session)
    }

    // ✅ SECURE: Parametreli profil araması (SQLi engeli)
    async fn find_user(&self, id: i64) -> Result<User, ApiError> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, username, password_hash, email, role, created_at FROM users WHERE id = $1"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    // ✅ SECURE: Parametreli post oluşturma (SQLi engeli)
    async fn create_post(&self, author_id: i64, content: &str) -> Result<i64, ApiError> {
        let row = sqlx::query!(
            "INSERT INTO posts (author_id, content) VALUES ($1, $2) RETURNING id",
            author_id,
            content
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.id)
    }

    // ✅ SECURE: SQL Injection korumalı parametreli arama ve birleştirme sorgusu
    async fn search_posts(&self, query: &str) -> Result<Vec<(Post, String)>, ApiError> {
        let search_pattern = format!("%{}%", query);
        
        let rows = sqlx::query_as::<_, (i64, i64, String, chrono::DateTime<chrono::Utc>, String)>(
            "SELECT p.id, p.author_id, p.content, p.created_at, u.username \
             FROM posts p JOIN users u ON p.author_id = u.id \
             WHERE p.content LIKE $1"
        )
        .bind(&search_pattern)
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
