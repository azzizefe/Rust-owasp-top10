// crates/core/src/secrets.rs — Cloud Secrets Provider Abstraction Layer
//
// Pluggable sır yönetimi altyapısı. Tüm hassas değerler (DB şifresi, SESSION_SECRET,
// TLS anahtarları vb.) bu katman üzerinden çözümlenir.
//
// Desteklenen provider'lar:
//   - env    : Mevcut .env / ortam değişkeni davranışı (varsayılan, geriye dönük uyumlu)
//   - vault  : HashiCorp Vault KV v2
//   - aws    : AWS Secrets Manager
//   - doppler: Doppler REST API

use async_trait::async_trait;
use std::collections::HashMap;
use tracing::{info, warn};

// ─────────────────────────────────────────────
// Error
// ─────────────────────────────────────────────

/// Secret provider'lardan kaynaklanan hataları saran enum.
#[derive(thiserror::Error, Debug)]
pub enum SecretsError {
    #[error("secret bulunamadı: {key}")]
    NotFound { key: String },

    #[error("provider bağlantı hatası: {0}")]
    ConnectionError(String),

    #[error("kimlik doğrulama hatası: {0}")]
    AuthError(String),

    #[error("provider hatası: {0}")]
    ProviderError(String),
}

// ─────────────────────────────────────────────
// Trait — Ortak Arayüz
// ─────────────────────────────────────────────

/// Tüm secret provider implementasyonlarının uygulaması gereken ortak arayüz.
///
/// # Tasarım Kararları
/// - `Send + Sync` bound: Provider'ın birden fazla Tokio task arasında güvenle paylaşılabilmesi için.
/// - Async metotlar: Vault/AWS/Doppler gibi ağ tabanlı provider'lar asenkron I/O gerektirir.
/// - `get_secrets` batch metodu: Birden fazla secret'ı tek roundtrip'te çekmek için.
#[async_trait]
pub trait SecretsProvider: Send + Sync {
    /// Tek bir secret değerini key'e göre çeker.
    async fn get_secret(&self, key: &str) -> Result<String, SecretsError>;

    /// Birden fazla secret'ı tek seferde çeker.
    /// Varsayılan implementasyon her key için `get_secret` çağırır;
    /// provider'lar bunu batch API ile override edebilir.
    async fn get_secrets(&self, keys: &[&str]) -> Result<HashMap<String, String>, SecretsError> {
        let mut map = HashMap::with_capacity(keys.len());
        for key in keys {
            let value = self.get_secret(key).await?;
            map.insert(key.to_string(), value);
        }
        Ok(map)
    }

    /// Provider'ın insan tarafından okunabilir adı (loglama için).
    fn provider_name(&self) -> &'static str;
}

// ─────────────────────────────────────────────
// EnvSecretsProvider — Varsayılan / Fallback
// ─────────────────────────────────────────────

/// Ortam değişkenlerinden (ve `.env` dosyasından) secret okuyan varsayılan provider.
///
/// Geriye dönük uyumluluk sağlar: `SECRETS_PROVIDER` tanımlanmamışsa bu kullanılır.
/// Development ortamı için idealdir; production'da harici bir provider tercih edilmelidir.
pub struct EnvSecretsProvider;

impl EnvSecretsProvider {
    pub fn new() -> Self {
        // .env dosyasını yükle (yoksa veya hatalıysa sessizce geç)
        let _ = dotenvy::dotenv();
        Self
    }
}

impl Default for EnvSecretsProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SecretsProvider for EnvSecretsProvider {
    async fn get_secret(&self, key: &str) -> Result<String, SecretsError> {
        std::env::var(key).map_err(|_| SecretsError::NotFound {
            key: key.to_string(),
        })
    }

    fn provider_name(&self) -> &'static str {
        "env"
    }
}

// ─────────────────────────────────────────────
// Factory — Provider Builder
// ─────────────────────────────────────────────

