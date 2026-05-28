// src/handlers/post_handlers.rs

use ammonia::clean;
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect},
    Form,
};
use tracing::warn;

use crate::config::AppMode;
use crate::error::ApiError;
use crate::handlers::user_handlers::resolve_current_user;
use crate::models::{NewPost, SearchQuery};
use crate::routes::AppState;
use crate::templates::SearchTemplate;

pub async fn create_post(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<NewPost>,
) -> impl IntoResponse {
    let current_user = match resolve_current_user(&state, &headers).await {
        Some(user) => user,
        None => return Err(ApiError::Unauthorized),
    };

    let content = match state.mode {
        AppMode::Vulnerable => {
            // ⚠️ VULNERABLE — Stored XSS (OWASP A04:2026)
            // Zafiyetli modda girdi hiçbir temizleme (sanitization) işleminden geçmeden kaydedilir.
            // Payload: <script>alert('Stored XSS')</script>
            form.content
        }
        AppMode::Secure => {
            // SECURE MOD: HTML Sanitization (OWASP A04:2026 Engeli)
            // Ammonia kütüphanesi ile tehlikeli tüm HTML etiketleri (script, iframe vb.) arındırılır.
            clean(&form.content)
        }
    };

    state.auth.create_post(current_user.id, &content).await?;

    Ok(Redirect::to("/search"))
}

pub async fn search_posts(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query_param): Query<SearchQuery>,
) -> impl IntoResponse {
    let current_user = resolve_current_user(&state, &headers).await;
    let query_str = query_param.q.unwrap_or_default();

    // ⚠️ VULNERABLE — Exception Leakage & SQLi (OWASP A10:2026 & A04:2026)
    // Zafiyetli modda arama yaparken, eğer SQLi tetiklenirse veritabanı hata mesajı doğrudan sızar.
    let posts = match state.auth.search_posts(&query_str).await {
        Ok(p) => p,
        Err(e) => {
            return if state.mode == AppMode::Vulnerable {
                // Hata detayı sızdırılıyor
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("SQL Arama Hatası: {:?}", e),
                )
                    .into_response()
            } else {
                // Güvenli hata mesajı
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Arama sırasında bir hata oluştu.",
                )
                    .into_response()
            };
        }
    };

    // Şablonu render et (Reflected XSS zafiyet modu is_vulnerable ile kontrol edilecek)
    SearchTemplate {
        query: query_str,
        posts,
        current_user,
        is_vulnerable: state.mode == AppMode::Vulnerable,
    }
    .into_response()
}

#[derive(serde::Deserialize)]
pub struct FetchUrlQuery {
    pub url: String,
}

// ⚠️ VULNERABLE — Sunucu Taraflı İstek Sahteciliği (SSRF - OWASP A01:2026)
// ⚠️ VULNERABLE — İstisnai Durumların Yanlış Yönetilmesi (unwrap kullanımı ile DoS/Crash - OWASP A10:2026)
pub async fn fetch_url(
    State(state): State<AppState>,
    Query(q): Query<FetchUrlQuery>,
) -> impl IntoResponse {
    let url_str = q.url;

    match state.mode {
        AppMode::Vulnerable => {
            // ⚠️ VULNERABLE: SSRF - URL doğrulaması yapılmaz, yerel IP'ler engellenmez.
            // ⚠️ VULNERABLE: A10 DoS - Geçersiz URL verilirse .unwrap() nedeniyle sunucu çöker (Crash/Panic DoS).
            let client = reqwest::Client::new();

            // unwrap() tetikleme simülasyonu
            let resp = client
                .get(&url_str)
                .send()
                .await
                .unwrap() // Geçersiz şema/host verilirse panikler!
                .text()
                .await
                .unwrap();

            (StatusCode::OK, resp).into_response()
        }
        AppMode::Secure => {
            // SECURE MOD: SSRF & Exception Handling Koruması
            // URL parse ve doğrulaması
            let parsed_url = match url::Url::parse(&url_str) {
                Ok(u) => u,
                Err(_) => return (StatusCode::BAD_REQUEST, "Geçersiz URL formatı").into_response(),
            };

            // Localhost / Özel ağ engeli (Blacklisting)
            if let Some(host) = parsed_url.host_str() {
                let h_lower = host.to_lowercase();
                if h_lower == "localhost"
                    || h_lower == "127.0.0.1"
                    || h_lower.starts_with("192.168.")
                    || h_lower.starts_with("10.")
                    || h_lower.starts_with("172.16.")
                {
                    warn!(
                        "🔒 SSRF ENGELENDİ: Kullanıcı iç ağdaki adrese ({}) erişmeye çalıştı!",
                        host
                    );
                    return (StatusCode::FORBIDDEN, "İç ağ adreslerine erişim yasaktır.")
                        .into_response();
                }
            }

            // Güvenli istek atımı
            let client = match reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(3))
                .build()
            {
                Ok(c) => c,
                Err(_) => {
                    return (StatusCode::INTERNAL_SERVER_ERROR, "İstemci oluşturulamadı.")
                        .into_response()
                }
            };

            match client.get(url_str).send().await {
                Ok(resp) => {
                    let status = resp.status();
                    match resp.text().await {
                        Ok(body) => (status, body).into_response(),
                        Err(_) => {
                            (StatusCode::BAD_GATEWAY, "Yanıttaki veri okunamadı.").into_response()
                        }
                    }
                }
                Err(e) => {
                    warn!("İstek başarısız oldu: {:?}", e);
                    (
                        StatusCode::BAD_GATEWAY,
                        "İstek gönderilen adres yanıt vermedi.",
                    )
                        .into_response()
                }
            }
        }
    }
}
