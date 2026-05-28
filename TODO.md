# 🛡️ OWASP Top 10 (2026 Next-Gen) Azaltma Raporu — Ultra-Güvenli Rust Uygulaması

> **Proje türü:** İleri Düzey Güvenlik Portfolyosu — "Zero-Trust Mimari ve Modern Tehdit İstihbaratı Raporu"
> **Hedef:** Kayıt/giriş formu olan bir Rust web uygulaması yaz → En güncel **OWASP 2025/2026** Top 10 zafiyetlerini (Tedarik Zinciri, İstisna Yönetimi dahil) **kontrollü ve izole** şekilde sömür → Rust'ın bellek güvenliği ve tip sistemiyle her birini mimari düzeyde düzelt → "Önce/Sonra" analizi ve uçtan uca otomasyonla kanıtla.
> **Ana teslim:** Mükemmel yapılandırılmış çalışan uygulama (Vulnerable + Secure mod) + Next-Gen Sömürü Kanıtları (PoC) + `owasp-report.md` profesyonel azaltma raporu.
> **Etik sınır:** Tüm zafiyetler izole, etiketli, yalnızca localhost/Docker. Hiçbir gerçek sistem hedeflenmez, hiçbir yıkıcı PoC yazılmaz. Yalnızca savunma amaçlı eğitim!

---

## 📑 İçindekiler

- Faz 0 — Ön Hazırlık & Ortam Kurulumu
- Faz 1 — Workspace, Repo Hijyeni & Sırların Yönetimi
- Faz 2 — Veritabanı Tasarımı & Bağlantı Katmanı
- Faz 3 — Uygulama İskeleti & Mod Anahtarı (vulnerable/secure)
- Faz 4 — ⚠️ Zafiyetli Sürüm (her OWASP kategorisi ayrı)
- Faz 5 — 🎯 Sömürü Kanıtları (PoC)
- Faz 6 — ✅ Güvenli Sürüm (Azaltma)
- Faz 7 — 🐳 Docker & Compose
- Faz 8 — ✅ Test & E2E (vulnerable + secure)
- Faz 9 — 🔄 CI/CD & Bağımlılık Taraması
- Faz 10 — 📊 Azaltma Raporu (ana teslim)
- Faz 11 — ✅ Final Doğrulama
- Ekler — Etik, öğrenme çıktıları, satır bütçesi

---

## 🧭 Mimari, Zero-Trust & Temel Tasarım Kararları (ÖNCE BUNU OKU)

### Klasör yapısı

```
owasp-mitigation-lab/
├── Cargo.toml                    # proje tanımı
├── rust-toolchain.toml
├── .env.example                  # anahtar isimleri (gerçek değer YOK)
├── .gitignore
├── .dockerignore
├── .editorconfig
├── rustfmt.toml
├── Dockerfile
├── docker-compose.yml
├── README.md
├── LICENSE
├── build.rs                      # sqlx offline / template derleme
├── migrations/
│   ├── 0001_create_users.sql
│   ├── 0002_create_sessions.sql
│   └── 0003_create_posts.sql      # stored XSS yüzeyi
├── src/
│   ├── main.rs                   # giriş noktası, router, mod seçimi
│   ├── config.rs                 # env okuma, APP_MODE
│   ├── db.rs                     # PgPool, migration
│   ├── error.rs                  # merkezi hata -> HTTP map
│   ├── models.rs                 # User, Session, Post, formlar
│   ├── session.rs                # oturum üretimi/doğrulama
│   ├── middleware.rs             # güvenlik header, rate limit, CSRF
│   ├── templates.rs              # HTML render köprüsü
│   ├── routes.rs                 # endpoint -> handler eşleme
│   ├── auth/
│   │   ├── mod.rs                # AuthBackend trait (vuln/secure ortak arayüz)
│   │   ├── vulnerable.rs         # ⚠️ AÇIK kod (etiketli, eğitim amaçlı)
│   │   └── secure.rs             # ✅ DÜZELTİLMİŞ kod
│   └── handlers/
│       ├── mod.rs
│       ├── auth_handlers.rs      # register/login/logout
│       ├── user_handlers.rs      # profil, IDOR yüzeyi
│       └── post_handlers.rs      # arama (reflected XSS), yorum (stored XSS)
├── templates/                    # askama .html şablonları
│   ├── base.html
│   ├── register.html
│   ├── login.html
│   ├── profile.html
│   └── search.html
├── static/
│   ├── style.css
│   └── app.js
├── exploits/                     # PoC saldırı scriptleri (zararsız)
│   ├── README.md
│   ├── 01_sqli_login_bypass.sh
│   ├── 02_sqli_union.sh
│   ├── 03_xss_reflected.html
│   ├── 04_xss_stored.sh
│   ├── 05_idor.sh
│   ├── 06_bruteforce.sh
│   └── 07_csrf.html
├── tests/
│   ├── common/mod.rs             # test sunucusu kaldırma yardımcıları
│   ├── vulnerable_mode.rs        # açık tetikleniyor mu (kanıt)
│   └── secure_mode.rs            # düzeltme tutuyor mu (kanıt)
└── docs/
    ├── owasp-report.md           # ⭐ ANA TESLİM
    ├── threat-model.md
    └── screenshots/
        ├── before/
        └── after/
```

### Tasarım Kararı #1 — Tek kod tabanı, iki mod (KRİTİK)

Zafiyetli ve güvenli kod **aynı anda** projede bulunur. Çalışma anında bir ortam değişkeni seçer:

```
APP_MODE=vulnerable   # açıkları gösterir
APP_MODE=secure       # düzeltilmiş davranış
```

Neden böyle:
- [x] Aynı sömürü scriptini iki modda çalıştırıp **kanıt** üretirsin: vulnerable'da SQLi çalışır, secure'da bloklanır
- [x] Zafiyetli kod hiçbir zaman "tek deploy yolu" olmaz → yanlışlıkla canlıya çıkma riski sıfır
- [x] Raporun "önce/sonra" yapısı doğal olarak ortaya çıkar
- [x] E2E testleri her iki modu da assert ederek azaltmayı otomatik doğrular

### Tasarım Kararı #2 — `AuthBackend` trait'i ile soyutlama

```rust
// auth/mod.rs
#[async_trait]
pub trait AuthBackend: Send + Sync {
    async fn login(&self, username: &str, password: &str) -> Result<Session, ApiError>;
    async fn register(&self, form: &RegisterForm) -> Result<UserId, ApiError>;
    async fn find_user(&self, id: i64) -> Result<User, ApiError>;
}
```

- [x] `VulnerableAuth` ve `SecureAuth` aynı trait'i uygular
- [x] `main.rs` `APP_MODE`'a göre birini `Arc<dyn AuthBackend>` olarak enjekte eder
- [x] Handler'lar hangi modda olduğunu **bilmez** → temiz, test edilebilir

### Tasarım Kararı #3 — Stack seçimi

