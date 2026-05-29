# 🏗️ Senior Engineering Roadmap — Rust OWASP Top 10 Lab

> **Hedef:** Mevcut monolitik yapıyı, kurumsal düzeyde bir Staff/Senior Engineer portfolyo projesine dönüştürmek.
> **Kural:** Her faz bağımsız olarak commit edilebilir. Hiçbir faz diğerini kırmaz.
> **Mevcut Durum:** Tüm 10 OWASP kategorisi çalışıyor, 10 E2E test yeşil, CI/CD aktif.

---

## Faz 1 — Cargo Workspace (Modüler Mimari)

> **Neden:** Tek crate monoliti → katmanlı, derlenme süresi düşük, sorumlulukları net ayrılmış çoklu paket mimarisi.

### 1.1 — Workspace Kök Yapısı
- [x] Kök `Cargo.toml`'u `[workspace]` tanımına çevir (`members = ["crates/core", "crates/web"]`)
- [x] `resolver = "2"` ekle (Rust 2021 workspace normu)
- [x] Ortak bağımlılıkları `[workspace.dependencies]` altında tanımla (sqlx, serde, chrono, thiserror, tracing)

### 1.2 — `crates/core` Paketi (Sıfır HTTP Bağımlılığı)
- [x] `crates/core/Cargo.toml` oluştur — sadece `sqlx`, `serde`, `argon2`, `chrono`, `thiserror`, `async-trait`
- [x] `models.rs` → `crates/core/src/models.rs` taşı
- [x] `error.rs` → `crates/core/src/error.rs` taşı (HTTP `IntoResponse` kısmını çıkar, sadece domain error)
- [x] `db.rs` → `crates/core/src/db.rs` taşı
- [x] `config.rs` → `crates/core/src/config.rs` taşı
- [x] `session.rs` → `crates/core/src/session.rs` taşı
- [x] `auth/mod.rs`, `auth/vulnerable.rs`, `auth/secure.rs` → `crates/core/src/auth/` taşı
- [x] `crates/core/src/lib.rs` yaz (pub mod tanımları)
- [x] `cargo build -p core` başarılı derleniyor

### 1.3 — `crates/web` Paketi (HTTP Katmanı)
- [x] `crates/web/Cargo.toml` oluştur — `core` + `axum`, `tower`, `tower-http`, `askama`, `tower_governor`
- [x] `routes.rs` → `crates/web/src/routes.rs` taşı
- [x] `handlers/` → `crates/web/src/handlers/` taşı
- [x] `templates.rs` → `crates/web/src/templates.rs` taşı
- [x] `middleware.rs` → `crates/web/src/middleware.rs` taşı
- [x] `main.rs` → `crates/web/src/main.rs` taşı
- [x] HTTP'ye özgü `IntoResponse for ApiError` impl'ini `crates/web/src/error_response.rs`'e taşı
- [x] `cargo build -p web` başarılı derleniyor

### 1.4 — Entegrasyon & Doğrulama
- [x] `templates/` ve `static/` dizinlerini `crates/web/` altına taşı
- [x] `migrations/` kök dizinde kalsın (sqlx normu)
- [x] `.sqlx/` offline metadata'yı yeni yapıya göre güncelle (`cargo sqlx prepare --workspace`)
- [x] `Dockerfile` yeni workspace yapısına göre güncelle
- [x] `docker-compose.yml` değişiklik gerekiyorsa güncelle
- [x] Tüm 10 E2E test (`cargo test`) yeşil
- [x] `cargo clippy --workspace -- -D warnings` temiz
- [x] CI pipeline (`.github/workflows/ci.yml`) workspace komutlarına güncelle

---

## Faz 2 — Deklaratif RBAC Middleware & Extractor

> **Neden:** Handler'lar içindeki manuel `if user.role != "admin"` kontrolleri → merkezi, tip-güvenli, unutulamaz yetki katmanı.

