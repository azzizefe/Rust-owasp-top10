// crates/core/src/secrets_vault.rs — HashiCorp Vault KV v2 Secrets Provider
//
// Vault'un Key/Value v2 secret engine'inden secret okur.
// Gerekli ortam değişkenleri:
//   - VAULT_ADDR  : Vault sunucu adresi (örn: http://127.0.0.1:8200)
//   - VAULT_TOKEN : Vault authentication token'ı
//   - VAULT_MOUNT : KV v2 mount noktası (varsayılan: "secret")
//   - VAULT_PATH  : Secret'ların saklandığı path (varsayılan: "owasp-lab")

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use tracing::{debug, error};

use crate::secrets::{SecretsError, SecretsProvider};

/// HashiCorp Vault KV v2 Secret Engine üzerinden secret okuyan provider.
///
/// # Vault KV v2 API
/// Secret'lar `GET {addr}/v1/{mount}/data/{path}` endpoint'inden çekilir.
/// Yanıt formatı:
/// ```json
/// {
///   "data": {
///     "data": {
///       "DATABASE_URL": "postgres://...",
///       "SESSION_SECRET": "..."
///     },
///     "metadata": { ... }
///   }
/// }
/// ```
///
/// # Önbellekleme
/// Tüm secret'lar ilk çağrıda Vault'tan çekilir ve belleğe alınır.
/// Bu sayede her `get_secret` çağrısı için ayrı HTTP isteği yapılmaz.
pub struct VaultSecretsProvider {
    /// Vault sunucu adresi (trailing slash olmadan)
    addr: String,
    /// Vault authentication token
    token: String,
    /// KV v2 mount noktası
    mount: String,
    /// Secret path (mount altındaki yol)
    path: String,
    /// HTTP client
    client: Client,
    /// Önbelleklenmiş secret'lar (ilk erişimde doldurulur)
    cache: tokio::sync::OnceCell<HashMap<String, String>>,
}

/// Vault KV v2 API yanıt yapısı
#[derive(Deserialize)]
struct VaultResponse {
    data: VaultDataWrapper,
}

#[derive(Deserialize)]
struct VaultDataWrapper {
    data: HashMap<String, serde_json::Value>,
}

impl VaultSecretsProvider {
    /// Ortam değişkenlerinden Vault bağlantı bilgilerini okuyarak provider oluşturur.
    pub fn from_env() -> Result<Self, SecretsError> {
        let addr = std::env::var("VAULT_ADDR").map_err(|_| {
            SecretsError::ConnectionError("VAULT_ADDR ortam değişkeni tanımlı değil".to_string())
        })?;

        let token = std::env::var("VAULT_TOKEN").map_err(|_| {
            SecretsError::AuthError("VAULT_TOKEN ortam değişkeni tanımlı değil".to_string())
        })?;

        let mount = std::env::var("VAULT_MOUNT").unwrap_or_else(|_| "secret".to_string());
        let path = std::env::var("VAULT_PATH").unwrap_or_else(|_| "owasp-lab".to_string());

        // Trailing slash temizliği
        let addr = addr.trim_end_matches('/').to_string();

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| {
                SecretsError::ConnectionError(format!("HTTP client oluşturulamadı: {e}"))
            })?;

