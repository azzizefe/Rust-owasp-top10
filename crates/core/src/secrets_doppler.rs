// crates/core/src/secrets_doppler.rs — Doppler Secrets Provider
//
// Doppler SaaS platformundan secret okur.
// Gerekli ortam değişkenleri:
//   - DOPPLER_TOKEN   : Doppler Service Token (dp.st.xxxxx formatında)
//   - DOPPLER_PROJECT : Proje adı (varsayılan: "owasp-lab")
//   - DOPPLER_CONFIG  : Konfigürasyon adı (varsayılan: "production")
//
// API Endpoint: https://api.doppler.com/v3/configs/config/secrets/download?format=json

use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;
use tracing::{debug, error};

use crate::secrets::{SecretsError, SecretsProvider};

/// Doppler SaaS platformu üzerinden secret okuyan provider.
///
/// # Doppler API
/// Tüm secret'lar tek bir API çağrısıyla JSON formatında indirilir:
/// ```text
/// GET https://api.doppler.com/v3/configs/config/secrets/download?format=json
/// Authorization: Bearer dp.st.xxxxx
/// ```
///
/// Yanıt:
/// ```json
/// {
///   "DATABASE_URL": "postgres://...",
///   "SESSION_SECRET": "..."
/// }
/// ```
///
/// # Token Güvenliği
/// `DOPPLER_TOKEN` bir **Service Token** olmalıdır (dp.st.* formatı).
/// Personal Access Token (dp.pt.*) kullanılmamalıdır — service token'lar
/// belirli bir proje/config'e kilitlidir ve daha güvenlidir.
pub struct DopplerSecretsProvider {
    /// Doppler Service Token
    token: String,
    /// Doppler API base URL (test için override edilebilir)
    base_url: String,
    /// HTTP client
    client: Client,
    /// Önbelleklenmiş secret'lar
    cache: tokio::sync::OnceCell<HashMap<String, String>>,
}

impl DopplerSecretsProvider {
    /// Ortam değişkenlerinden Doppler bağlantı bilgilerini okuyarak provider oluşturur.
    pub fn from_env() -> Result<Self, SecretsError> {
        let token = std::env::var("DOPPLER_TOKEN").map_err(|_| {
            SecretsError::AuthError("DOPPLER_TOKEN ortam değişkeni tanımlı değil".to_string())
        })?;

        // Service token formatı doğrulama (dp.st.* olmalı)
        if !token.starts_with("dp.st.") && !token.starts_with("dp.ct.") {
            tracing::warn!(
                "DOPPLER_TOKEN bir Service Token (dp.st.*) veya CLI Token (dp.ct.*) olmalıdır. \
                 Personal Access Token (dp.pt.*) kullanmaktan kaçının."
            );
        }

        let base_url = std::env::var("DOPPLER_API_URL")
            .unwrap_or_else(|_| "https://api.doppler.com".to_string());

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| {
                SecretsError::ConnectionError(format!("HTTP client oluşturulamadı: {e}"))
            })?;

        Ok(Self {
            token,
            base_url,
            client,
            cache: tokio::sync::OnceCell::new(),
        })
    }

    /// Doppler API'den tüm secret'ları çeker ve önbelleğe alır.
    async fn fetch_all(&self) -> Result<&HashMap<String, String>, SecretsError> {
        self.cache
            .get_or_try_init(|| async {
                let url = format!(
                    "{}/v3/configs/config/secrets/download?format=json",
                    self.base_url
                );

                debug!(url = %url, "Doppler'dan secret'lar çekiliyor...");

                let response = self
                    .client
                    .get(&url)
                    .bearer_auth(&self.token)
                    .header("Accept", "application/json")
                    .send()
                    .await
                    .map_err(|e| {
                        error!("Doppler bağlantı hatası: {:?}", e);
                        SecretsError::ConnectionError(format!("Doppler API'ye bağlanılamadı: {e}"))
                    })?;

                match response.status() {
                    s if s == reqwest::StatusCode::UNAUTHORIZED
                        || s == reqwest::StatusCode::FORBIDDEN =>
                    {
                        return Err(SecretsError::AuthError(
                            "Doppler token geçersiz veya yetkisiz. \
                             Service Token (dp.st.*) kullandığınızdan emin olun."
                                .to_string(),
                        ));
                    }
                    s if s == reqwest::StatusCode::NOT_FOUND => {
                        return Err(SecretsError::NotFound {
                            key: "Doppler project/config".to_string(),
                        });
                    }
                    s if !s.is_success() => {
                        let body = response.text().await.unwrap_or_default();
                        return Err(SecretsError::ProviderError(format!(
                            "Doppler API hatası (HTTP {s}): {body}"
                        )));
                    }
                    _ => {}
                }

                let secrets: HashMap<String, serde_json::Value> =
                    response.json().await.map_err(|e| {
                        SecretsError::ProviderError(format!("Doppler yanıtı parse edilemedi: {e}"))
                    })?;

                // Doppler bazen DOPPLER_* meta key'leri de döndürür, bunları filtrele
                let result: HashMap<String, String> = secrets
                    .into_iter()
                    .filter(|(k, _)| !k.starts_with("DOPPLER_"))
                    .map(|(k, v)| {
                        let val = match v {
                            serde_json::Value::String(s) => s,
                            other => other.to_string(),
                        };
                        (k, val)
                    })
                    .collect();

                debug!(
                    secret_count = result.len(),
                    "Doppler'dan {} secret başarıyla yüklendi",
                    result.len()
                );

                Ok(result)
            })
            .await
    }
}

#[async_trait]
impl SecretsProvider for DopplerSecretsProvider {
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
        "doppler"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_env_fails_without_token() {
        std::env::remove_var("DOPPLER_TOKEN");

        let result = DopplerSecretsProvider::from_env();
        match result {
            Err(SecretsError::AuthError(msg)) => {
                assert!(msg.contains("DOPPLER_TOKEN"));
            }
            Err(other) => panic!("Beklenen AuthError, alınan: {:?}", other),
            Ok(_) => panic!("Hata bekleniyordu ama başarı döndü"),
        }
    }

    #[test]
    fn from_env_succeeds_with_token() {
        std::env::set_var("DOPPLER_TOKEN", "dp.st.test_token_12345");

        let provider = DopplerSecretsProvider::from_env();
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.provider_name(), "doppler");
        assert_eq!(provider.base_url, "https://api.doppler.com");

        std::env::remove_var("DOPPLER_TOKEN");
    }
}