### 2.1 — `AuthenticatedUser` Extractor
- [x] `crates/web/src/extractors/mod.rs` oluştur
- [x] `AuthenticatedUser` Axum extractor struct'ı yaz (cookie'den session çözer, `User` döner)
- [x] Oturum yoksa otomatik `401 Unauthorized` döner (handler'da kontrol gereksiz)
- [x] Mevcut `resolve_current_user` fonksiyonunu bu extractor ile değiştir

### 2.2 — `RequireRole` Guard Layer
- [x] `RequireRole` enum/struct tanımla (`Admin`, `User`, `Owner(i64)`)
- [x] Axum `middleware::from_fn` veya `tower::Layer` ile route-seviyesi guard yaz
- [x] Örnek kullanım: `.route("/admin/panel", get(admin_panel).route_layer(RequireRole::Admin))`
- [x] Yetkisiz erişimde `403 Forbidden` + audit log (`tracing::warn!`)

### 2.3 — Handler Refaktör
- [x] `show_profile` handler'dan inline yetki kontrolünü kaldır → `RequireRole::Owner` kullan
- [x] `show_debug` handler'dan inline mod kontrolünü kaldır → guard layer kullan
- [x] `create_post` handler'a `AuthenticatedUser` extractor ekle
- [x] Tüm handler'ların ilk satırında yetki kararı OLMASIN, sadece iş mantığı

### 2.4 — Doğrulama
- [x] IDOR testi hâlâ yeşil (secure modda `403`)
- [x] Debug endpoint testi hâlâ yeşil (secure modda `403`)
- [x] Yeni birim testi: `AuthenticatedUser` extractor oturumsuz → 401
- [x] Yeni birim testi: `RequireRole::Admin` normal kullanıcıyla → 403

---

## Faz 3 — Structured JSON Logging & Correlation ID (A09:2026+)

> **Neden:** Düz metin loglar → SIEM-uyumlu yapılandırılmış JSON + her isteği izlenebilir kılan RequestId.

### 3.1 — Structured JSON Formatter
- [x] `tracing-subscriber` features'a `"json"` ekle
- [x] `Cargo.toml`'a `tracing-serde` ekle (opsiyonel)
- [x] `main.rs`'deki tracing init'i güncelle: ortam değişkeniyle (`LOG_FORMAT=json|pretty`) seçilebilir
- [x] JSON formatında log çıktısı: `{"timestamp":"...","level":"WARN","target":"...","message":"..."}`
- [x] Development'ta `pretty` (renkli), production/Docker'da `json` varsayılan

### 3.2 — Request Correlation ID Middleware
- [x] `tower-http`'nin `RequestIdLayer` veya özel UUID middleware yaz
- [x] Her gelen HTTP isteğine `X-Request-Id` header'ı ata (yoksa üret)
- [x] Response'a da `X-Request-Id` header'ı ekle (client tarafı korelasyon)
- [x] Tüm handler loglarında `request_id` span'ı otomatik olarak görünsün
- [x] `tracing::info_span!("request", request_id = %id)` ile span oluştur

### 3.3 — Security Audit Log Yapısı
- [x] Başarısız login denemelerini yapılandırılmış logla: `{"event":"login_failed","ip":"...","username":"..."}`
- [x] IDOR ihlal girişimlerini logla: `{"event":"idor_blocked","attacker_id":...,"target_id":...}`
- [x] Rate limit tetiklemelerini logla: `{"event":"rate_limited","ip":"..."}`
- [x] Tüm audit logları `tracing::warn!` seviyesinde, `target = "security_audit"` ile

### 3.4 — Doğrulama
- [x] Docker loglarında JSON formatı doğrula (`docker compose logs | jq .`)
- [x] Her response'ta `X-Request-Id` header'ı mevcut (test)
- [x] Brute-force testi esnasında audit loglarında `login_failed` event'leri görünüyor
- [x] `.env.example`'a `LOG_FORMAT=json` ekle

---

## Faz 4 — Gelişmiş Oturum & Çerez Güvenliği (A02:2026+)

> **Neden:** Düz metin session token → şifreli/imzalı (Authenticated Encryption) çerez + HKDF anahtar türetme.

### 4.1 — HKDF Anahtar Türetme
- [x] `Cargo.toml`'a `hkdf = "0.12"` ve `sha2 = "0.10"` ekle
- [x] `SESSION_SECRET` envvar'ından HKDF-SHA256 ile iki ayrı anahtar türet: `signing_key` + `encryption_key`
- [x] Anahtar türetme fonksiyonunu `crates/core/src/crypto.rs` modülüne yaz
- [x] Birim testi: aynı secret → aynı türetilmiş anahtarlar (deterministic)

### 4.2 — AES-GCM Encrypted Cookie
- [x] `Cargo.toml`'a `aes-gcm = "0.10"` ekle
- [x] `encrypt_cookie(key, plaintext) -> base64_ciphertext` fonksiyonu yaz
- [x] `decrypt_cookie(key, base64_ciphertext) -> Result<plaintext>` fonksiyonu yaz
- [x] Her şifrelemede benzersiz 96-bit nonce üret (`OsRng`)
- [x] Nonce'u ciphertext'in önüne ekle: `nonce || ciphertext || tag`

### 4.3 — HMAC Cookie İmzası (Tamper-Proof)
- [x] `Cargo.toml`'a `hmac = "0.12"` ekle
- [x] `sign_cookie(key, value) -> value.signature` fonksiyonu yaz
- [x] `verify_cookie(key, value, signature) -> bool` fonksiyonu yaz
- [x] Geçersiz imza → oturumu sil + 401

### 4.4 — Entegrasyon
- [x] `auth_handlers::login` → secure modda `encrypt_cookie` + `sign_cookie` kullan
- [x] `resolve_current_user` → secure modda `verify_cookie` + `decrypt_cookie` kullan
- [x] Zafiyetli mod etkilenMEZ (eski Base64 davranış korunur, karşılaştırma için)
- [x] Mevcut session token DB mekanizması korunur (çerez sadece transport katmanı)

### 4.5 — Doğrulama
- [x] Birim testi: encrypt → decrypt round-trip
- [x] Birim testi: tampered cookie → verify başarısız
- [x] E2E: secure modda login → cookie şifreli (base64 decode edilince anlamsız)
- [x] E2E: cookie'yi manuel değiştirince 401 dönüyor
- [x] Tüm mevcut 10 E2E test hâlâ yeşil

---

## Faz 5 — DB Transaction & Connection Resilience

> **Neden:** Bağımsız sorgular → atomik transaction'lar + bağlantı kopmasına dayanıklılık.

### 5.1 — Transaction Wrapper
- [ ] `crates/core/src/db.rs`'e generic `with_tx` yardımcı fonksiyon ekle:
  ```rust
  pub async fn with_tx<F, T>(pool: &PgPool, f: F) -> Result<T, ApiError>
  where F: FnOnce(&mut Transaction<'_, Postgres>) -> BoxFuture<'_, Result<T, ApiError>>
  ```
- [ ] `SecureAuth::register` → transaction içine al (user insert + otomatik session oluşturma)
- [ ] `SecureAuth::login` → transaction içine al (session insert + eski session temizleme)
- [ ] Hata durumunda otomatik `ROLLBACK` (sqlx Transaction drop semantiği)

### 5.2 — Connection Pool Hardening
- [ ] `PgPoolOptions` ayarlarını production-grade yap:
  - [ ] `max_connections(10)` (Docker ortamı için makul)
  - [ ] `acquire_timeout(Duration::from_secs(5))`
  - [ ] `idle_timeout(Duration::from_secs(600))`
  - [ ] `max_lifetime(Duration::from_secs(1800))`
- [ ] `min_connections(2)` ile warm pool (soğuk başlangıç engeli)
- [ ] Bağlantı havuzu metriklerini log'a yaz (başlangıçta `pool.size()`, `pool.num_idle()`)

### 5.3 — Graceful Shutdown
- [ ] `tokio::signal::ctrl_c()` ile graceful shutdown entegre et
- [ ] Shutdown sırasında aktif bağlantıları tamamla, yeni istek kabul etme
- [ ] `pool.close().await` ile veritabanı bağlantılarını temiz kapat
- [ ] Shutdown log mesajı: `"Server shutting down gracefully..."`

### 5.4 — Health Check Derinleştirme
- [ ] `/health` endpoint'ini zenginleştir: DB bağlantısı kontrol et (`SELECT 1`)
- [ ] Response: `{"status":"healthy","db":"connected","uptime_secs":...}`
- [ ] DB bağlantısı yoksa `503 Service Unavailable` dön
- [ ] Docker `healthcheck` komutunu bu endpoint'e yönlendir

### 5.5 — Doğrulama
- [ ] Register sırasında DB hatası → ROLLBACK (kısmi veri YOK)
- [ ] `/health` DB kapalıyken 503 dönüyor
- [ ] Graceful shutdown sırasında aktif istek tamamlanıyor
- [ ] Tüm 10 E2E test hâlâ yeşil

---

## 🏁 Final Kontrol Listesi

- [ ] Tüm 5 faz tamamlandı
- [ ] `cargo build --workspace` hatasız
- [ ] `cargo clippy --workspace -- -D warnings` uyarısız
- [ ] `cargo fmt --all --check` temiz
- [ ] `cargo test` → tüm testler yeşil
- [ ] `docker compose up --build` → uygulama sağlıklı çalışıyor
- [ ] README.md yeni mimariyi yansıtacak şekilde güncellendi
- [ ] Git tag: `v2.0.0-senior`
