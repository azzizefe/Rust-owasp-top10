// crates/web/src/main.rs

#![warn(clippy::unwrap_used, clippy::expect_used)]

use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use owasp_core::{auth, config, db, secrets};
use owasp_web::routes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Tracing loglamayı başlat
    let log_format = std::env::var("LOG_FORMAT").unwrap_or_else(|_| "pretty".to_string());
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "owasp_web=info,owasp_core=info,tower_http=info".into());

    let registry = tracing_subscriber::registry().with(env_filter);

    if log_format == "json" {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .json()
            .flatten_event(true)
            .with_current_span(true)
            .with_span_list(true);
        registry.with(fmt_layer).init();
    } else {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .pretty()
            .with_thread_ids(true);
        registry.with(fmt_layer).init();
    }

    info!("OWASP Lab Sunucusu ilklendiriliyor...");

    // 2. Secrets provider'ı oluştur ve konfigürasyonu yükle
    let provider = secrets::build_provider_async().await;
    info!(
        secrets_provider = provider.provider_name(),
        "Secrets provider başarıyla yüklendi"
    );

    let cfg = match config::Config::load(&*provider).await {
        Ok(c) => c,
        Err(e) => {
            error!("Konfigürasyon hatası: {}", e);
            std::process::exit(1);
        }
    };

    // 3. Veritabanı bağlantısı kur ve şemaları migrate et
    let skip_tls = std::env::var("SKIP_DB_TLS").unwrap_or_default() == "true";
    let require_ssl = cfg.mode == config::AppMode::Secure && !skip_tls;
    let pool = match db::connect(&cfg.database_url, require_ssl).await {
        Ok(p) => p,
        Err(e) => {
            error!("Veritabanına bağlanılamadı: {:?}", e);
            std::process::exit(1);
        }
    };

    let pool_clone = pool.clone();

    // 🛡️ Principle of Least Privilege: Decouple DDL migrations from application runtime in SECURE mode.
    // In SECURE mode, the application runs using a low-privileged DML-only user (owasp_app_user).
    // Schema DDL changes are run prior to deployment in the CI/CD pipeline using owasp_migration_user.
    if cfg.mode == config::AppMode::Secure {
        info!("🔒 SECURE Mode: Skipping runtime database migrations (Schema migrations must be run out-of-band in the CI/CD pipeline).");
    } else {
        info!("⚠️ VULNERABLE Mode: Running runtime database migrations automatically.");
        if let Err(e) = db::run_migrations(&pool).await {
            error!("Migration'lar uygulanamadı: {:?}", e);
            std::process::exit(1);
        }
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
        cookie_secure: cfg.cookie_secure,
        start_time: chrono::Utc::now(),
    };

    let app = routes::create_router(state);

    let listener = tokio::net::TcpListener::bind(&cfg.bind_addr).await?;
    info!("Sunucu başarıyla dinliyor: http://{}", cfg.bind_addr);

    // Graceful Shutdown (Phase 5.3) ile sunucuyu ayağa kaldır
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal(pool_clone))
    .await?;

    Ok(())
}

async fn shutdown_signal(pool: sqlx::PgPool) {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut sig) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            sig.recv().await;
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    warn!("Server shutting down gracefully...");

    // Aktif bağlantıların bitmesini bekleyerek bağlantı havuzunu kapatır
    pool.close().await;
    info!("Veritabanı bağlantı havuzu başarıyla kapatıldı.");
}
