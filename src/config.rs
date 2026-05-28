// src/config.rs

use std::str::FromStr;

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
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        // .env dosyasını oku (hata verirse yoksay, environment'tan da gelebilir)
        let _ = dotenvy::dotenv();

        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| "DATABASE_URL environment variable is missing!".to_string())?;

        let bind_addr = std::env::var("BIND_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:8080".to_string());

        let mode_str = std::env::var("APP_MODE").unwrap_or_default();
        // Güvenli Varsayılan İlkesi (Secure by Default): Geçersiz veya eksikse 'Secure' mod seçilir
        let mode = AppMode::from_str(&mode_str).unwrap_or(AppMode::Secure);

        let session_secret = std::env::var("SESSION_SECRET")
            .unwrap_or_else(|_| "CHANGE_ME_64_RANDOM_BYTES_FOR_SESSION_SIGNING_AND_SECURITY_KEY".to_string());

        Ok(Config {
            database_url,
            bind_addr,
            mode,
            session_secret,
        })
    }
}
