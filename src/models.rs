// src/models.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub email: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Session {
    pub token: String,
    pub user_id: i64,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Post {
    pub id: i64,
    pub author_id: i64,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RegisterForm {
    pub username: String,
    pub password: String,
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SearchQuery {
    pub q: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NewPost {
    pub content: String,
}

// Güvenli (Secure) mod için girdi validasyonları (OWASP A03:2026 Giriş Doğrulama)
impl RegisterForm {
    pub fn validate(&self) -> Result<(), String> {
        // Username kontrolü: 3-20 karakter, sadece alfanümerik ve alt çizgi
        if self.username.len() < 3 || self.username.len() > 20 {
            return Err("Kullanıcı adı 3 ile 20 karakter arasında olmalıdır.".to_string());
        }
        if !self.username.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err("Kullanıcı adı sadece harf, rakam ve alt çizgi içerebilir.".to_string());
        }

        // Email format kontrolü
        if !self.email.contains('@') || self.email.len() < 5 {
            return Err("Geçersiz e-posta formatı.".to_string());
        }

        // Parola kontrolü: En az 8 karakter, güçlü parola
        if self.password.len() < 8 {
            return Err("Parola en az 8 karakter uzunluğunda olmalıdır.".to_string());
        }

        Ok(())
    }
}
