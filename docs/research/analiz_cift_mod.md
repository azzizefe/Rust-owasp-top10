# 🎓 Araştırma Raporu 1: Çift Modlu Canlı Karşılaştırma Paradigması (Pillar 1)

Bu çalışma, tek bir kod tabanında, zafiyetli (`vulnerable`) ve zırhlandırılmış (`secure`) çalışma modlarının çalışma zamanı (boot-time) parametreleriyle dinamik olarak karşılaştırılabilmesini sağlayan mimari tasarımı incelemektedir. 

Geleneksel siber güvenlik laboratuvarlarının aksine, bu sistemde iki ayrı uygulama yerine **aynı veri modeli ve routing katmanını kullanan, fakat iş mantığında ayrışan** bir "çift modlu" mimari kurgulanmıştır.

---

## 🏛️ Mimari Tasarım ve Soyutlama Katmanı (Trait Pattern)

Çift modlu yapının temelinde, Rust'ın güçlü **Polimorfizm** ve **Trait (Arayüz)** yetenekleri yatmaktadır. Uygulamanın web handler katmanı (HTTP isteklerini karşılayan fonksiyonlar), arka planda hangi güvenlik modunun çalıştığından tamamen bağımsız (decoupled) şekilde kodlanmıştır.

### 1. `AuthBackend` Soyutlaması

Tüm kimlik doğrulama, kullanıcı yönetimi, gönderi paylaşımı ve arama işlevleri, `crates/core/src/auth/mod.rs` içerisinde tanımlanan asenkron bir `AuthBackend` trait'i ile soyutlanmıştır:

```rust
#[async_trait]
pub trait AuthBackend: Send + Sync {
    async fn login(&self, u: &str, p: &str) -> Result<Session, ApiError>;
    async fn register(&self, f: &RegisterForm) -> Result<i64, ApiError>;
    async fn find_user(&self, id: i64) -> Result<User, ApiError>;
    async fn create_post(&self, user_id: i64, content: &str) -> Result<Post, ApiError>;
    async fn search_posts(&self, query: &str) -> Result<Vec<Post>, ApiError>;
}
```

Bu arayüz, iki farklı yapı (struct) tarafından implement edilmiştir:
-   `VulnerableAuth`: Zafiyetli iş mantığını, güvensiz sorguları ve korumasız oturum işlemlerini barındırır.
-   `SecureAuth`: OWASP Top 10 korumaları entegre edilmiş, zırhlandırılmış mantığı barındırır.

### 2. Çalışma Zamanı Karar Mekanizması (Factory Pattern)

Uygulama ayağa kalkarken (boot phase), `.env` dosyasından okunan `APP_MODE` değişkenine göre hangi backend'in kullanılacağına karar verilir ve bu nesne bir `Arc` (Atomic Reference Counter) içine sarmalanarak thread-safe bir şekilde web sunucusuna enjekte edilir:

```rust
pub fn build(mode: &AppMode, pool: PgPool) -> Arc<dyn AuthBackend> {
    match mode {
        AppMode::Vulnerable => Arc::new(VulnerableAuth { pool }),
        AppMode::Secure => Arc::new(SecureAuth { pool }),
    }
}
```

Web handler'ları (`crates/web/src/handlers/`), kendilerine gelen HTTP isteklerini işlerken doğrudan bu soyut `Arc<dyn AuthBackend>` nesnesini çağırır. Örneğin:

```rust
// auth_handlers.rs
pub async fn login_handler(
    State(state): State<AppState>,
    Form(payload): Form<LoginForm>,
) -> Result<impl IntoResponse, ApiError> {
    let session = state.auth.login(&payload.username, &payload.password).await?;
    // Oturum işlemleri...
}
```

---

## 🔬 Bilimsel Yenilik (Pedagogical & Empirical Novelty)

Bu çift modlu mimari, siber güvenlik eğitiminde ve güvenlik testlerinde 3 kritik ampirik avantaj sağlar:

1.  **Tam Eşdeğerlikli Karşılaştırma (Identical Profiling):** Her iki modda da veritabanı şeması, HTTP routing kuralları, Nginx proxy ayarları ve ağ gecikmeleri birebir aynıdır. Değişen tek şey güvenlik iş mantığıdır. Bu sayede, güvenlik yamalarının uygulama performansı (CPU döngüleri, RAM kullanımı) üzerindeki gerçek etkisi tam bir bilimsel izolasyon altında ölçülebilir.
2.  **Otomatik PoC Doğrulaması (Deterministic E2E Testing):** Yazılan sömürü betikleri (PoC), herhangi bir yapısal değişiklik gerektirmeden her iki moda karşı da çalıştırılabilir.
    *   `vulnerable` modda: Sömürünün başarılı olduğu (200 OK, veri sızıntısı, XSS alert tetiklenmesi) doğrulanır.
    *   `secure` modda: Aynı girdinin başarısız olduğu (401 Unauthorized, 403 Forbidden, 429 Too Many Requests, ya da zararsız kaçış karakterleri) doğrulanır.
    This determinism proves the correctness of defenses instantly through E2E tests.
3.  **Hızlı DevSecOps Simülasyonu:** Geliştiriciler ve öğrenciler, tek bir docker-compose ortamında `APP_MODE` bayrağını değiştirerek kodun güvenli hale gelme anını canlı izleyebilir, SIEM loglarındaki farkı eşzamanlı gözlemleyebilirler.

---

## 🛡️ Canlı Ortam Güvenliği (Fail-Safe Production Principle)

Çift modlu laboratuvarların en büyük riski, zafiyetli kodun (Vulnerable Mode) kazara dış dünyaya açık üretim (production) ortamında aktifleşmesidir. RustSec-analyzer bu riski bertaraf etmek için **Fail-Safe** tasarımı benimsemiştir:

*   **Güvenli Varsayılan (Secure by Default):** `.env` dosyasında `APP_MODE` eksik veya geçersiz bir değerse, sistem otomatik olarak en güvenli mod olan `Secure` moduna geçer.
*   **Derleme Seviyesinde Kilitleme İmkanı:** İleri seviye kurumsal sürümlerde, `vulnerable` mod kod tabanından derleme zamanı koşullu derleme (`#[cfg(feature = "vulnerable")]`) özellikleri ile tamamen çıkartılabilir. Bu sayede canlı sunucu imajı (docker image) zafiyetli kod bloklarını fiziksel olarak hiç içermez.
*   **Sesli Hata / Uyarı Loglama:** Uygulama `vulnerable` modda başlatıldığında terminalde büyük, kırmızı uyarı logları (`WARN: RUNNING IN VULNERABLE MODE — localhost only`) basılarak operatörün dikkati çekilir.

---

## 📊 Karşılaştırmalı Özet

| Kriter | Geleneksel Ayrı Güvenlik Laboratuvarları | RustSec-analyzer Çift Modlu Mimari |
|---|---|---|
| **Efor ve Bakım** | İki farklı kod tabanı, iki ayrı deploy süreci. | Tek kod tabanı, tek bir entegre CI/CD pipeline'ı. |
| **Ölçüm İzolasyonu** | Farklı kütüphaneler ve ağ yapıları performansı etkiler. | Ağ, router ve veri modeli %100 izole, sadece iş mantığı değişir. |
| **Sömürü Doğrulaması** | Manuel veya ayrı test scriptleri gerekir. | Aynı PoC scripti iki modda da çalışarak diferansiyel analiz sunar. |
| **Derleme Güvencesi** | Genelde dinamik diller (Node/Python) kullanılır, statik analiz zayıftır. | Rust'ın statik tiplemesi altında her iki modun da doğruluğu garanti edilir. |
