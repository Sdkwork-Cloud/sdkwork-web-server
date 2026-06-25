use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use rand::RngCore;

use crate::{AcmeServiceError, AcmeServiceResult};

const NONCE_LEN: usize = 12;

pub fn encrypt_secret(key: &[u8], plaintext: &[u8]) -> AcmeServiceResult<String> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|error| AcmeServiceError::Encryption(error.to_string()))?;
    let mut nonce_bytes = [0_u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|error| AcmeServiceError::Encryption(error.to_string()))?;
    let mut payload = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    payload.extend_from_slice(&nonce_bytes);
    payload.extend_from_slice(&ciphertext);
    Ok(STANDARD.encode(payload))
}

pub fn decrypt_secret(key: &[u8], encoded: &str) -> AcmeServiceResult<Vec<u8>> {
    let payload = STANDARD
        .decode(encoded)
        .map_err(|error| AcmeServiceError::Encryption(error.to_string()))?;
    if payload.len() <= NONCE_LEN {
        return Err(AcmeServiceError::Encryption(
            "encrypted payload too short".to_string(),
        ));
    }
    let (nonce_bytes, ciphertext) = payload.split_at(NONCE_LEN);
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|error| AcmeServiceError::Encryption(error.to_string()))?;
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|error| AcmeServiceError::Encryption(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_encrypt_decrypt() {
        let key = [7_u8; 32];
        let encoded = encrypt_secret(&key, b"private-key-pem").expect("encrypt");
        let decoded = decrypt_secret(&key, &encoded).expect("decrypt");
        assert_eq!(decoded, b"private-key-pem");
    }
}
