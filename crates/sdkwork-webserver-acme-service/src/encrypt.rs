//! 机密加解密薄包装，委托到 sdkwork-utils-rust 的 AES-256-GCM 实现。
//!
//! 保留本模块以维持 `AcmeServiceError` 错误类型契约，
//! 同时消除与 utils crate 的重复加密逻辑。

use sdkwork_utils_rust::{aes_gcm_decrypt, aes_gcm_encrypt};

use crate::{AcmeServiceError, AcmeServiceResult};

pub fn encrypt_secret(key: &[u8], plaintext: &[u8]) -> AcmeServiceResult<String> {
    aes_gcm_encrypt(key, plaintext).map_err(|error| AcmeServiceError::Encryption(error.to_string()))
}

pub fn decrypt_secret(key: &[u8], encoded: &str) -> AcmeServiceResult<Vec<u8>> {
    aes_gcm_decrypt(key, encoded).map_err(|error| AcmeServiceError::Encryption(error.to_string()))
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
