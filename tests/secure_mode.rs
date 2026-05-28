// tests/secure_mode.rs

mod common;

use reqwest::{Client, StatusCode};
use rust_owasp_top10::config::AppMode;

#[tokio::test]
async fn test_secure_sqli_blocked() {
    let app = common::spawn_app(AppMode::Secure).await;
    let client = Client::new();

    // 🛡️ SECURE SQL INJECTION PROTECTION:
    // Parametreli sorgu sayesinde SQL enjeksiyonu bloke edilir ve yetkisiz hatası (401) alınır.
    let sql_bypass_form = [
        ("username", "' OR '1'='1' --"),
        ("password", "any_wrong_password"),
    ];

    let login_resp = client
        .post(format!("{}/login", app.address))
        .form(&sql_bypass_form)
        .send()
        .await
        .unwrap();

    assert_eq!(
        login_resp.status(),
        StatusCode::UNAUTHORIZED,
        "SQLi Login Bypass güvenli modda engellenmeli ve 401 Unauthorized dönmelidir!"
    );
}

#[tokio::test]
async fn test_secure_reflected_xss_escaped() {
    let app = common::spawn_app(AppMode::Secure).await;
    let client = Client::new();

    // 🛡️ SECURE REFLECTED XSS PROTECTION:
    // Arama kutusuna gönderilen JavaScript kodu Askama tarafından otomatik escape edilir.
    let payload = "<script>alert(1)</script>";

    let resp = client
        .get(format!("{}/search?q={}", app.address, payload))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.text().await.unwrap();

    // Güvenli modda payload escape edilerek (&lt;script&gt;) yansıtılmalıdır
    assert!(
        !body.contains(payload),
        "Reflected XSS payload'ı güvenli modda ham (raw) olarak render edilmemelidir!"
    );
    assert!(
        body.contains("&lt;script&gt;"),
        "Payload HTML escape edilerek güvenli bir şekilde yansıtılmalıdır!"
    );
}

#[tokio::test]
async fn test_secure_csp_and_security_headers() {
    let app = common::spawn_app(AppMode::Secure).await;
    let client = Client::new();

    let resp = client
        .get(format!("{}/health", app.address))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let headers = resp.headers();

    // 🛡️ SECURE CONFIGURATION (OWASP A06:2026):
    // CSP, Frame-Options, Content-Type nosniff ve Referrer-Policy header'ları mevcut olmalıdır.
    assert!(
        headers.contains_key("content-security-policy"),
        "Content-Security-Policy başlığı eksik!"
    );
    assert_eq!(
        headers.get("x-frame-options").unwrap().to_str().unwrap(),
        "DENY"
    );
    assert_eq!(
        headers
            .get("x-content-type-options")
            .unwrap()
            .to_str()
            .unwrap(),
        "nosniff"
    );
    assert_eq!(
        headers.get("referrer-policy").unwrap().to_str().unwrap(),
        "no-referrer"
    );
}

#[tokio::test]
async fn test_secure_idor_blocked() {
    let app = common::spawn_app(AppMode::Secure).await;
    let client = Client::new();

    // 🛡️ SECURE IDOR MITIGATION:
    // Oturum veya yetki bulunmadığı için doğrudan profile/1 erişimi 401 Unauthorized ile engellenir.
    let resp = client
        .get(format!("{}/profile/1", app.address))
        .send()
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "Güvenli modda yetkisiz profil erişimi engellenmeli ve 401 Unauthorized dönmelidir!"
    );
}

#[tokio::test]
async fn test_secure_debug_endpoint_disabled() {
    let app = common::spawn_app(AppMode::Secure).await;
    let client = Client::new();

    // 🛡️ SECURE MISCONFIGURATION MITIGATION:
    // Debug endpoint'i güvenli modda erişime tamamen kapatılır.
    let resp = client
        .get(format!("{}/api/debug", app.address))
        .send()
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        StatusCode::FORBIDDEN,
        "Güvenli modda debug endpoint'ine erişim yasaklanmalı ve 403 Forbidden dönmelidir!"
    );
}

#[tokio::test]
async fn test_secure_rate_limiting() {
    let app = common::spawn_app(AppMode::Secure).await;
    let client = Client::new();

    // 🛡️ SECURE RATE LIMITING (OWASP A07:2026):
    // Ardı ardına hızlı login istekleri gönderildiğinde hız sınırı aşılır (429 Too Many Requests).
    let login_form = [("username", "admin"), ("password", "wrong_pass")];

    let mut rate_limited = false;

    for _ in 0..10 {
        let resp = client
            .post(format!("{}/login", app.address))
            .form(&login_form)
            .send()
            .await
            .unwrap();

        if resp.status() == StatusCode::TOO_MANY_REQUESTS {
            rate_limited = true;
            break;
        }
    }

    assert!(
        rate_limited,
        "Hızlı brute-force istekleri 429 Too Many Requests ile rate-limit'e takılmalıdır!"
    );
}