/// `SECRETS_PROVIDER` ortam değişkenine göre uygun provider'ı oluşturur.
///
/// | Değer     | Provider               | Notlar                                 |
/// |-----------|------------------------|----------------------------------------|
/// | `env`     | `EnvSecretsProvider`   | Varsayılan, .env + ortam değişkenleri  |
/// | `vault`   | `VaultSecretsProvider` | `vault` feature flag'i gerektirir      |
/// | `aws`     | `AwsSecretsProvider`   | `aws` feature flag'i gerektirir        |
/// | `doppler` | `DopplerSecretsProvider`| Ek bağımlılık gerektirmez (reqwest)   |
/// | *boş/diğer* | `EnvSecretsProvider` | Sessiz fallback                       |
pub async fn build_provider_async() -> Box<dyn SecretsProvider> {
    // Factory'nin kendisi de .env'den okuyabilmeli (SECRETS_PROVIDER değerini bulmak için)
    let _ = dotenvy::dotenv();

    let provider_key = std::env::var("SECRETS_PROVIDER")
        .unwrap_or_else(|_| "env".to_string())
        .to_lowercase();

    match provider_key.as_str() {
        "env" | "" => {
            info!(
                provider = "env",
                "Secrets provider: Ortam değişkenleri (EnvSecretsProvider)"
            );
            Box::new(EnvSecretsProvider::new())
        }

        // ── HashiCorp Vault ──
        #[cfg(feature = "vault")]
        "vault" => {
            info!(provider = "vault", "Secrets provider: HashiCorp Vault");
            Box::new(
                crate::secrets_vault::VaultSecretsProvider::from_env().expect(
                    "Vault provider oluşturulamadı — VAULT_ADDR ve VAULT_TOKEN kontrol edin",
                ),
            )
        }
        #[cfg(not(feature = "vault"))]
        "vault" => {
            panic!("Vault provider kullanmak için `vault` feature flag'ini etkinleştirin: cargo build --features vault")
        }

        // ── AWS Secrets Manager ──
        #[cfg(feature = "aws")]
        "aws" => {
            info!(provider = "aws", "Secrets provider: AWS Secrets Manager");
            Box::new(
                crate::secrets_aws::AwsSecretsProvider::new()
                    .await
                    .expect("AWS Secrets Manager provider oluşturulamadı"),
            )
        }
        #[cfg(not(feature = "aws"))]
        "aws" => {
            panic!("AWS provider kullanmak için `aws` feature flag'ini etkinleştirin: cargo build --features aws")
        }

        // ── Doppler ──
        #[cfg(feature = "doppler")]
        "doppler" => {
            info!(provider = "doppler", "Secrets provider: Doppler");
            Box::new(
                crate::secrets_doppler::DopplerSecretsProvider::from_env()
                    .expect("Doppler provider oluşturulamadı — DOPPLER_TOKEN kontrol edin"),
            )
        }
        #[cfg(not(feature = "doppler"))]
        "doppler" => {
            panic!("Doppler provider kullanmak için `doppler` feature flag'ini etkinleştirin: cargo build --features doppler")
        }

        other => {
            warn!(
                provider = other,
                "Bilinmeyen SECRETS_PROVIDER değeri, EnvSecretsProvider'a düşülüyor"
            );
            Box::new(EnvSecretsProvider::new())
        }
    }
}

// ─────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn env_provider_reads_existing_var() {
        std::env::set_var("TEST_SECRET_KEY_12345", "super_secret_value");
        let provider = EnvSecretsProvider::new();

        let result = provider.get_secret("TEST_SECRET_KEY_12345").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "super_secret_value");

        std::env::remove_var("TEST_SECRET_KEY_12345");
    }

    #[tokio::test]
    async fn env_provider_returns_not_found_for_missing_var() {
        let provider = EnvSecretsProvider::new();

        let result = provider.get_secret("ABSOLUTELY_NONEXISTENT_KEY_XYZ").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            SecretsError::NotFound { key } => {
                assert_eq!(key, "ABSOLUTELY_NONEXISTENT_KEY_XYZ");
            }
            other => panic!("Beklenen SecretsError::NotFound, alınan: {:?}", other),
        }
    }

    #[tokio::test]
    async fn env_provider_batch_get_secrets() {
        std::env::set_var("BATCH_KEY_A", "value_a");
        std::env::set_var("BATCH_KEY_B", "value_b");

        let provider = EnvSecretsProvider::new();
        let result = provider.get_secrets(&["BATCH_KEY_A", "BATCH_KEY_B"]).await;

        assert!(result.is_ok());
        let map = result.unwrap();
        assert_eq!(map.get("BATCH_KEY_A").unwrap(), "value_a");
        assert_eq!(map.get("BATCH_KEY_B").unwrap(), "value_b");

        std::env::remove_var("BATCH_KEY_A");
        std::env::remove_var("BATCH_KEY_B");
    }

    #[tokio::test]
    async fn env_provider_name_is_env() {
        let provider = EnvSecretsProvider::new();
        assert_eq!(provider.provider_name(), "env");
    }
}
