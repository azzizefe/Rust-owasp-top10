// crates/core/src/secrets_aws.rs — AWS Secrets Manager Provider
//
// AWS Secrets Manager'dan secret okur.
// Gerekli ortam değişkenleri:
//   - AWS_REGION         : AWS region (varsayılan: "eu-west-1")
//   - AWS_SECRET_NAME    : Secret adı (varsayılan: "owasp-lab/production")
//   - AWS_ACCESS_KEY_ID  : (opsiyonel) Standart AWS credential chain'den gelir
//   - AWS_SECRET_ACCESS_KEY : (opsiyonel) Standart AWS credential chain'den gelir
//
// AWS credential chain sıralaması:
//   1. Ortam değişkenleri (AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY)
//   2. AWS credentials dosyası (~/.aws/credentials)
//   3. IAM Role (EC2/ECS/Lambda üzerinde çalışırken)
//   4. ECS Task Role
//   5. Web Identity Token (EKS Pod Identity)

use async_trait::async_trait;
use std::collections::HashMap;
use tracing::{debug, error};

use crate::secrets::{SecretsError, SecretsProvider};

/// AWS Secrets Manager üzerinden secret okuyan provider.
///
/// # Secret Formatı
/// AWS Secrets Manager'daki secret değeri bir JSON objesi olmalıdır:
/// ```json
/// {
///   "DATABASE_URL": "postgres://user:pass@host:5432/db",
///   "SESSION_SECRET": "super-secret-key-64-bytes-long..."
/// }
/// ```
///
/// # Kimlik Doğrulama
/// Standart AWS credential chain kullanır. Production'da
/// IAM Role veya ECS Task Role tercih edilmelidir.
pub struct AwsSecretsProvider {
    /// AWS SDK Secrets Manager client
    client: aws_sdk_secretsmanager::Client,
    /// Secrets Manager'daki secret adı
    secret_name: String,
    /// Önbelleklenmiş secret'lar
    cache: tokio::sync::OnceCell<HashMap<String, String>>,
}

impl AwsSecretsProvider {
    /// AWS SDK config'ini yükleyerek provider oluşturur.
    pub async fn new() -> Result<Self, SecretsError> {
        let region = std::env::var("AWS_REGION").unwrap_or_else(|_| "eu-west-1".to_string());
        let secret_name =
            std::env::var("AWS_SECRET_NAME").unwrap_or_else(|_| "owasp-lab/production".to_string());

        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(aws_config::Region::new(region))
            .load()
            .await;

        let client = aws_sdk_secretsmanager::Client::new(&config);

        Ok(Self {
            client,
            secret_name,
            cache: tokio::sync::OnceCell::new(),
        })
    }

    /// AWS Secrets Manager'dan tüm secret'ları çeker ve önbelleğe alır.
    async fn fetch_all(&self) -> Result<&HashMap<String, String>, SecretsError> {
        self.cache
            .get_or_try_init(|| async {
                debug!(
                    secret_name = %self.secret_name,
                    "AWS Secrets Manager'dan secret çekiliyor..."
                );

                let response = self
                    .client
                    .get_secret_value()
                    .secret_id(&self.secret_name)
                    .send()
                    .await
                    .map_err(|e| {
                        error!("AWS Secrets Manager hatası: {:?}", e);

                        let err_msg = format!("{e}");
                        if err_msg.contains("ResourceNotFoundException") {
                            return SecretsError::NotFound {
                                key: self.secret_name.clone(),
                            };
                        }
                        if err_msg.contains("AccessDeniedException")
                            || err_msg.contains("UnrecognizedClientException")
                        {
                            return SecretsError::AuthError(format!(
                                "AWS kimlik doğrulama hatası: {e}"
                            ));
                        }
                        SecretsError::ProviderError(format!("AWS Secrets Manager hatası: {e}"))
                    })?;

                let secret_string = response.secret_string().ok_or_else(|| {
                    SecretsError::ProviderError(
                        "Secret değeri string formatında değil (binary secret desteklenmiyor)"
                            .to_string(),
                    )
                })?;

                // JSON olarak parse et
                let secrets: HashMap<String, serde_json::Value> =
                    serde_json::from_str(secret_string).map_err(|e| {
                        SecretsError::ProviderError(format!(
                            "Secret değeri geçerli JSON değil: {e}. \
                             AWS secret'ı {{\"KEY\": \"VALUE\"}} formatında olmalıdır."
                        ))
                    })?;

                let result: HashMap<String, String> = secrets
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
                    secret_count = result.len(),
                    "AWS Secrets Manager'dan {} secret başarıyla yüklendi",
                    result.len()
                );

                Ok(result)
            })
            .await
    }
}

#[async_trait]
impl SecretsProvider for AwsSecretsProvider {
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
        "aws"
    }
}
