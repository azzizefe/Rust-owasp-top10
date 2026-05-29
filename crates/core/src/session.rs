// crates/core/src/session.rs

use chrono::{Duration, Utc};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sqlx::PgPool;

use crate::error::ApiError;
use crate::models::{Session, User};

// Yeni bir oturum oluşturur (kriptografik rastgele token)
pub async fn create_session(
    pool: &PgPool,
    user_id: i64,
    expires_in_hours: i64,
) -> Result<Session, ApiError> {
    // 32 karakterli güçlü, tahmin edilemez oturum belirteci üretimi (OWASP A02:2026 Koruması)
    let token: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    let expires_at = Utc::now() + Duration::hours(expires_in_hours);

    let session = sqlx::query_as::<_, Session>(
        "INSERT INTO sessions (token, user_id, expires_at) VALUES ($1, $2, $3) RETURNING *",
    )
    .bind(&token)
    .bind(user_id)
    .bind(expires_at)
    .fetch_one(pool)
    .await?;

    Ok(session)
}

// Token üzerinden geçerli bir oturumu sorgular
pub async fn get_session(pool: &PgPool, token: &str) -> Result<Option<(Session, User)>, ApiError> {
    let session_opt =
        sqlx::query_as::<_, Session>("SELECT * FROM sessions WHERE token = $1 AND expires_at > $2")
            .bind(token)
            .bind(Utc::now())
            .fetch_optional(pool)
            .await?;

    if let Some(session) = session_opt {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(session.user_id)
            .fetch_one(pool)
            .await?;
        Ok(Some((session, user)))
    } else {
        Ok(None)
    }
}

// Oturumu veritabanından siler (Logout)
pub async fn delete_session(pool: &PgPool, token: &str) -> Result<(), ApiError> {
    sqlx::query("DELETE FROM sessions WHERE token = $1")
        .bind(token)
        .execute(pool)
        .await?;
    Ok(())
}
