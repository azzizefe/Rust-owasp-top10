// tests/common/mod.rs

use std::net::TcpListener;
use sqlx::{PgPool};
use rust_owasp_top10::config::AppMode;
use rust_owasp_top10::routes::{create_router, AppState};
use rust_owasp_top10::auth;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

pub async fn spawn_app(mode: AppMode) -> TestApp {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info,sqlx=warn,tower_http=info")
        .try_init();

    // 1. .env dosyasından DATABASE_URL oku, yoksa varsayılanı kullan
    dotenvy::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/owasp_lab".to_string());

    use sqlx::postgres::PgPoolOptions;
    let db_pool = PgPoolOptions::new()
        .max_connections(2) // Paralel entegrasyon testlerinde DB tıkanıklığını önlemek için sınırlandırıyoruz
        .connect(&db_url)
        .await
        .expect("Test sunucusu veritabanına bağlanamadı");

    // 2. Auth backend'i ilklendir
    let auth_backend = auth::build(&mode, db_pool.clone());

    // 3. AppState oluştur
    let state = AppState {
        auth: auth_backend,
        pool: db_pool.clone(),
        mode,
        session_secret: "test_session_secret_32_bytes_long_12345".to_string(),
    };

    // 4. Rastgele bir port dinle (Tokio native async)
    let tokio_listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Rastgele porta bağlanılamadı");
    let port = tokio_listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    // 5. Router'ı oluştur ve arka planda tokio ile ayağa kaldır
    let router = create_router(state);
    tokio::spawn(async move {
        axum::serve(
            tokio_listener,
            router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap();
    });

    TestApp { address, db_pool }
}
