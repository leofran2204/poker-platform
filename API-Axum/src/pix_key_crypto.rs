// pix_key_crypto.rs — Cifra da chave PIX do payout worker (AES-256-GCM).
//
// A chave em claro existe somente na memória do request que cria o saque e
// dentro do worker no momento do envio. No banco vai apenas o blob
// `nonce_hex.cipher_hex`; o segredo de 32 bytes vem só de
// `DEPIX_PIXKEY_ENC_KEY` (64 hex chars, sem espaços). Sem o segredo, cifrar e
// decifrar falham fechado — nunca em claro, nunca em log.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};

const ENV_VAR: &str = "DEPIX_PIXKEY_ENC_KEY";

fn decode_hex(bytes: &str) -> Result<Vec<u8>, String> {
    if !bytes.len().is_multiple_of(2) {
        return Err("pix-key crypto: hex length must be even".to_string());
    }
    (0..bytes.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&bytes[i..i + 2], 16)
                .map_err(|_| "pix-key crypto: invalid hex".to_string())
        })
        .collect()
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Carrega a chave de 32 bytes do ambiente (fail-closed).
pub fn load_key() -> Result<[u8; 32], String> {
    let raw = std::env::var(ENV_VAR)
        .map_err(|_| format!("{ENV_VAR} is not set: automatic PIX payouts are disabled"))?;
    let raw = raw.trim();
    if raw.len() != 64 {
        return Err(format!(
            "{ENV_VAR} must be 64 hex chars (32 bytes): automatic PIX payouts are disabled"
        ));
    }
    let decoded = decode_hex(raw).map_err(|_| {
        format!("{ENV_VAR} must be 64 hex chars (32 bytes): automatic PIX payouts are disabled")
    })?;
    let mut key = [0u8; 32];
    key.copy_from_slice(&decoded);
    Ok(key)
}

/// Cifra o JSON `{"pix_key":..,"tax_number":..}` com nonce aleatório (12 bytes
/// tirados de um UUID v4). Retorna `nonce_hex.cipher_hex`.
pub fn encrypt_blob(key: &[u8; 32], plaintext: &str) -> Result<String, String> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| "pix-key crypto: invalid key length".to_string())?;
    let uuid = uuid::Uuid::new_v4().into_bytes();
    let nonce = Nonce::from_slice(&uuid[..12]);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|_| "pix-key crypto: encryption failed".to_string())?;
    Ok(format!(
        "{}.{}",
        encode_hex(&uuid[..12]),
        encode_hex(&ciphertext)
    ))
}

/// Decifra um blob `nonce_hex.cipher_hex`. Qualquer adulteração falha fechado.
pub fn decrypt_blob(key: &[u8; 32], blob: &str) -> Result<String, String> {
    let (nonce_hex, cipher_hex) = blob
        .split_once('.')
        .ok_or_else(|| "pix-key crypto: malformed blob".to_string())?;
    let nonce_bytes =
        decode_hex(nonce_hex).map_err(|_| "pix-key crypto: malformed blob".to_string())?;
    if nonce_bytes.len() != 12 {
        return Err("pix-key crypto: malformed blob".to_string());
    }
    let cipher_bytes =
        decode_hex(cipher_hex).map_err(|_| "pix-key crypto: malformed blob".to_string())?;
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| "pix-key crypto: invalid key length".to_string())?;
    let plaintext = cipher
        .decrypt(Nonce::from_slice(&nonce_bytes), cipher_bytes.as_ref())
        .map_err(|_| "pix-key crypto: decryption failed".to_string())?;
    String::from_utf8(plaintext).map_err(|_| "pix-key crypto: decryption failed".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; 32] {
        [0x2a; 32]
    }

    #[test]
    fn roundtrip_keeps_pix_key_and_tax_number() {
        let payload = r#"{"pix_key":"leofran@exemplo.com","tax_number":"26814306824"}"#;
        let blob = encrypt_blob(&test_key(), payload).unwrap();
        assert!(blob.contains('.'));
        assert!(!blob.contains("leofran"));
        assert_eq!(decrypt_blob(&test_key(), &blob).unwrap(), payload);
    }

    #[test]
    fn nonces_are_unique_per_encryption() {
        let a = encrypt_blob(&test_key(), "mesmo-texto").unwrap();
        let b = encrypt_blob(&test_key(), "mesmo-texto").unwrap();
        assert_ne!(a, b);
        assert_eq!(decrypt_blob(&test_key(), &a).unwrap(), "mesmo-texto");
        assert_eq!(decrypt_blob(&test_key(), &b).unwrap(), "mesmo-texto");
    }

    #[test]
    fn tampered_blob_fails_closed() {
        let blob = encrypt_blob(&test_key(), "segredo").unwrap();
        let mut tampered = blob.clone();
        tampered.pop();
        tampered.push(if tampered.ends_with('0') { '1' } else { '0' });
        assert!(decrypt_blob(&test_key(), &tampered).is_err());
        assert!(decrypt_blob(&[0x00; 32], &blob).is_err());
        assert!(decrypt_blob(&test_key(), "sem-separador").is_err());
    }

    #[test]
    fn loader_rejects_missing_or_malformed_key() {
        assert!(std::env::var(ENV_VAR).is_err());
        assert!(load_key().is_err());
    }
}
