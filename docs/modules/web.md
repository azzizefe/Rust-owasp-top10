# 🌐 Web Modülü Teknik Dokümantasyonu (`crates/web`)

`crates/web`, projenin dış dünyaya açılan HTTP katmanıdır. İstek yönlendirmelerini, oturum yönetimini, HTML arayüzlerinin oluşturulmasını ve OWASP Top 10 zafiyetlerinin güvenlik önlemlerini yöneten middleware yapılarını barındırır.

---

## 📂 Dizin Yapısı ve Modüller

```
crates/web/src/
├── extractors/          # Axum özel veri çıkarıcıları (Session vb.)
├── handlers/            # HTTP İstek işleyicileri (Controller yapısı)
│   ├── auth.rs          # Kayıt, Giriş ve Çıkış işlemleri
│   ├── idor.rs          # Profil okuma ve IDOR zafiyet test alanı
│   ├── ssrf.rs          # SSRF zafiyet test alanı (Ağ seviyesinde filtreleme)
│   ├── xss.rs           # Arama çubuğu üzerinden XSS simülasyonu
│   └── telemetry.rs     # Prometheus metrikleri ve sağlık kontrolleri
├── middleware/          # Güvenlik ve izleme ara katmanları
│   ├── security.rs      # CSP, HSTS, X-Frame-Options başlıkları
│   └── rate_limit.rs    # Hız sınırlama (Rate-Limiting) konfigürasyonu
├── error_response.rs    # Core hatalarının HTTP durum kodlarına eşlenmesi
├── lib.rs               # Modül kütüphane tanımları
├── main.rs              # Uygulama ayağa kaldırma ve Graceful Shutdown
├── routes.rs            # Uç nokta (Endpoint) ve HTTP metot eşlemeleri
└── templates.rs         # Askama şablon (Template) tanımları ve veri bağlama
```

---

## 🛡️ Güvenlik ve Middleware Katmanları

### 1. Özel İstek Çıkarıcıları (`extractors/`)
*   **`SessionExtractor`:** Gelen isteklerdeki oturum çerezini (`Session Cookie`) yakalar, `crates/core` kriptografi modülüyle çözer, HMAC imzasını doğrular ve geçerli kullanıcı bilgisini otomatik olarak handler fonksiyonuna aktarır.

### 2. Güvenlik Başlıkları Middleware (`middleware/security.rs`)
Secure mod aktif olduğunda tarayıcı güvenliğini en üst düzeye çıkarmak için aşağıdaki HTTP yanıt başlıklarını ekler:
*   **Content-Security-Policy (CSP):** `default-src 'self'; script-src 'self' 'nonce-...'; style-src 'self' 'unsafe-inline';` gibi katı kurallarla tarayıcıda yetkisiz kod çalışmasını (XSS) engeller.
*   **Strict-Transport-Security (HSTS):** Bağlantıların yalnızca HTTPS üzerinden kurulacağını tarayıcıya bildirir.
*   **X-Content-Type-Options (nosniff):** Tarayıcıların MIME tipi koklamasını engeller.
*   **X-Frame-Options (DENY):** Clickjacking saldırılarına karşı sayfanın bir `iframe` içinde gösterilmesini engeller.

### 3. Hız Sınırlayıcı (`middleware/rate_limit.rs`)
*   **Tower-Governor** entegrasyonu ile IP adresi bazında hız sınırlaması uygular.
*   Brute-force ve Denial-of-Service (DoS) saldırılarına karşı güvenli modda login/register gibi uç noktaları korur.

### 4. Otomatik Şablon Kaçış (`templates.rs`)
*   **Askama:** HTML şablonlarını derleme zamanında (compile-time) derler. Rust tip sistemiyle doğrudan bütünleşir.
*   Secure modda Askama şablonları, şablon içine yazdırılan tüm dinamik verileri (örneğin kullanıcı aramaları veya kullanıcı adları) otomatik olarak HTML-escape işlemine tabi tutarak Reflected/Stored XSS açıklarını tamamen kapatır.

---

## 📊 Telemetri ve Sağlık İzleme (`handlers/telemetry.rs`)

*   **Prometheus Metrikleri:** Uygulamanın çalışma süresi, aktif HTTP istek sayıları ve veritabanı bağlantı havuzunun anlık durumunu izleyen `/metrics` uç noktası sunar.
*   **Docker Healthcheck:** Docker daemon'un uygulamanın sağlıklı çalışıp çalışmadığını izleyebilmesi için `/health` uç noktası tanımlanmıştır.