        Ok(Self {
            addr,
            token,
            mount,
            path,
            client,
            cache: tokio::sync::OnceCell::new(),
        })
    }

    /// Vault'tan tüm secret'ları çeker ve önbelleğe alır.
    async fn fetch_all(&self) -> Result<&HashMap<String, String>, SecretsError> {
        self.cache
            .get_or_try_init(|| async {
                let url = format!("{}/v1/{}/data/{}", self.addr, self.mount, self.path);

                debug!(url = %url, "Vault'tan secret'lar çekiliyor...");

                let response = self
                    .client
                    .get(&url)
                    .header("X-Vault-Token", &self.token)
                    .header("X-Vault-Request", "true")
                    .send()
                    .await
                    .map_err(|e| {
                        error!("Vault bağlantı hatası: {:?}", e);
                        SecretsError::ConnectionError(format!("Vault'a bağlanılamadı: {e}"))
                    })?;

                if response.status() == reqwest::StatusCode::FORBIDDEN
                    || response.status() == reqwest::StatusCode::UNAUTHORIZED
                {
                    return Err(SecretsError::AuthError(
                        "Vault token geçersiz veya yetkisiz".to_string(),
                    ));
                }

                if response.status() == reqwest::StatusCode::NOT_FOUND {
                    return Err(SecretsError::NotFound {
                        key: format!("{}/{}", self.mount, self.path),
                    });
                }

                if !response.status().is_success() {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    return Err(SecretsError::ProviderError(format!(
                        "Vault yanıt hatası (HTTP {status}): {body}"
                    )));
                }

                let vault_resp: VaultResponse = response.json().await.map_err(|e| {
                    SecretsError::ProviderError(format!("Vault yanıtı parse edilemedi: {e}"))
                })?;

                // JSON değerlerini string'e dönüştür
                let secrets: HashMap<String, String> = vault_resp
                    .data
                    .data
                    .into_iter()
                    .map(|(k, v)| {
                        let val = match v {
                            serde_json::Value::String(s) => s,
                            other => other.to_string(),
                        };
                        (k, val)
                    })
                    .collect();

                debug!(
                    secret_count = secrets.len(),
                    "Vault'tan {} secret başarıyla yüklendi",
                    secrets.len()
                );

                Ok(secrets)
            })
            .await
    }
}

#[async_trait]
impl SecretsProvider for VaultSecretsProvider {
    async fn get_secret(&self, key: &str) -> Result<String, SecretsError> {
        let secrets = self.fetch_all().await?;
        secrets
            .get(key)
            .cloned()
            .ok_or_else(|| SecretsError::NotFound {
                key: key.to_string(),
            })
    }

    async fn get_secrets(&self, keys: &[&str]) -> Result<HashMap<String, String>, SecretsError> {
        let all_secrets = self.fetch_all().await?;
        let mut result = HashMap::with_capacity(keys.len());
        for key in keys {
            let val = all_secrets
                .get(*key)
                .cloned()
                .ok_or_else(|| SecretsError::NotFound {
                    key: key.to_string(),
                })?;
            result.insert(key.to_string(), val);
        }
        Ok(result)
    }

    fn provider_name(&self) -> &'static str {
        "vault"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_env_fails_without_vault_addr() {
        // VAULT_ADDR'ın tanımlı olmadığından emin ol
        std::env::remove_var("VAULT_ADDR");
        std::env::remove_var("VAULT_TOKEN");

        let result = VaultSecretsProvider::from_env();
        match result {
            Err(SecretsError::ConnectionError(msg)) => {
                assert!(msg.contains("VAULT_ADDR"));
            }
            Err(other) => panic!("Beklenen ConnectionError, alınan: {:?}", other),
            Ok(_) => panic!("Hata bekleniyordu ama başarı döndü"),
        }
    }

    #[test]
    fn from_env_fails_without_vault_token() {
        std::env::set_var("VAULT_ADDR", "http://127.0.0.1:8200");
        std::env::remove_var("VAULT_TOKEN");

        let result = VaultSecretsProvider::from_env();
        match result {
            Err(SecretsError::AuthError(msg)) => {
                assert!(msg.contains("VAULT_TOKEN"));
            }
            Err(other) => panic!("Beklenen AuthError, alınan: {:?}", other),
            Ok(_) => panic!("Hata bekleniyordu ama başarı döndü"),
        }

        std::env::remove_var("VAULT_ADDR");
    }

    #[test]
    fn from_env_succeeds_with_required_vars() {
        std::env::set_var("VAULT_ADDR", "http://127.0.0.1:8200");
        std::env::set_var("VAULT_TOKEN", "test-token");

        let provider = VaultSecretsProvider::from_env();
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.addr, "http://127.0.0.1:8200");
        assert_eq!(provider.mount, "secret");
        assert_eq!(provider.path, "owasp-lab");
        assert_eq!(provider.provider_name(), "vault");

        std::env::remove_var("VAULT_ADDR");
        std::env::remove_var("VAULT_TOKEN");
    }
}
