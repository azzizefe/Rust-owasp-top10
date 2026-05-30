// crates/core/src/lib.rs

pub mod auth;
pub mod config;
pub mod crypto;
pub mod db;
pub mod error;
pub mod models;
pub mod secrets;
#[cfg(feature = "aws")]
pub mod secrets_aws;
#[cfg(feature = "doppler")]
pub mod secrets_doppler;
#[cfg(feature = "vault")]
pub mod secrets_vault;
pub mod session;
