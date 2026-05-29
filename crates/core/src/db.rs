// crates/core/src/db.rs

use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;
use tracing::{error, info};

pub async fn connect(url: &str) -> Result<PgPool, sqlx::Error> {
    info!("Veritabanına bağlanılıyor...");

    // Güvenli ve optimize bağlantı havuzu ayarları
    PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(30))
        .connect(url)
        .await
        .map_err(|e| {
            error!("Veritabanı bağlantı hatası: {:?}", e);
            e
        })
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    info!("Veritabanı migration'ları çalıştırılıyor...");
    sqlx::migrate!("../../migrations")
        .run(pool)
        .await
        .map_err(|e| {
            error!("Migration hatası: {:?}", e);
            e
        })?;

    info!("Veritabanı migration'ları başarıyla tamamlandı!");
    Ok(())
}