- [x] Dil: **Rust** (öğrenme + güvenlik vurgusu için ideal; bellek güvenliği bonus)
- [x] Web: **axum 0.7** (tower middleware ekosistemi güçlü)
- [x] DB: **PostgreSQL + sqlx 0.7** (compile-time sorgu kontrolü = ekstra güvenlik anlatısı)
- [x] Template: **askama** (derleme zamanı, varsayılan auto-escape = XSS savunması)
- [x] Parola: **argon2** (OWASP önerisi)
- [ ] Not: Prensipler dil-bağımsız; Node/Express+Postgres veya Python/Flask'a uyarlanabilir

### Veri akışı

```
HTTP request
  → middleware (header, rate limit, CSRF, body limit)
  → router (routes.rs)
  → handler (handlers/*)
  → Arc<dyn AuthBackend> (vulnerable VEYA secure)
  → db.rs (PgPool)
  → PostgreSQL
  → template render (auto-escape secure modda)
  → HTTP response (güvenlik header'ları ile)
```

---

## 📋 Faz 0 — Ön Hazırlık & Ortam Kurulumu

### 0.1 — Araç kurulumu
- [x] `rustc --version` ≥ 1.74 doğrula (yoksa `rustup update`)
- [x] `cargo --version` çalışıyor
- [x] `rustup component add clippy rustfmt`
- [x] Docker Desktop / Engine kurulu (`docker --version`)
- [x] Docker Compose v2 (`docker compose version`)
- [x] `cargo install sqlx-cli --no-default-features --features postgres`
- [x] (opsiyonel) `cargo install cargo-audit` — bağımlılık CVE taraması
- [x] (opsiyonel) `cargo install cargo-watch` — geliştirme sırasında auto-reload

