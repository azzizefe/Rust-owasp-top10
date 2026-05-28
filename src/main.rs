// src/main.rs

#![warn(clippy::unwrap_used, clippy::expect_used)]

use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod config;
mod db;
mod error;
mod handlers;
mod middleware;
mod models;
mod routes;
mod session;
mod templates;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Tracing loglamayı başlat
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rust_owasp_top10=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("OWASP Lab Sunucusu ilklendiriliyor...");

    // 2. Konfigürasyonu yükle
    let cfg = match config::Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            error!("Konfigürasyon hatası: {}", e);
            std::process::exit(1);
        }
    };

    // 3. Veritabanı bağlantısı kur ve şemaları migrate et
    let pool = match db::connect(&cfg.database_url).await {
        Ok(p) => p,
        Err(e) => {
            error!("Veritabanına bağlanılamadı: {:?}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = db::run_migrations(&pool).await {
        error!("Migration'lar uygulanamadı: {:?}", e);
        std::process::exit(1);
    }

    // 4. Mod seçimine göre AuthBackend oluştur
    let auth_backend = auth::build(&cfg.mode, pool.clone());

    // 5. Başlangıç logunda modu açıkça yaz
    match cfg.mode {
        config::AppMode::Vulnerable => {
            warn!("⚠️⚠️⚠️ DİKKAT: Uygulama VULNERABLE (Zafiyetli) modda çalışıyor!");
            warn!("⚠️⚠️⚠️ ASLA bu modu üretim (production) ortamında kullanmayın!");
        }
        config::AppMode::Secure => {
            info!("🔒 GÜVENLİ: Uygulama SECURE (Zırhlandırılmış) modda başarıyla başlatıldı.");
        }
    }

    // 6. Router durumunu ilklendir ve sunucuyu dinlemeye aç
    let state = routes::AppState {
        auth: auth_backend,
        pool,
        mode: cfg.mode,
        session_secret: cfg.session_secret,
    };

    let app = routes::create_router(state);

    let listener = tokio::net::TcpListener::bind(&cfg.bind_addr).await?;
    info!("Sunucu başarıyla dinliyor: http://{}", cfg.bind_addr);
    
    axum::serve(listener, app).await?;

    Ok(())
}
