# 📦 Core Modülü Teknik Dokümantasyonu (`crates/core`)

`crates/core`, **Rust OWASP Top 10 Security Lab** projesinin iş mantığını (domain logic), veri erişim katmanını, kriptografik işlevlerini ve konfigürasyon yönetimini barındıran temel Rust kütüphanesidir. Web katmanından (`crates/web`) tamamen izole edilmiştir.

---

## 📂 Dizin Yapısı ve Modüller

```
crates/core/src/
├── auth/               # Argon2id şifre doğrulama ve timing attack koruması
├── config.rs           # Çevresel değişkenler ve yapılandırma yönetimi
├── crypto.rs           # Simetrik şifreleme (AES-256-GCM), HMAC ve HKDF
├── db.rs               # PostgreSQL bağlantı havuzu (sqlx::PgPool) yönetimi
├── error.rs            # Merkezi hata tanımları (thiserror)
├── lib.rs              # Modül dışa aktarımları (re-exports)
├── models.rs           # Veritabanı tablolarının Rust veri modelleri
├── secrets.rs          # Pluggable Secrets Provider soyutlaması (Trait)
├── secrets_aws.rs      # AWS Secrets Manager entegrasyonu
├── secrets_doppler.rs  # Doppler Secrets Manager entegrasyonu
├── secrets_vault.rs    # HashiCorp Vault KV v2 entegrasyonu
└── session.rs          # Oturum verisi yapısı ve serileştirme
```

---

## 🛠️ Ana Bileşen Detayları

### 1. Kriptografi Modülü (`crypto.rs`)
Sistemdeki tüm hassas verilerin taşınması ve saklanması sürecinde kullanılan AEAD (Authenticated Encryption with Associated Data) mimarisini uygular.
*   **AES-256-GCM:** Oturum çerezleri gibi hassas verilerin şifrelenmesinde kullanılır. Çerezlerin istemci tarafında okunmasını engeller.
*   **HMAC-SHA256:** Şifrelenmiş verilerin bütünlüğünü doğrulamak ve tahrif edilip edilmediğini kontrol etmek için imzalama yapar.
*   **HKDF (HMAC-based Extract-and-Expand Key Derivation Function):** Ana anahtardan (Session Secret) deterministik ve kriptografik olarak güvenli alt anahtarlar türetir.

### 2. Sır Sağlayıcı Modülü (`secrets.rs` ve varyasyonları)
Sistemdeki gizli anahtarların ve veritabanı kimlik bilgilerinin diskte düz metin olarak saklanmasını önlemek için geliştirilmiş, asenkron ve genişletilebilir bir Trait tabanlı altyapıdır.
*   **`SecretsProvider` Trait:** Gizli anahtarları getirmek için standart bir arayüz tanımlar.
*   **Sağlayıcılar:**
    *   `EnvSecretsProvider`: `.env` dosyasından veya doğrudan OS ortam değişkenlerinden okuma yapar (Yerel geliştirme).
    *   `VaultSecretsProvider`: HashiCorp Vault KV v2 motoruna asenkron API istekleri atar.
    *   `AwsSecretsProvider`: AWS Secrets Manager API'sini kullanır.
    *   `DopplerSecretsProvider`: Doppler API entegrasyonu sağlar.

### 3. Kimlik Doğrulama Modülü (`auth/`)
Parola güvenliği ve zamanlama analizi (Timing Attack) sönümleme mantığı bu modülde yer alır.
*   **Argon2id:** Endüstri standardı olan en güncel ve güvenli parola özetleme (hashing) algoritmasını kullanır.
*   **Yapay Argon2id Gecikmesi (Dummy Hash Pad):** Kullanıcı veritabanında bulunamadığında, doğrulamayı hemen kesmek yerine arka planda yapay bir Argon2id hesaplaması yaparak brute-force saldırganlarına karşı kullanıcı mevcudiyeti bilgisinin sızmasını engeller.

### 4. Veritabanı Bağlantı Havuzu (`db.rs`)
*   `sqlx::PgPool` kullanarak PostgreSQL veritabanına asenkron bağlantıları yönetir.
*   Zero-Trust ilkeleri gereği veritabanı bağlantısı `sslmode=require` parametresi ile programatik olarak zorlanır.

### 5. Hata Yönetimi (`error.rs`)
*   `thiserror` kütüphanesi kullanılarak sistem genelinde oluşan hatalar (`DatabaseError`, `CryptoError`, `ConfigError`, `SecretsError`) tek bir enum altında toplanmıştır.
*   Güvenlik gerekçesiyle iç hata detayları (ham SQL hataları vb.) kullanıcıya dönülmez, sadece loglanır.