### 0.2 — Bilgi ön gereksinimleri (eksikse önce bunları gözden geçir)
- [x] HTTP request/response yaşam döngüsü
- [x] SQL temel sorguları (SELECT/INSERT/WHERE)
- [x] HTML/JS temel (XSS'i anlamak için)
- [x] Async/await temel mantığı (tokio)

### 0.3 — Proje başlangıcı
- [x] Klasör: `mkdir owasp-mitigation-lab && cd $_` (Mevcut flat workspace kullanıldı)
- [x] `git init`
- [x] İlk boş commit: `git commit --allow-empty -m "chore: init project"`
- [x] `rust-toolchain.toml` yaz:
  ```toml
  [toolchain]
  channel = "stable"
  components = ["clippy", "rustfmt"]
  ```

---

## 🗂️ Faz 1 — Proje, Repo Hijyeni & Sırların Yönetimi

### 1.1 — Proje iskeleti
- [x] `cargo init` çalıştır. (Flat workspace iskeleti kuruldu)
- [x] Kök `Cargo.toml` dosyasına bağımlılıkları ekle (Faz 2.1).

### 1.2 — `.gitignore` (sır sızıntısı = OWASP A05, repo hijyeni konunun kendisi)
- [x] `/target`, `**/target`
- [x] `.env`, `.env.local`, `.env.*.local` (**gerçek şifreler ASLA repoda olmaz**)
- [x] `*.log`, `logs/`
- [x] `docs/screenshots/*.tmp`, `*.swp`
- [x] `static/uploads/` (kullanıcı yüklemesi varsa)
- [x] `.DS_Store`, `Thumbs.db`
- [x] `node_modules/` (frontend araçları varsa)
- [x] `*.sqlite`, `*.db` (yerel test db'leri)

### 1.3 — `.env.example` (anahtar isimleri, gerçek değer YOK)
- [x] ```
  # Veritabanı
  DATABASE_URL=postgres://app:CHANGEME@localhost:5432/owasp_lab
  POSTGRES_USER=app
  POSTGRES_PASSWORD=CHANGEME
  POSTGRES_DB=owasp_lab
  # Uygulama
  APP_MODE=secure          # vulnerable | secure
  BIND_ADDR=0.0.0.0:8080
  SESSION_SECRET=CHANGE_ME_64_RANDOM_BYTES
  RUST_LOG=info
  ```
- [x] README'ye not: "Çalıştırmadan önce `.env.example` → `.env` kopyala, değerleri doldur"

### 1.4 — Kalite & düzen dosyaları
- [x] `.editorconfig`: `indent_size=4`, `end_of_line=lf`, `charset=utf-8`, `insert_final_newline=true`, `trim_trailing_whitespace=true`
- [x] `rustfmt.toml`: `max_width = 100`, `edition = "2021"`
- [x] `clippy` lint seviyesi: `src/main.rs` üstüne `#![warn(clippy::unwrap_used, clippy::expect_used)]`

### 1.5 — Lisans & README iskeleti
- [x] `LICENSE` (MIT)
- [x] `README.md` başlıkları: Proje amacı, Mimari, Kurulum, Çalıştırma (iki mod), Sömürü adımları, Azaltma özeti, Etik not
- [x] `.dockerignore`: `target/`, `.git/`, `.env`, `docs/`, `exploits/`, `tests/`

---

## 🗄️ Faz 2 — Veritabanı Tasarımı & Bağlantı Katmanı

### 2.1 — Bağımlılıklar (`Cargo.toml`)
- [x] `axum = "0.7"`
- [x] `tokio = { version = "1", features = ["full"] }`
- [x] `sqlx = { version = "0.7", features = ["runtime-tokio", "postgres", "macros", "chrono"] }`
- [x] `serde = { version = "1", features = ["derive"] }` + `serde_json = "1"`
- [x] `argon2 = "0.5"` — parola hash
- [x] `rand = "0.8"` — kriptografik rastgele (oturum token)
- [x] `askama = "0.12"` + `askama_axum = "0.4"` — template (auto-escape)
- [x] `tower = "0.4"` + `tower-http = { version = "0.5", features = ["limit", "trace", "set-header"] }`
- [x] `tower_governor = "0.3"` — rate limit (secure modda)
- [x] `thiserror = "1"`
- [x] `tracing` + `tracing-subscriber`
- [x] `dotenvy = "0.15"`
- [x] `chrono = { version = "0.4", features = ["serde"] }`
- [x] `ammonia = "4"` — HTML sanitization (secure modda, zengin metin için)
- [x] `async-trait = "0.1"`
- [x] `[dev-dependencies]`: `reqwest = { version = "0.12", features = ["cookies","json"] }`, `tokio` test

### 2.2 — Şema migration'ları

#### `migrations/0001_create_users.sql`
- [x] ```sql
  CREATE TABLE users (
      id            BIGSERIAL PRIMARY KEY,
      username      TEXT NOT NULL UNIQUE,
      password_hash TEXT NOT NULL,        -- argon2, ASLA düz metin
      email         TEXT NOT NULL,
      role          TEXT NOT NULL DEFAULT 'user',
      created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
  );
  ```
- [x] `username` üzerinde UNIQUE constraint (kayıt çakışması)

#### `migrations/0002_create_sessions.sql`
- [x] ```sql
  CREATE TABLE sessions (
      token      TEXT PRIMARY KEY,         -- kriptografik rastgele
      user_id    BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
      expires_at TIMESTAMPTZ NOT NULL,
      created_at TIMESTAMPTZ NOT NULL DEFAULT now()
  );
  ```

#### `migrations/0003_create_posts.sql`
- [x] ```sql
  CREATE TABLE posts (
      id         BIGSERIAL PRIMARY KEY,
      author_id  BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
      content    TEXT NOT NULL,            -- stored XSS yüzeyi
      created_at TIMESTAMPTZ NOT NULL DEFAULT now()
  );
  ```

### 2.3 — Migration uygulama
- [x] Postgres'i Docker'da kaldır (Faz 7'den compose veya hızlı: `docker run --name pg -e POSTGRES_PASSWORD=... -p 5432:5432 -d postgres:16`) (Docker Compose ile Postgres başarıyla kaldırıldı)
- [x] `export DATABASE_URL=...` (.env dosyası üzerinden otomatik yapıldı)
- [x] `sqlx database create` (Veritabanı oluşturuldu)
- [x] `sqlx migrate run` (Migration'lar uygulandı)
- [x] `cargo sqlx prepare` → offline sorgu metadata cache'i (`.sqlx/`), CI/Docker için (Başarıyla çalıştırıldı)

### 2.4 — Bağlantı katmanı (`src/db.rs`)
- [x] `pub async fn connect(url: &str) -> Result<PgPool, sqlx::Error>`
- [x] `PgPoolOptions` ile `max_connections`, `acquire_timeout`
- [x] `pub async fn run_migrations(pool: &PgPool)` → `sqlx::migrate!()`
- [x] Bağlantı hatasında düzgün log + temiz çıkış (panic değil)

### 2.5 — Config (`src/config.rs`)
- [x] ```rust
  pub enum AppMode { Vulnerable, Secure }
  pub struct Config {
      pub database_url: String,
      pub bind_addr: String,
      pub mode: AppMode,
      pub session_secret: String,
  }
  ```
- [x] `Config::from_env()` → `dotenvy::dotenv()` + `std::env::var`
- [x] `APP_MODE` parse; geçersizse default `Secure` (güvenli varsayılan ilkesi)
- [x] Eksik kritik env → açık hata mesajı (panic değil, düzgün exit)

---

## 🏗️ Faz 3 — Uygulama İskeleti & Mod Anahtarı

### 3.1 — Hata tipi (`src/error.rs`)
- [ ] ```rust
  #[derive(thiserror::Error, Debug)]
  pub enum ApiError {
      #[error("bad request")] BadRequest(String),
      #[error("unauthorized")] Unauthorized,
      #[error("forbidden")] Forbidden,
      #[error("not found")] NotFound,
      #[error("too many requests")] RateLimited,
      #[error("internal error")] Internal,   // iç detay ASLA dışarı sızmaz
  }
  ```
- [ ] `impl IntoResponse for ApiError` → her varyant doğru HTTP status + generic JSON gövde
- [ ] **Kritik (A05):** `Internal` varyantı kullanıcıya stack trace / SQL hatası göstermez; detay yalnızca `tracing::error!` ile loglanır
- [ ] `From<sqlx::Error>` impl'i → DB hatasını `Internal`'a çevir, orijinali logla

### 3.2 — Modeller (`src/models.rs`)
- [ ] `#[derive(sqlx::FromRow)] struct User { id, username, password_hash, email, role, created_at }`
- [ ] `struct Session { token, user_id, expires_at }`
- [ ] `struct Post { id, author_id, content, created_at }`
- [ ] Form tipleri (`#[derive(Deserialize)]`):
  - [ ] `RegisterForm { username, password, email }`
  - [ ] `LoginForm { username, password }`
  - [ ] `SearchQuery { q: String }`
  - [ ] `NewPost { content: String }`
- [ ] Validasyon yardımcıları (secure modda kullanılacak): username uzunluk/karakter, email format, password min uzunluk

### 3.3 — Auth trait (`src/auth/mod.rs`)
- [ ] `#[async_trait] pub trait AuthBackend: Send + Sync { ... }` (login/register/find_user/create_post/search_posts)
- [ ] `pub fn build(mode: &AppMode, pool: PgPool) -> Arc<dyn AuthBackend>` fabrika
  - [ ] `Vulnerable` → `Arc::new(VulnerableAuth { pool })`
  - [ ] `Secure` → `Arc::new(SecureAuth { pool })`

### 3.4 — Router & state (`src/main.rs` + `routes.rs`)
- [ ] `#[derive(Clone)] struct AppState { auth: Arc<dyn AuthBackend>, pool: PgPool, mode: AppMode }`
- [ ] `routes.rs`: `Router::new().route(...)` ile tüm endpoint'ler
  - [ ] `GET /` → ana sayfa
  - [ ] `GET /register`, `POST /register`
  - [ ] `GET /login`, `POST /login`, `POST /logout`
  - [ ] `GET /profile/:id` → IDOR yüzeyi
  - [ ] `GET /search` → reflected XSS yüzeyi
  - [ ] `POST /posts`, `GET /posts` → stored XSS yüzeyi
  - [ ] `GET /health` → 200 (test/uptime)
- [ ] `main()`:
  - [ ] tracing init
  - [ ] config yükle
  - [ ] db bağlan + migrate
  - [ ] auth backend seç (mode'a göre)
  - [ ] middleware uygula (secure modda tam, vulnerable modda eksik — bilinçli)
  - [ ] `axum::serve` + graceful shutdown
- [ ] **Başlangıç logunda modu açıkça yaz:** `WARN: running in VULNERABLE mode` (kazara unutma engeli)

### 3.5 — Template köprüsü (`src/templates.rs`)
- [ ] askama struct'ları her sayfa için (`#[derive(Template)] #[template(path="...")]`)
- [ ] **Not:** askama varsayılan auto-escape yapar → secure modda XSS savunması "bedava"
- [ ] Vulnerable modda XSS göstermek için ham HTML enjeksiyon yolu (aşağıda 4.2)


---

## ⚠️ Faz 4 — Zafiyetli Sürüm (Next-Gen OWASP Top 10: 2026 Standartları)

> **Her zafiyetli fonksiyonun başına standart etiket koy:**
> `// ⚠️ VULNERABLE — OWASP AXX:2026 — eğitim amaçlı, ASLA production'da kullanma. Güvenli sürüm: secure.rs`

### 4.1 — A01:2026 → Broken Access Control & SSRF
- [ ] **IDOR (Erişim Kontrolü Kırılması):** `GET /profile/:id` endpoint'i başkasının verisini okumaya (yetkisiz) izin verir.
- [ ] **SSRF (Sunucu Taraflı İstek Sahteciliği):** Kullanıcının URL vererek tetiklediği dış istek, doğrulanmadan `127.0.0.1` veya AWS Metadata IP'sine yönlendirilebilir.

### 4.2 — A02:2026 → Cryptographic Failures
- [ ] **Hassas Veri İfşası:** Kullanıcıların gizli notları veya API anahtarları veritabanında plaintext (şifresiz) tutulur.
- [ ] **Zayıf Kriptografi:** Şifre sıfırlama veya oturum token'ı için kriptografik olmayan `rand::random` kullanımı. HTTP'den HTTPS'e zorlama olmaması (HSTS eksikliği).

### 4.3 — A03:2026 → Software Supply Chain Failures (Yeni Nesil Tehdit)
- [ ] **Zafiyetli Bağımlılık:** Bilinen bir CVE'ye sahip kasıtlı eski bir kütüphane eklenmesi (`cargo audit` ile tespit demosu için).
- [ ] **Build Zehirlenmesi:** Kötü niyetli bir bağımlılığın `build.rs` üzerinden çevre değişkenlerini sızdırma potansiyelinin simülasyonu.

### 4.4 — A04:2026 → Injection (SQLi, XSS)
- [ ] **SQL Injection (Login Bypass):** `VulnerableAuth::login` içinde `format!` makrosuyla kullanıcı girdisi doğrudan SQL string'ine gömülür. Payload: `' OR '1'='1' --`
- [ ] **Cross-Site Scripting:** Arama kutusunda HTML escape edilmeden HTML render edilir (Reflected). Post içerikleri temizlenmeden DB'ye yazılır (Stored XSS).

### 4.5 — A05:2026 → Insecure Design
- [ ] **İş Mantığı Zafiyeti:** Hassas işlemlerde (şifre değiştirme) `re-authentication` (tekrar parola sorma) istenmemesi.
- [ ] **CSRF:** State değiştiren POST'larda (profil güncelleme) CSRF token eksikliği (`07_csrf.html` tetikleyici).

### 4.6 — A06:2026 → Security Misconfiguration
- [ ] **Güvenlik Kalkanları Eksikliği:** `CSP`, `X-Frame-Options`, `X-Content-Type-Options` gibi temel header'ların olmaması.
- [ ] **Açık Yüzeyler:** Debug endpoint'lerinin (`/api/debug`) production'da kasıtlı erişilebilir bırakılması.

### 4.7 — A07:2026 → Identification and Authentication Failures
- [ ] **Zayıf Parola Saklama:** Şifreler düz metin (veya kırılmış `MD5`) olarak tutulur.
- [ ] **Brute-Force & Enumeration:** Rate limit olmaması ve "Kullanıcı yok" ile "Şifre yanlış" mesajlarının farklı verilerek geçerli kullanıcı hesaplarının ifşa edilmesi.

### 4.8 — A08:2026 → Software and Data Integrity Failures
- [ ] **Güvensiz Deserialization:** Kullanıcı yetkisi `{"role":"user"}` şeklinde şifrelenmemiş ve imzasız bir Base64 cookie olarak saklanır (Saldırgan bunu admin yapar).

### 4.9 — A09:2026 → Security Logging and Monitoring Failures
- [ ] **Sessiz İhlaller:** Hatalı giriş denemeleri, yetki ihlalleri (403) hiçbir şekilde loglanmaz. Saldırgan brute-force yaparken sunucu iz bırakmaz.

### 4.10 — A10:2026 → Mishandling of Exceptional Conditions (Yeni!)
- [ ] **Hata Yönetimi Zafiyeti (Fail Open / Crash):** Hatalı veri geldiğinde Rust'ın `.unwrap()` metodu kasten kullanılarak uygulamanın *panic* olmasına (DoS) sebep olunur.
- [ ] **Bilgi Sızıntısı:** Veritabanı hataları (`Err(e)`) doğrudan kullanıcıya HTTP response olarak stack trace / SQL detaylarıyla döner.


---

## 🎯 Faz 5 — Sömürü Kanıtları (`exploits/`)

> **Kural:** Tüm PoC'ler yalnızca KENDİ localhost uygulamana karşı çalışır. Hepsi **zararsızdır** — okuma / `alert()` / bypass gösterimi; veri yıkımı, sızdırma sunucuya gönderme YOK. Her PoC `APP_MODE=vulnerable` ile çalışır, `secure` ile bloklanır.

### 5.1 — `exploits/README.md`
- [ ] Her PoC'nin ne yaptığını, hangi OWASP kategorisine ait olduğunu yaz
- [ ] "Önce vulnerable modda çalıştır, sonra secure modda tekrarla" talimatı
- [ ] Etik uyarı en üstte

### 5.2 — SQLi PoC'leri
- [ ] `01_sqli_login_bypass.sh`:
  ```bash
  curl -s -X POST localhost:8080/login \
    --data-urlencode "username=' OR '1'='1' --" \
    --data-urlencode "password=x" -i
  # Beklenen (vulnerable): 200 + oturum cookie / giriş başarılı
  # Beklenen (secure):     401 unauthorized
  ```
- [ ] `02_sqli_union.sh`: UNION payload ile veri sızdırma denemesi
- [ ] Her ikisinin çıktısını `docs/screenshots/before/` altına al

### 5.3 — XSS PoC'leri
- [ ] `03_xss_reflected.html`: zararsız `alert(1)` tetikleyen link/iframe
- [ ] `04_xss_stored.sh`: post içeriğine `<img src=x onerror=alert(1)>` gönder, sonra `/posts` aç
- [ ] Tarayıcıda alert göründüğü ekran görüntüsü (before)

### 5.4 — IDOR PoC
- [ ] `05_idor.sh`: bir kullanıcıyla giriş yap, başka id'lerin profillerini gez
  ```bash
  for id in 1 2 3 4 5; do curl -s localhost:8080/profile/$id -b cookies.txt; done
  ```

### 5.5 — Brute force PoC
- [ ] `06_bruteforce.sh`: aynı kullanıcıya 50 deneme, hepsinin kabul edildiğini göster (rate limit yok)
- [ ] Secure modda: belirli denemeden sonra 429 dönmeli

### 5.6 — Insecure Design (CSRF & Logic) PoC (A05)
- [ ] `07_csrf.html`: Kurban oturumdayken otomatik POST atan gizli form.

### 5.7 — Bütünlük ve Tedarik Zinciri (A08, A03)
- [ ] `08_integrity.sh`: Base64 kodlanmış çerezi decode et, `role=admin` yap, tekrar encode edip sunucuya yolla (A08).
- [ ] A03: `cargo audit` çıktısında kritik CVE barındıran paketin tespit edildiği ekran görüntüsü.

### 5.8 — Logging & Exception Handling (A09, A10)
- [ ] A09: Brute-force saldırısı esnasında terminal loglarında hiçbir uyarı (audit log) oluşmadığının kanıtı.
- [ ] A10 (DoS & Sızıntı): API'ye hatalı veri göndererek uygulamanın paniklemesi (`unwrap` yüzünden crash) veya SQL detaylarını ifşa ettiği ekran görüntüsü.

### 5.9 — Kanıt toplama (Next-Gen Raporlama)
- [ ] Her PoC için "Before" ekran görüntüsü / terminal çıktısı kaydet.
- [ ] Çıktıları OWASP 2026 kodlarıyla adlandır (ör. `a04-sqli-before.png`, `a10-panic-dos.png`).

---

## ✅ Faz 6 — Güvenli Sürüm (`auth/secure.rs` + middleware) ⭐ PROJENİN KALBİ

> Her düzeltme için: kök neden → uygulanan kontrol → neden işe yaradığı. Bunları rapora birebir taşı.

### 6.1 — A03 SQLi → Parametreli Sorgu (Prepared Statements)
- [ ] `SecureAuth::login` `sqlx::query_as!` makrosu ile:
  ```rust
  // ✅ SECURE — $1/$2 parametreleri girdiyi SQL'den tamamen ayırır
  let user = sqlx::query_as!(
      User,
      "SELECT id, username, password_hash, email, role, created_at \
       FROM users WHERE username = $1",
      username
  ).fetch_optional(&self.pool).await?;
  ```
- [ ] **Neden tutuyor:** parametre veri kanalından gider, parser'a kod olarak ulaşmaz → `' OR '1'='1` artık literal string
- [ ] **Bonus:** `query_as!` makrosu derleme zamanında SQL'i DB şemasına karşı doğrular (yanlış kolon = derleme hatası) → rapora "compile-time güvenlik" notu
- [ ] Tüm dinamik sorguları parametreliye çevir (arama dahil)

### 6.2 — A03 XSS → Escaping + CSP + Sanitization (3 katman)
- [ ] **Katman 1 — Output escaping:** askama template kullan, `format!`+`Html` enjeksiyonunu kaldır → `<` otomatik `&lt;`
- [ ] **Katman 2 — CSP header (middleware):**
  ```
  Content-Security-Policy: default-src 'self'; script-src 'self'; object-src 'none'; base-uri 'self'
  ```
  → inline `<script>` ve `onerror=` çalışmaz
- [ ] **Katman 3 — Input sanitization:** zengin metin gerekiyorsa `ammonia::clean(content)` ile whitelist temizleme (tehlikeli tag/attr atılır)
- [ ] **Neden tutuyor:** reflected/stored payload artık ya escape edilir ya CSP ile bloklanır ya da sanitize edilir → derinlemesine savunma

### 6.3 — A07 → Güçlü Kimlik Doğrulama
- [ ] **Parola hash:** argon2id ile salt'lı hash
  ```rust
  // ✅ SECURE
  let salt = SaltString::generate(&mut OsRng);
  let hash = Argon2::default().hash_password(pw.as_bytes(), &salt)?.to_string();
  ```
- [ ] **Doğrulama:** `Argon2::default().verify_password(...)`
- [ ] **Rate limit:** `tower_governor` ile login endpoint'ine IP başına sınır → brute force 429
- [ ] **Oturum:** 32+ byte `OsRng` token, cookie `HttpOnly; Secure; SameSite=Strict`, makul `expires_at`
- [ ] **Sabit mesaj:** "kullanıcı adı veya parola hatalı" (enumeration engeli)
- [ ] **Timing:** kullanıcı yoksa bile sahte hash doğrulaması yap (timing attack azaltma — bonus)

### 6.4 — A01 → Erişim Kontrolü
- [ ] `GET /profile/:id`: oturumdaki `user_id` ile istenen `id` eşleşmiyorsa **ve** rol admin değilse → 403
  ```rust
  // ✅ SECURE — sahiplik/rol kontrolü
  if session.user_id != path_id && session.role != "admin" {
      return Err(ApiError::Forbidden);
  }
  ```
- [ ] **Mass assignment engeli:** `role` gibi alanlar client formundan ASLA okunmaz; sunucu belirler
- [ ] Yetkilendirme kontrolünü merkezi bir extractor/middleware'e taşı (her endpoint'te tekrar etme)

### 6.5 — A05 → Güvenli Konfigürasyon (`middleware.rs`)
- [ ] **Güvenlik header'ları** (`SetResponseHeaderLayer`):
  - [ ] `X-Content-Type-Options: nosniff`
  - [ ] `X-Frame-Options: DENY`
  - [ ] `Content-Security-Policy: ...` (6.2)
  - [ ] `Referrer-Policy: no-referrer`
  - [ ] `Strict-Transport-Security: max-age=31536000` (HTTPS arkasındaysa)
- [ ] **Generic hata:** `ApiError::Internal` → kullanıcıya "Sunucu hatası", detay sadece log
- [ ] **Body limit:** `RequestBodyLimitLayer` (örn. 64KB) → büyük payload DoS engeli
- [ ] **Timeout:** `TimeoutLayer`
- [ ] **Server banner kapat / minimize et**

### 6.6 — A05 → Insecure Design & İş Mantığı
- [ ] State değiştiren formlara CSRF token entegre et.
- [ ] Kritik işlemler (şifre/email değişimi) için "Re-authentication" (tekrar doğrulama) zorunluluğu ekle.

### 6.7 — A03 & A06 → Tedarik Zinciri Güvenliği ve Misconfig Savunması
- [ ] A03 (Supply Chain): `cargo audit` entegrasyonunu CI pipeline'a zorunlu koş. `build.rs` scriptlerinin ne çalıştırdığını kısıtla.
- [ ] A06: Güvenlik header'larını (CSP, HSTS, NoSniff) Tower middleware ile zırhla. Debug route'ları kapat.

### 6.8 — A02 & A01 → Kriptografi ve Erişim (SSRF) Kalkanları
- [ ] A02: Hassas verileri DB'ye yazarken AES-GCM ile şifrele. Token'lar için `rand::rngs::OsRng` kullan.
- [ ] A01 (SSRF): Dışa istek atan endpoint'lerde URL'yi parse et, Localhost ve Private IP bloklarına erişimi strict şekilde blokla.

### 6.9 — A08 → Yazılım ve Veri Bütünlüğü
- [ ] Cookie tabanlı state yönetimini HMAC imzalı `SignedCookie` (`axum-extra`) ile koru. İmzası tutmayan çerezleri anında reddet (Tampering koruması).

### 6.10 — A09 & A10 → Loglama ve Kusursuz Hata Yönetimi (Rust'ın Gücü)
- [ ] A09: `tracing` ile başarısız login ve 403 hatalarına yapılandırılmış audit loglar bırak (`tracing::warn!("Bruteforce attempt from IP...")`).
- [ ] A10 (Exceptions): ASLA `.unwrap()` veya `.expect()` kullanma. Tüm hataları `Result` ile `ApiError` enum'unda karşıla (Fail Safe). API kullanıcılarına sadece statik "Internal Server Error" dön, orijinal hata detayını güvenli loga yaz.


---

## 🐳 Faz 7 — Docker & Compose

### 7.1 — `Dockerfile` (multi-stage)
- [ ] ```dockerfile
  # Build stage
  FROM rust:1-slim AS builder
  WORKDIR /app
  RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
  COPY . .
  ENV SQLX_OFFLINE=true
  RUN cargo build --release

  # Runtime stage
  FROM debian:bookworm-slim
  RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
  RUN useradd -m -u 10001 appuser
  COPY --from=builder /app/target/release/owasp-mitigation-lab /usr/local/bin/app
  COPY --from=builder /app/migrations /migrations
  USER appuser
  EXPOSE 8080
  ENTRYPOINT ["app"]
  ```
- [ ] `SQLX_OFFLINE=true` → derlemede DB'ye bağlanmadan `.sqlx/` cache kullan
- [ ] Non-root `appuser` (container güvenliği)

### 7.2 — `docker-compose.yml`
- [ ] ```yaml
  services:
    db:
      image: postgres:16
      environment:
        POSTGRES_USER: ${POSTGRES_USER}
        POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
        POSTGRES_DB: ${POSTGRES_DB}
      healthcheck:
        test: ["CMD-SHELL", "pg_isready -U $POSTGRES_USER"]
        interval: 5s
        retries: 5
      # port DIŞARI açılmıyor — sadece internal network (güvenlik)
    app:
      build: .
      depends_on:
        db: { condition: service_healthy }
      environment:
        DATABASE_URL: postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@db:5432/${POSTGRES_DB}
        APP_MODE: ${APP_MODE:-secure}
        BIND_ADDR: 0.0.0.0:8080
      ports:
        - "8080:8080"
  ```
- [ ] Postgres portu host'a açılmıyor (sadece `app` erişir)
- [ ] `APP_MODE` env ile mod seçimi (`APP_MODE=vulnerable docker compose up`)
- [ ] Migration uygulamayı app başlangıcında otomatik çalıştır (`run_migrations`)

### 7.3 — Doğrulama
- [ ] `docker compose up --build` → app + db kalkıyor
- [ ] `curl localhost:8080/health` → 200
- [ ] `docker compose down -v` ile temizlik

---

## ✅ Faz 8 — Test & E2E (vulnerable + secure)

> Bu testler raporun **otomatik kanıtı**. "Önce açık vardı" ve "sonra kapandı" ikisini de assert eder.

### 8.1 — Test altyapısı (`tests/common/mod.rs`)
- [ ] Belirli `APP_MODE` ile test sunucusu kaldıran yardımcı (`spawn_app(mode)`)
- [ ] Her test izole geçici DB / şema (test çakışmasın)
- [ ] `reqwest` client (cookie store açık)

### 8.2 — Birim testleri
- [ ] argon2 hash → verify round-trip
- [ ] `ammonia::clean` `<script>`'i atıyor, düz metni koruyor
- [ ] Validasyon: geçersiz email/username reddediliyor
- [ ] Session token uzunluk/rastgelelik (çakışma yok)

### 8.3 — Vulnerable mod testleri (`tests/vulnerable_mode.rs`)
- [ ] SQLi payload `' OR '1'='1' --` → **giriş başarılı** (açık kanıtlandı)
- [ ] Reflected XSS: `?q=<script>` → yanıt gövdesinde **escape edilmemiş** `<script>` var
- [ ] Stored XSS: post + listele → ham payload dönüyor
- [ ] IDOR: başka id profili **erişilebilir**
- [ ] Hata yanıtında iç detay (SQL/path) sızıyor
- [ ] Brute force: 50 deneme hepsi işleniyor (429 yok)

### 8.4 — Secure mod testleri (`tests/secure_mode.rs`)
- [ ] Aynı SQLi payload → **401** (bloklandı)
- [ ] Reflected XSS payload → yanıtta `&lt;script&gt;` (escape edildi)
- [ ] CSP header **mevcut** (`assert response header contains`)
- [ ] Güvenlik header'ları (nosniff, X-Frame-Options) mevcut
- [ ] Stored XSS payload → sanitize/escape edilmiş
- [ ] IDOR: başka id → **403**
- [ ] Brute force → belirli denemeden sonra **429**
- [ ] Hata yanıtı generic, iç detay YOK
- [ ] Parola DB'de argon2 hash (düz metin değil) — DB satırını kontrol et

### 8.5 — Lint & statik analiz
- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `cargo audit` → bağımlılık CVE
- [ ] (bonus) `cargo deny check` → lisans + ban listesi

---

## 🔄 Faz 9 — CI/CD (`.github/workflows/`)

### 9.1 — `ci.yml`
- [ ] Trigger: `push`, `pull_request`
- [ ] Postgres servis container'ı:
  ```yaml
  services:
    postgres:
      image: postgres:16
      env: { POSTGRES_PASSWORD: postgres, POSTGRES_DB: owasp_lab }
      ports: ["5432:5432"]
      options: >-
        --health-cmd pg_isready --health-interval 5s --health-retries 5
  ```
- [ ] Adımlar:
  - [ ] `actions/checkout@v4`
  - [ ] `dtolnay/rust-toolchain@stable` (clippy+rustfmt)
  - [ ] `Swatinem/rust-cache@v2`
  - [ ] `sqlx migrate run` (DATABASE_URL env ile)
  - [ ] `cargo fmt --all --check`
  - [ ] `cargo clippy --all-targets -- -D warnings`
  - [ ] `cargo test` (vulnerable + secure e2e ikisi de)
  - [ ] `cargo build --release`

### 9.2 — `audit.yml` (zamanlanmış güvenlik taraması)
- [ ] `cargo install cargo-audit`
- [ ] `cargo audit` → CVE bulursa fail
- [ ] (opsiyonel) haftalık `schedule` cron

### 9.3 — (opsiyonel) Docker build job
- [ ] İmajı build et, smoke test (`/health` 200)

---

## 📊 Faz 10 — Azaltma Raporu (`docs/owasp-report.md`) ⭐ ANA TESLİM

> Jüri/işveren öncelikle bunu okur. Tutarlı şablon = profesyonel izlenim.

### 10.1 — Rapor yapısı
- [ ] **Yönetici özeti:** kaç zafiyet, hangi kategoriler, sonuç (hepsi kapatıldı)
- [ ] **Kapsam & metodoloji:** nasıl test edildi (manuel PoC + otomatik e2e), ortam (izole localhost/Docker)
- [ ] **Özet tablo (OWASP 2026 Standartları):**
  | OWASP | Zafiyet Adı | Konum | Sömürü (PoC) | Mimari Düzeltme | Doğrulama | Durum |
  |---|---|---|---|---|---|---|
  | A01 | Broken Access (IDOR+SSRF) | /profile & /fetch | Başka ID / SSRF | Sahiplik kontrolü + IP Whitelist | 403 test | ✅ |
  | A02 | Cryptographic Failures | DB / Token | Plaintext / Zayıf RNG | AES-GCM şifreleme + OsRng | Şifreli DB test | ✅ |
  | A03 | Supply Chain Failures | Cargo.toml | Bilinen CVE sömürüsü | `cargo audit` CI otomasyonu | CI Audit Pass | ✅ |
  | A04 | Injection (SQLi & XSS) | auth / arama | `' OR '1'='1` / `<script>` | Prepared stmt ($1) + Escaping | 401 & HTML Assert | ✅ |
  | A05 | Insecure Design | POST formları | CSRF / Onaysız işlem | CSRF Token + Re-auth | 403 test | ✅ |
  | A06 | Security Misconfig | error.rs / header | Güvenlik kalkanı yok | Tower Header Middleware | Header assert | ✅ |
  | A07 | Auth Failures | login | Brute-force | argon2id + rate limit | 429 test | ✅ |
  | A08 | Veri Bütünlüğü (Integ.) | Cookie parsing | Base64 değiştirme | HMAC Signed Cookie | İmza İhlali 400 | ✅ |
  | A09 | Security Logging | handler / auth | Logsuz saldırı | `tracing::warn!` Audit Log | Log grep test | ✅ |
  | A10 | Exceptional Conditions | Tüm handler'lar | `.unwrap()` DoS / Hata Sızdırma | Güvenli `ApiError` Error Mapping | DoS koruması | ✅ |

### 10.2 — Her zafiyet için detay bölümü (aynı şablon)
- [ ] **Başlık:** OWASP kategorisi + zafiyet adı
- [ ] **Konum:** dosya + satır + endpoint
- [ ] **Açıklama:** zafiyet neden var, kök neden
- [ ] **Sömürü adımları:** birebir komut/payload (PoC referansı)
- [ ] **Ekran görüntüsü (ÖNCE):** `screenshots/before/aXX-...png` — sömürü başarılı
- [ ] **Etki:** ne elde edilebilir (auth bypass, veri okuma, çerez çalma...)
- [ ] **Düzeltme — kod karşılaştırması (yan yana diff):**
  ```diff
  - let q = format!("... WHERE username = '{}'", username);
  - sqlx::query(&q)...
  + sqlx::query_as!(User, "... WHERE username = $1", username)
  ```
- [ ] **Neden işe yarıyor:** kontrolün mekanizması
- [ ] **Ekran görüntüsü (SONRA):** `screenshots/after/...` — aynı payload bloklanıyor
- [ ] **Doğrulama:** ilgili otomatik test adı + sonucu

### 10.3 — Tehdit modeli (`docs/threat-model.md`)
- [ ] Saldırı yüzeyi tablosu (her endpoint + kabul ettiği girdi + risk)
- [ ] STRIDE kategorizasyonu (Spoofing/Tampering/Repudiation/Info disclosure/DoS/Elevation)
- [ ] Güven sınırları diyagramı (kullanıcı → app → db)

### 10.4 — Ekran görüntüsü disiplini
- [ ] Tutarlı isimlendirme: `aXX-zafiyet-before.png` / `-after.png`
- [ ] Hassas gerçek veri olmasın (sahte test kullanıcıları)
- [ ] Her görüntüde ne gösterdiğini kısa alt yazı

### 10.5 — README final
- [ ] Mimari diyagram (vulnerable/secure mod akışı)
- [ ] "Nasıl çalıştırılır" (iki mod)
- [ ] "Nasıl sömürülür" (exploits/ rehberi)
- [ ] "Nasıl düzeltildi" özeti
- [ ] Demo GIF / asciinema (SQLi bypass before → secure'da 401 after 🔥)
- [ ] Etik not (en görünür yerde)


---

## ✅ Faz 11 — Final Doğrulama (teslim öncesi son kontrol)

### 11.1 — Fonksiyonel
- [ ] `cp .env.example .env` + değerleri doldur
- [ ] `docker compose up --build` → app + Postgres sağlıklı
- [ ] `curl localhost:8080/health` → 200
- [ ] Kayıt → giriş → profil → post akışı uçtan uca çalışıyor (secure modda)

### 11.2 — Açık kanıtı (vulnerable mod)
- [ ] `APP_MODE=vulnerable` ile kalk
- [ ] `01_sqli_login_bypass.sh` → giriş başarılı
- [ ] `03_xss_reflected.html` → alert görünüyor
- [ ] `05_idor.sh` → başka profiller erişilebilir
- [ ] `06_bruteforce.sh` → 429 yok, log kaydı sıfır (A09)
- [ ] `08_integrity.sh` → Base64 oynamasıyla yetki yükseltildi (A08)
- [ ] API'ye yanlış tip verilerek uygulamanın paniklemesi / SQL detay sızdırması kanıtlandı (A10)
- [ ] Before ekran görüntüleri alındı

### 11.3 — Azaltma kanıtı (secure mod)
- [ ] `APP_MODE=secure` ile kalk
- [ ] Aynı SQLi → 401
- [ ] Aynı XSS → escape edilmiş / CSP blokladı
- [ ] Aynı IDOR → 403
- [ ] Brute force → 429
- [ ] Hata yanıtları generic
- [ ] After ekran görüntüleri alındı

### 11.4 — Kalite & güvenlik
- [ ] `cargo fmt --all --check` temiz
- [ ] `cargo clippy --all-targets -- -D warnings` uyarısız
- [ ] `cargo test` → vulnerable + secure tüm e2e yeşil
- [ ] `cargo audit` → bilinen CVE yok
- [ ] CI yeşil

### 11.5 — Hijyen & teslim
- [ ] `.env` repoda **değil**, `.env.example` var
- [ ] Hiçbir gerçek sır/şifre commit edilmemiş (`git log -p | grep -i password` kontrol)
- [ ] `owasp-report.md` tüm bölümler dolu, before/after görüntüleri bağlı
- [ ] `threat-model.md` tamam
- [ ] README + diyagram + demo hazır
- [ ] `git tag v1.0.0`

---

## ⚠️ Etik & Yasal Not (mutlaka oku ve rapora ekle)

- [ ] Zafiyetli kod **yalnızca izole localhost/Docker** ortamında çalışır; internete açık deploy **edilmez**.
- [ ] Tüm sömürü PoC'leri **kendi uygulamana** karşıdır. Sahip olmadığın/izin almadığın bir sisteme SQLi/XSS/brute-force denemek **birçok ülkede suçtur** (ör. yetkisiz erişim yasaları).
- [ ] PoC'ler **zararsızdır:** `alert()`, okuma, bypass gösterimi — veri yıkımı, dışarı sızdırma, kalıcı zarar **yok**.
- [ ] `vulnerable.rs` açıkça etiketlidir ve secure sürümle birlikte tutulur; tek başına deploy edilebilir tek yol değildir.
- [ ] Bu proje **savunma odaklıdır:** amaç kendi kodunu güvenli yazmayı öğrenmek ve azaltmaları belgelemektir.

---

## 💡 Öğrenme Çıktıları (CV/portfolyoda vurgula)

- [ ] En güncel **OWASP 2026/2025** Top 10 standartlarını gerçek bir projede tatbik etme
- [ ] Zero-Trust mimari ve "Secure by Design" (Tasarım Aşamasında Güvenlik) prensipleri
- [ ] Tedarik zinciri güvenliği (`cargo audit`) ve Build süreci zehirlenmelerine karşı önlemler (A03)
- [ ] Rust'ın güçlü tip sistemi ve error handling özellikleri ile "Exceptional Conditions" (A10) zaaflarını (panic, DoS) yok etme
- [ ] Parametreli sorgu / prepared statement ile modern SQLi engelleme
- [ ] Derinlemesine savunma: Output escaping + CSP + Signed Cookies + HSTS
- [ ] "Zafiyeti sömür → Rust ile çöz → E2E otomasyonla kanıtla" güvenlik döngüsü
- [ ] Profesyonel ve vizyoner bir güvenlik araştırma raporu yazımı

---

## 📐 Tahmini Çaba Dağılımı (planlama için)

| Faz | İçerik | Tahmini efor |
|---|---|---:|
| 0-1 | Ortam + workspace + hijyen | %5 |
| 2-3 | DB + iskelet + mod anahtarı | %15 |
| 4 | Zafiyetli sürüm (tüm kategoriler) | %15 |
| 5 | PoC scriptleri + before görüntüleri | %10 |
| 6 | Güvenli sürüm (azaltmalar) | %25 |
| 7 | Docker + compose | %5 |
| 8-9 | Test + E2E + CI | %15 |
| 10 | Azaltma raporu (ana teslim) | %10 |

> **İpucu:** Önce TEK bir zafiyeti uçtan uca tamamla (SQLi: zafiyetli → PoC → düzeltme → test → rapor bölümü). Bu dikey dilim çalışınca, diğer kategorileri aynı şablonla hızla çoğaltırsın.

---

## 🧩 Ek A — Hızlı Başlangıç Dosya İskeletleri (kopyala-uyarla)

> Bu iskeletler "boş sayfa" sorununu çözer. Doldurman gereken yerler `// TODO` ile işaretli.

### A.1 — `src/main.rs` iskeleti
- [ ] ```rust
  mod config; mod db; mod error; mod models; mod session;
  mod middleware; mod templates; mod routes;
  mod auth; mod handlers;

  use std::sync::Arc;

  #[tokio::main]
  async fn main() -> anyhow::Result<()> {
      tracing_subscriber::fmt::init();
      let cfg = config::Config::from_env()?;
      let pool = db::connect(&cfg.database_url).await?;
      db::run_migrations(&pool).await?;

      if matches!(cfg.mode, config::AppMode::Vulnerable) {
          tracing::warn!("⚠️  RUNNING IN VULNERABLE MODE — localhost only");
      }

      let auth = auth::build(&cfg.mode, pool.clone());
      let state = routes::AppState { auth, pool, mode: cfg.mode };
      let app = routes::router(state, &cfg);  // middleware mode'a göre

      let listener = tokio::net::TcpListener::bind(&cfg.bind_addr).await?;
      tracing::info!("listening on {}", cfg.bind_addr);
      axum::serve(listener, app).await?;
      Ok(())
  }
  ```

### A.2 — `auth/mod.rs` trait iskeleti
- [ ] ```rust
  use async_trait::async_trait;
  use crate::{models::*, error::ApiError, config::AppMode};
  use std::sync::Arc;
  use sqlx::PgPool;

  #[async_trait]
  pub trait AuthBackend: Send + Sync {
      async fn login(&self, u: &str, p: &str) -> Result<Session, ApiError>;
      async fn register(&self, f: &RegisterForm) -> Result<i64, ApiError>;
      async fn find_user(&self, id: i64) -> Result<User, ApiError>;
  }

  mod vulnerable; mod secure;
  pub use vulnerable::VulnerableAuth;
  pub use secure::SecureAuth;

  pub fn build(mode: &AppMode, pool: PgPool) -> Arc<dyn AuthBackend> {
      match mode {
          AppMode::Vulnerable => Arc::new(VulnerableAuth { pool }),
          AppMode::Secure => Arc::new(SecureAuth { pool }),
      }
  }
  ```

### A.3 — `error.rs` IntoResponse iskeleti
- [ ] ```rust
  use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
  use serde_json::json;

  impl IntoResponse for ApiError {
      fn into_response(self) -> Response {
          let (status, msg) = match &self {
              ApiError::BadRequest(_) => (StatusCode::BAD_REQUEST, "bad request"),
              ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized"),
              ApiError::Forbidden => (StatusCode::FORBIDDEN, "forbidden"),
              ApiError::NotFound => (StatusCode::NOT_FOUND, "not found"),
              ApiError::RateLimited => (StatusCode::TOO_MANY_REQUESTS, "too many requests"),
              ApiError::Internal => {
                  // iç detay yalnızca log'a — kullanıcıya ASLA
                  tracing::error!(error = ?self, "internal error");
                  (StatusCode::INTERNAL_SERVER_ERROR, "internal error")
              }
          };
          (status, Json(json!({ "error": msg }))).into_response()
      }
  }
  ```

### A.4 — `middleware.rs` güvenlik header iskeleti (secure mod)
- [ ] ```rust
  use tower_http::set_header::SetResponseHeaderLayer;
  use axum::http::{HeaderValue, header};

  pub fn security_headers() -> Vec<SetResponseHeaderLayer<HeaderValue>> {
      vec![
          SetResponseHeaderLayer::overriding(
              header::HeaderName::from_static("x-content-type-options"),
              HeaderValue::from_static("nosniff")),
          SetResponseHeaderLayer::overriding(
              header::HeaderName::from_static("x-frame-options"),
              HeaderValue::from_static("DENY")),
          SetResponseHeaderLayer::overriding(
              header::HeaderName::from_static("content-security-policy"),
              HeaderValue::from_static(
                  "default-src 'self'; script-src 'self'; object-src 'none'; base-uri 'self'")),
      ]
  }
  ```

---

## 🪤 Ek B — Sık Karşılaşılan Tuzaklar & Çözümleri

- [ ] **`sqlx::query!` derlemede DB istiyor:** Çözüm → `cargo sqlx prepare` ile `.sqlx/` cache üret, `SQLX_OFFLINE=true` set et (CI/Docker için şart).
- [ ] **askama escape'i fazla agresif geldi:** Zengin metin için `ammonia` ile sanitize edip `| safe` filtresiyle bas — ama YALNIZCA sanitize sonrası.
- [ ] **CSP her şeyi kırıyor:** Önce `Content-Security-Policy-Report-Only` ile test et, ihlalleri gör, sonra zorunlu hale getir.
- [ ] **Rate limit testte de tetikleniyor:** Test ortamında limiti yükselt veya IP'yi değiştir; secure_mode brute-force testinde bilinçli düşür.
- [ ] **Cookie `Secure` yerelde çalışmıyor:** HTTPS olmayan localhost'ta `Secure` flag cookie'yi engelleyebilir; geliştirmede ayrı, üretimde açık tut (env ile).
- [ ] **Postgres bağlantısı CI'da hazır değil:** `pg_isready` healthcheck + `depends_on: condition: service_healthy` kullan.
- [ ] **Vulnerable kod yanlışlıkla clippy'yi bozuyor:** `vulnerable.rs` üstüne `#[allow(clippy::all)]` + açık yorum koy; bilinçli kötü kod olduğunu belirt.
- [ ] **argon2 yavaş, testleri uzatıyor:** Testlerde düşük cost parametresi kullan (üretimde varsayılan/yüksek).
- [ ] **IDOR düzeltmesini her handler'da tekrar yazıyorsun:** Oturum + sahiplik kontrolünü bir axum extractor'a taşı, DRY kal.

---

## 📊 Ek C — Demo Senaryosu (sunum/jüri için 3 dakikalık akış)

- [ ] **0:00** — "İşte kayıt/giriş uygulamam, iki modda çalışıyor."
- [ ] **0:20** — `APP_MODE=vulnerable`: login'e `' OR '1'='1' --` yaz → **giriş açıldı** (ekranda göster)
- [ ] **0:50** — arama kutusuna `<script>alert('XSS')</script>` → **alert patladı**
- [ ] **1:20** — `/profile/2`, `/profile/3` gez → **başkasının verisi** (IDOR)
- [ ] **1:50** — "Şimdi tek satır değiştirmeden, sadece modu değiştiriyorum." → `APP_MODE=secure`
- [ ] **2:10** — Aynı SQLi → **401**. Aynı XSS → **escape edilmiş metin**. Aynı IDOR → **403**.
- [ ] **2:40** — "Ve hepsi otomatik testlerle kanıtlı:" `cargo test` → yeşil
- [ ] **3:00** — `owasp-report.md` önce/sonra tablosunu göster → kapanış

> Bu senaryo, "anladığını" değil "uygulayıp kanıtladığını" gösterir — değerlendirmede en yüksek puanı bu getirir.