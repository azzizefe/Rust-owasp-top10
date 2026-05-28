# 🎯 Güvenlik Tehdit Modeli (Threat Model)

Bu tehdit modeli, **Rust-owasp-top10** laboratuvarının saldırı yüzeylerini, STRIDE tehdit analizini ve Zero-Trust mimarisinin çevre sınırlarını detaylandırır.

---

## 1. Saldırı Yüzeyi Tablosu

| Endpoint | HTTP Metodu | Kabul Edilen Girdi | Potansiyel Risk / Zafiyet | Azaltma / Güvenlik Kontrolü |
| :--- | :--- | :--- | :--- | :--- |
| `/register` | `POST` | `RegisterForm` (username, email, password) | Zayıf parola, XSS girdisi, plain-text kayıt | Güçlü Argon2id hashing, Regex girdi doğrulaması |
| `/login` | `POST` | `LoginForm` (username, password) | SQL Injection, timing timing attack, brute-force | Parametreli sqlx sorgusu, timing delay, Tower rate-limiting |
| `/search` | `GET` | `q` (arama metni query) | SQL Injection, Reflected XSS | Prepared Statements, Askama otomatik HTML escaping |
| `/post` | `POST` | `content` (gönderi içeriği) | Stored XSS, yetkisiz post paylaşımı | Ammonia HTML temizleyicisi, oturum yetkilendirme doğrulaması |
| `/profile/:id` | `GET` | Path parametresi (`id`) | IDOR (Broken Access Control) | Sahiplik kontrolü (`current_user.id == profile_id` veya `admin` rolü) |
| `/fetch_url` | `GET` | `url` (Fetch edilecek adres) | SSRF (İç ağ taraması), unwrap() ile DoS | İç ağ IP engelleme listesi (Blacklisting), Timeout kontrolü, Result hata yakalama |
| `/api/debug` | `GET` | Yok | Bilgi sızıntısı (Güvenlik Yapılandırma Hatası) | Secure modda endpoint'in tamamen kapatılması ve 403 dönülmesi |

---

## 2. STRIDE Tehdit Analizi

### 👥 Spoofing (Kimlik Taklidi)
* **Açıklama:** Saldırganın, oturumu doğrulanmış başka bir kullanıcının kimliğini ele geçirerek işlem yapması.
* **Azaltma:**
  * Güvenli modda rastgele, yüksek entropili kriptografik oturum token'ları oluşturularak veritabanında saklanır.
  * Oturum çerezleri `HttpOnly` (JS erişimini engeller), `Secure` (sadece HTTPS üzerinden iletimi sağlar) ve `SameSite=Strict` (CSRF saldırılarını önler) bayraklarıyla set edilir.

### ✍️ Tampering (Veri Manipülasyonu)
* **Açıklama:** Saldırganın istemci ile sunucu arasında giden oturum çerezlerini veya parametreleri değiştirerek yetkisiz işlemleri tetiklemesi.
* **Azaltma:**
  * Zafiyetli modda düz metin base64 olarak saklanan oturum verisi, güvenli modda veritabanı kontrollü session token ile ikame edilir.
  * Gelen tüm form ve path girdileri güçlü validasyon kurallarına tabi tutulur.

### 🚫 Repudiation (İnkar Edilemezlik)
* **Açıklama:** Saldırganın gerçekleştirdiği zararlı eylemlerin kanıtlanamaması veya kayıt altına alınamaması.
* **Azaltma:**
  * `tracing` entegrasyonuyla kritik başarısız login girişimleri, şüpheli brute-force hareketleri ve yetkisiz profile erişim denemeleri IP bilgisiyle loglanır (`tracing::warn!`).

### ℹ️ Information Disclosure (Bilgi Sızıntısı)
* **Açıklama:** Veritabanı şifrelerinin, iç dizin yollarının veya veritabanı hata mesajlarının doğrudan API kullanıcısına sızması.
* **Azaltma:**
  * `/api/debug` endpoint'i secure modda tamamen kapatılır.
  * Axum hata işleyicisi (Error mapper) tüm iç hataları (`sqlx::Error` vb.) yutup, kullanıcıya yalnızca statik ve generic güvenli mesajlar döner.

### ☄️ Denial of Service (Servis Dışı Bırakma - DoS)
* **Açıklama:** Aşırı büyük istek boyutları veya panik oluşturan girdilerle Rust web sunucusunun çökertilmesi ya da kaynaklarının tüketilmesi.
* **Azaltma:**
  * `RequestBodyLimitLayer` ile istek gövdesi maksimum **64KB** ile sınırlandırılmıştır.
  * Kod tabanında `.unwrap()` ve `.expect()` kullanımları tamamen elenerek yerine Rust'ın `?` hata yayma operatörü kullanılmıştır (Fail-Safe).

### 👑 Elevation of Privilege (Yetki Yükseltme)
* **Açıklama:** Normal bir kullanıcının `admin` yetkilerini ele geçirerek yönetimsel işlemleri yapabilmesi.
* **Azaltma:**
  * Profil görüntüleme ve kritik işlemlerde strictly-defined rol tabanlı kontroller mevcuttur (`user.role == "admin"`).

---

## 3. Zero-Trust Güven Sınırları Diyagramı

Aşağıdaki diyagram, uygulamanın Zero-Trust mimarisini ve veri akışlarının güven sınırlarını göstermektedir:

```mermaid
graph TD
    subgraph Web_Browser [Güvensiz Bölge (Client)]
        User[Kullanıcı / Saldırgan]
    end

    subgraph Security_Boundary [Güven Sınırı Middleware]
        CSP[CSP / Security Headers]
        RateLimit[Rate Limiting - Governor]
        CookieParser[Cookie Integrity & Signature]
    end

    subgraph App_Server [İç Uygulama Çevresi (Axum)]
        Router[Axum Routes]
        Auth[Auth Service - SecureAuth]
        Sanitizer[Ammonia Sanitizer]
        ErrorMapper[Fail-Safe Error Handler]
    end

    subgraph DB_Zone [Veritabanı Çevresi (PostgreSQL)]
        DB[(PostgreSQL - parameterized)]
    end

    User -->|HTTP Requests| CSP
    CSP -->|IP & Rate Check| RateLimit
    RateLimit -->|Session Cookie Validation| CookieParser
    CookieParser -->|Sanitized Input| Router
    Router -->|Input Validation| Sanitizer
    Sanitizer -->|Secure Async Task| Auth
    Auth -->|Parameterized Query| DB
    ErrorMapper -.->|Generic Safe Error| User
```
