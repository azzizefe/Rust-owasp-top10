// crates/core/src/crypto.rs

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::Engine;
use hkdf::Hkdf;
use hmac::{Hmac, Mac};
use rand::Rng;
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

#[derive(thiserror::Error, Debug)]
pub enum CryptoError {
    #[error("Base64 decode hatası: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("AES-GCM deşifreleme hatası")]
    Decryption,
    #[error("HMAC imza doğrulama hatası (veri tahrif edilmiş olabilir)")]
    InvalidSignature,
    #[error("Geçersiz veri formatı")]
    InvalidFormat,
}

/// Gelen anahtardan (session_secret) HKDF-SHA256 ile signing_key ve encryption_key türetir
pub fn derive_keys(secret: &[u8]) -> ( [u8; 32], [u8; 32] ) {
    let hk = Hkdf::<Sha256>::new(None, secret);
    
    let mut signing_key = [0u8; 32];
    let mut encryption_key = [0u8; 32];
    
    hk.expand(b"session-signing-key-v1", &mut signing_key)
        .expect("HKDF expand signing_key failed");
    hk.expand(b"session-encryption-key-v1", &mut encryption_key)
        .expect("HKDF expand encryption_key failed");
        
    (signing_key, encryption_key)
}

/// AES-GCM ile veriyi şifreler, 96-bit nonce ile birleştirip Base64 kodlu döner
pub fn encrypt_cookie(key: &[u8; 32], plaintext: &str) -> String {
    let cipher = Aes256Gcm::new(key.into());
    
    // 96-bit (12 byte) random nonce üretimi
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    // Şifreleme işlemi (tag otomatik olarak ciphertext'in sonuna eklenir)
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .expect("AES-GCM encryption failed");
        
    // nonce || ciphertext + tag birleşimi
    let mut combined = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);
    
    base64::engine::general_purpose::STANDARD.encode(&combined)
}

/// Base64 kodlu veriyi deşifre eder
pub fn decrypt_cookie(key: &[u8; 32], base64_ciphertext: &str) -> Result<String, CryptoError> {
    let combined = base64::engine::general_purpose::STANDARD.decode(base64_ciphertext)?;
    
    if combined.len() < 12 {
        return Err(CryptoError::InvalidFormat);
    }
    
    // Nonce ve ciphertext'i ayır
    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let cipher = Aes256Gcm::new(key.into());
    
    let decrypted_bytes = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| CryptoError::Decryption)?;
        
    String::from_utf8(decrypted_bytes).map_err(|_| CryptoError::InvalidFormat)
}

/// Değeri HMAC-SHA256 ile imzalar ve value.signature formatında döner
pub fn sign_cookie(key: &[u8; 32], value: &str) -> String {
    let mut mac = <HmacSha256 as Mac>::new_from_slice(key).expect("HMAC key size is fixed to 32 bytes");
    mac.update(value.as_bytes());
    let result = mac.finalize();
    let signature = base64::engine::general_purpose::STANDARD.encode(result.into_bytes());
    
    format!("{}.{}", value, signature)
}

/// value.signature formatındaki çerezi doğrular, tahrif edilmemişse asıl değeri döner
pub fn verify_cookie(key: &[u8; 32], signed_value: &str) -> Result<String, CryptoError> {
    let parts: Vec<&str> = signed_value.split('.').collect();
    if parts.len() != 2 {
        return Err(CryptoError::InvalidFormat);
    }
    
    let value = parts[0];
    let signature_b64 = parts[1];
    
    let signature = base64::engine::general_purpose::STANDARD.decode(signature_b64)?;
    
    let mut mac = <HmacSha256 as Mac>::new_from_slice(key).expect("HMAC key size is fixed to 32 bytes");
    mac.update(value.as_bytes());
    
    mac.verify_slice(&signature)
        .map_err(|_| CryptoError::InvalidSignature)?;
        
    Ok(value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hkdf_deterministic() {
        let secret = b"my_super_secret_session_key_64_bytes_long_value_here";
        let (sign1, enc1) = derive_keys(secret);
        let (sign2, enc2) = derive_keys(secret);
        
        assert_eq!(sign1, sign2);
        assert_eq!(enc1, enc2);
    }

    #[test]
    fn test_aead_round_trip() {
        let secret = b"some_other_secret_value_for_testing_purposes_only";
        let (_, enc_key) = derive_keys(secret);
        
        let plaintext = "secret-session-token-123456";
        let encrypted = encrypt_cookie(&enc_key, plaintext);
        
        // Şifreli veri düz metni içermemeli
        assert!(!encrypted.contains(plaintext));
        
        let decrypted = decrypt_cookie(&enc_key, &encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_hmac_tamper_proof() {
        let secret = b"some_secret_key_used_to_generate_keys_for_signing";
        let (sign_key, _) = derive_keys(secret);
        
        let original_value = "my-encrypted-cookie-data";
        let signed = sign_cookie(&sign_key, original_value);
        
        // Doğrulama başarılı olmalı
        let verified = verify_cookie(&sign_key, &signed).unwrap();
        assert_eq!(verified, original_value);
        
        // Tahrif edilmiş çerez doğrulamadan geçmemeli
        let tampered = signed.replace("my-encrypted", "my-attacker");
        assert!(verify_cookie(&sign_key, &tampered).is_err());
    }
}
