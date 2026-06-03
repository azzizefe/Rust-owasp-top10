// crates/web/src/handlers/post_handlers.rs

use ammonia::clean;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    Form,
};
use tracing::warn;

use crate::error_response::AppError;
use crate::extractors::{AuthenticatedUser, OptionalUser};
use crate::routes::AppState;
use crate::templates::SearchTemplate;
use owasp_core::config::AppMode;
use owasp_core::models::{NewPost, SearchQuery};

pub async fn create_post(
    State(state): State<AppState>,
    AuthenticatedUser(current_user): AuthenticatedUser,
    Form(form): Form<NewPost>,
) -> Result<Redirect, AppError> {
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

    state
        .auth
        .create_post(current_user.id, &content)
        .await
        .map_err(AppError::from)?;

    Ok(Redirect::to("/search"))
}

pub async fn search_posts(
    State(state): State<AppState>,
    OptionalUser(current_user): OptionalUser,
    Query(query_param): Query<SearchQuery>,
) -> impl IntoResponse {
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

            let scheme = parsed_url.scheme();
            if scheme != "http" && scheme != "https" {
                return (
                    StatusCode::BAD_REQUEST,
                    "Sadece HTTP ve HTTPS şemalarına izin verilir.",
                )
                    .into_response();
            }

            let host_str = parsed_url.host_str().unwrap_or("");
            let port = parsed_url.port_or_known_default().unwrap_or(80);

            // DNS Çözümlemesi ile Gerçek IP Adresini Doğrulama (SSRF Bypass Önlemi)
            let resolved_ips = match tokio::net::lookup_host((host_str, port)).await {
                Ok(addrs) => {
                    let mut ips = Vec::new();
                    for addr in addrs {
                        ips.push(addr.ip());
                    }
                    if ips.is_empty() {
                        return (StatusCode::BAD_REQUEST, "Sunucu adresi çözümlenemedi.")
                            .into_response();
                    }
                    ips
                }
                Err(_) => {
                    return (
                        StatusCode::BAD_REQUEST,
                        "Geçersiz sunucu adresi veya DNS hatası.",
                    )
                        .into_response()
                }
            };

            for ip in resolved_ips {
                if ip.is_loopback() || ip.is_unspecified() || ip.is_multicast() {
                    warn!("🔒 SSRF ENGELENDİ: Özel IP tespit edildi ({})", ip);
                    return (
                        StatusCode::FORBIDDEN,
                        "Özel ve yerel IP adreslerine erişim yasaktır.",
                    )
                        .into_response();
                }

                // RFC 1918 Private IP & RFC 3927 Link-Local IPv4 Kontrolleri
                match ip {
                    std::net::IpAddr::V4(ipv4) => {
                        let octets = ipv4.octets();
                        if octets[0] == 10
                            || (octets[0] == 172 && octets[1] >= 16 && octets[1] <= 31)
                            || (octets[0] == 192 && octets[1] == 168)
                            || (octets[0] == 169 && octets[1] == 254)
                        {
                            warn!("🔒 SSRF ENGELENDİ: İç ağ IP adresi tespit edildi ({})", ip);
                            return (StatusCode::FORBIDDEN, "İç ağ adreslerine erişim yasaktır.")
                                .into_response();
                        }
                    }
                    std::net::IpAddr::V6(ipv6) => {
                        let segments = ipv6.segments();
                        if (segments[0] & 0xfe00) == 0xfc00 || (segments[0] & 0xffc0) == 0xfe80 {
                            warn!(
                                "🔒 SSRF ENGELENDİ: İç ağ IPv6 adresi tespit edildi ({})",
                                ip
                            );
                            return (StatusCode::FORBIDDEN, "İç ağ adreslerine erişim yasaktır.")
                                .into_response();
                        }
                    }
                }
            }

            // Güvenli istek atımı ve Redirect Koruması (OWASP A01:2026 - SSRF)
            // Open Redirect veya yönlendirme ile localhost'a atlamayı engellemek için redirect kapatılır
            let client = match reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(3))
                .redirect(reqwest::redirect::Policy::none())
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
                        "İstek gönderilen adres yanıt vermedi veya yönlendirme yapılmaya çalışıldı.",
                    )
                        .into_response()
                }
            }
        }
    }
}
