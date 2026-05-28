// tests/vulnerable_mode.rs

mod common;

use reqwest::{Client, StatusCode};
use rust_owasp_top10::config::AppMode;
use rust_owasp_top10::models::RegisterForm;

#[tokio::test]
async fn test_vulnerable_sqli_login_bypass() {
    let app = common::spawn_app(AppMode::Vulnerable).await;
    let client = Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();

    // 1. Önce test verisi için rastgele bir kullanıcı oluşturuyoruz (başarılı register olabilmesi için)
    let register_form = RegisterForm {
        username: "testuser_sqli".to_string(),
        password: "password123".to_string(),
        email: "sqli@test.com".to_string(),
    };
    
    let resp = client
        .post(&format!("{}/register", app.address))
        .form(&register_form)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK); // Askama template rendering / register success

    // 2. ⚠️ VULNERABLE SQL INJECTION LOGIN BYPASS:
    // Kullanıcı adı alanına SQLi payload'ı gönderiyoruz. Parola tamamen uydurma.
    let sql_bypass_form = [
        ("username", "' OR '1'='1' --"),
        ("password", "any_wrong_password"),
    ];

    let login_resp = client
        .post(&format!("{}/login", app.address))
        .form(&sql_bypass_form)
        .send()
        .await
        .unwrap();

    // Zafiyetli modda SQLi login bypass başarılı olur ve bizi `/search` sayfasına yönlendirir (HTTP 303 veya HTTP 200 ile cookie set eder)
    assert!(
        login_resp.status() == StatusCode::OK || login_resp.status() == StatusCode::SEE_OTHER,
        "SQLi Login Bypass zafiyetli modda çalışmalıdır!"
    );

    // Çerez başlığında base64 formatlı 'user_session' olup olmadığını kontrol ediyoruz
    let headers = login_resp.headers();
    if let Some(cookie) = headers.get(reqwest::header::SET_COOKIE) {
        let cookie_str = cookie.to_str().unwrap();
        assert!(cookie_str.contains("user_session="), "İmzasız base64 çerezi set edilmelidir!");
    }
}

#[tokio::test]
async fn test_vulnerable_reflected_xss() {
    let app = common::spawn_app(AppMode::Vulnerable).await;
    let client = Client::new();

    // ⚠️ VULNERABLE REFLECTED XSS:
    // Arama kutusuna gönderilen JavaScript kodu escape edilmeden gövdeye yansıtılır.
    // SQLi zafiyeti bulunan sorgunun kırılmaması için tek tırnak içermeyen bir payload seçiyoruz.
    let payload = "<script>alert(1)</script>";
    
    let resp = client
        .get(&format!("{}/search?q={}", app.address, payload))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.text().await.unwrap();
    
    // Zafiyetli modda arama girdisi auto-escape olmadan yansır
    assert!(
        body.contains(payload),
        "Reflected XSS payload'ı zafiyetli modda kaçış (escape) karakteri olmadan dönmelidir!"
    );
}

#[tokio::test]
async fn test_vulnerable_idor() {
    let app = common::spawn_app(AppMode::Vulnerable).await;
    let client = Client::new();

    // IDOR zafiyetinin tetiklenebilmesi için veritabanında en az bir profil olmalıdır.
    // Önce bir kullanıcı kaydediyoruz
    let register_form = RegisterForm {
        username: "testuser_idor".to_string(),
        password: "password123".to_string(),
        email: "idor@test.com".to_string(),
    };
    let reg_resp = client
        .post(&format!("{}/register", app.address))
        .form(&register_form)
        .send()
        .await
        .unwrap();
    assert_eq!(reg_resp.status(), StatusCode::OK);

    // ⚠️ VULNERABLE IDOR:
    // Herhangi bir oturum veya yetkilendirme olmaksızın doğrudan profile/1 veya profile/2 adresine istek atılabilir.
    let resp = client
        .get(&format!("{}/profile/1", app.address))
        .send()
        .await
        .unwrap();

    // Zafiyetli modda yetkisiz profil istekleri engellenmez ve 200 OK döner.
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "IDOR zafiyeti nedeniyle yetkisiz profil erişimi başarılı olmalıdır!"
    );
}

#[tokio::test]
async fn test_vulnerable_debug_leak() {
    let app = common::spawn_app(AppMode::Vulnerable).await;
    let client = Client::new();

    // ⚠️ VULNERABLE GÜVENLİK YAPILANDIRMA HATASI (Security Misconfiguration):
    // Debug endpoint'i dışarıya açıktır ve tüm gizli anahtarları sızdırır.
    let resp = client
        .get(&format!("{}/api/debug", app.address))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let json_data: serde_json::Value = resp.json().await.unwrap();
    
    // JSON içerisinde veritabanı şifresi veya session_secret olup olmadığını kontrol ediyoruz
    assert_eq!(json_data["status"], "vulnerable_debug_active");
    assert!(json_data["database_url"].as_str().is_some());
    assert!(json_data["session_secret"].as_str().is_some());
}
