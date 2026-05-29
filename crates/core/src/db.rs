// crates/core/src/db.rs

use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::{Postgres, Transaction};
use std::time::Duration;
use tracing::{error, info};
use futures::future::BoxFuture;
use crate::error::ApiError;

pub async fn connect(url: &str) -> Result<PgPool, sqlx::Error> {
    info!("Veritabanına bağlanılıyor...");

    // 🛡️ GÜVENLİ VE OPTİMİZE BAĞLANTI HAVUZU AYARLARI (Phase 5.2)
    let pool = PgPoolOptions::new()
        .max_connections(10) // Docker / microservice ortamı için makul üst sınır
        .min_connections(2)  // Soğuk başlangıç gecikmelerini engellemek için warm pool
        .acquire_timeout(Duration::from_secs(5)) // Bağlantı alamama durumunda max bekleme süresi
        .idle_timeout(Duration::from_secs(600))  // Boştaki bağlantıların kapatılma süresi (10 dk)
        .max_lifetime(Duration::from_secs(1800)) // Bağlantıların maksimum ömrü (30 dk)
        .connect(url)
        .await
        .map_err(|e| {
            error!("Veritabanı bağlantı hatası: {:?}", e);
            e
        })?;

    // Havuz metriklerini loglama
    info!(
        "Veritabanı bağlantı havuzu kuruldu! Havuz Boyutu: {}, Boştaki Bağlantılar: {}",
        pool.size(),
        pool.num_idle()
    );

    Ok(pool)
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

/// Generic asenkron transaction sarmalayıcısı (Phase 5.1)
/// Hata durumunda otomatik ROLLBACK, başarı durumunda COMMIT işlemini garanti altına alır.
pub async fn with_tx<F, T>(pool: &PgPool, f: F) -> Result<T, ApiError>
where
    for<'c> F: FnOnce(&'c mut Transaction<'_, Postgres>) -> BoxFuture<'c, Result<T, ApiError>>,
{
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| ApiError::Internal(format!("Transaction başlatılamadı: {:?}", e)))?;

    match f(&mut tx).await {
        Ok(val) => {
            tx.commit()
                .await
                .map_err(|e| ApiError::Internal(format!("Transaction commit hatası: {:?}", e)))?;
            Ok(val)
        }
        Err(e) => {
            // Herhangi bir hatada veya erken drop durumunda rollback yapılır
            let _ = tx.rollback().await;
            Err(e)
        }
    }
}
