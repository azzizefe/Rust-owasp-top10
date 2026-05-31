// crates/core/src/config.rs

use std::str::FromStr;

use crate::secrets::SecretsProvider;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Vulnerable,
    Secure,
}

impl FromStr for AppMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "vulnerable" => Ok(AppMode::Vulnerable),
            "secure" => Ok(AppMode::Secure),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub bind_addr: String,
    pub mode: AppMode,
    pub session_secret: String,
    pub cookie_secure: bool,
}

impl Config {
    /// Konfigürasyonu verilen `SecretsProvider` üzerinden yükler.
    ///
    /// **Secret değerler** (DATABASE_URL, SESSION_SECRET) provider'dan çekilir.
    /// **Non-secret konfigürasyon** (APP_MODE, BIND_ADDR, COOKIE_SECURE) hâlâ
    /// ortam değişkenlerinden okunur — bunlar hassas veri değil, deployment parametresidir.
    pub async fn load(provider: &dyn SecretsProvider) -> Result<Self, String> {
        // ── Secret değerler (provider'dan) ──
        let database_url = provider.get_secret("DATABASE_URL").await.map_err(|e| {
            format!(
                "DATABASE_URL secret alınamadı (provider: {}): {}",
                provider.provider_name(),
                e
            )
        })?;

        let session_secret = provider
            .get_secret("SESSION_SECRET")
            .await
            .unwrap_or_default();

        // ── Non-secret konfigürasyon (ortam değişkenlerinden) ──
        let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());

        let mode_str = std::env::var("APP_MODE").unwrap_or_default();
        // Güvenli Varsayılan İlkesi (Secure by Default): Geçersiz veya eksikse 'Secure' mod seçilir
        #[allow(unused_mut)]
        let mut mode = AppMode::from_str(&mode_str).unwrap_or(AppMode::Secure);

        // 🔒 Fail-Safe / Hardening: Production release environments MUST never run in Vulnerable mode.
        // Compile-time debug_assertions check protects against compiling/running vulnerable code in production.
        #[cfg(not(debug_assertions))]
        {
            if mode == AppMode::Vulnerable {
                eprintln!("[⚠️ SECURITY ALERT] Production/Release binary detected! 'vulnerable' mode is strictly forbidden. Overriding APP_MODE to 'secure' (Fail-Safe activated).");
                mode = AppMode::Secure;
            }
        }

        if mode == AppMode::Secure
            && (session_secret.is_empty()
                || session_secret
                    == "CHANGE_ME_64_RANDOM_BYTES_FOR_SESSION_SIGNING_AND_SECURITY_KEY")
        {
            return Err(
                "SESSION_SECRET environment variable is missing or insecure in SECURE mode!"
                    .to_string(),
            );
        }

        let session_secret = if session_secret.is_empty() {
            "CHANGE_ME_64_RANDOM_BYTES_FOR_SESSION_SIGNING_AND_SECURITY_KEY".to_string()
        } else {
            session_secret
        };

        let cookie_secure_str = std::env::var("COOKIE_SECURE").unwrap_or_default();
        let cookie_secure = if mode == AppMode::Secure {
            cookie_secure_str.to_lowercase() != "false"
        } else {
            false
        };

        Ok(Config {
            database_url,
            bind_addr,
            mode,
            session_secret,
            cookie_secure,
        })
    }

    /// Geriye dönük uyumlu kısayol: `EnvSecretsProvider` ile `load()` çağırır.
    ///
    /// Mevcut kullanım noktalarının bozulmaması için korunmuştur.
    /// Yeni kod `Config::load(provider)` kullanmalıdır.
    pub async fn from_env() -> Result<Self, String> {
        let provider = crate::secrets::EnvSecretsProvider::new();
        Self::load(&provider).await
    }
}
