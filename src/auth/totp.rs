use hmac::{Hmac, Mac};
use sha1::Sha1;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TotpError {
    #[error("Chave secreta inválida")]
    InvalidSecret,
    #[error("Erro de codificação HMAC")]
    HmacError,
}

// CORREÇÃO CRÍTICA:
// Usar `Sha1` real via `hmac::Hmac<sha1::Sha1>` conforme estipulado pela RFC 6238 / RFC 4226.
// Isso garante compatibilidade total com Google Authenticator, Authy e Bitwarden.
type HmacSha1 = Hmac<Sha1>;

/// Gera um código TOTP de 6 dígitos baseado no horário atual e segredo.
pub fn generate_totp_code(secret: &[u8], time_step_seconds: u64, current_timestamp: u64) -> Result<String, TotpError> {
    let counter = current_timestamp / time_step_seconds;
    let counter_bytes = counter.to_be_bytes();

    let mut mac = HmacSha1::new_from_slice(secret).map_err(|_| TotpError::InvalidSecret)?;
    mac.update(&counter_bytes);
    let result = mac.finalize().into_bytes();

    let offset = (result[result.len() - 1] & 0x0f) as usize;
    let binary_code = ((result[offset] as u32 & 0x7f) << 24)
        | ((result[offset + 1] as u32 & 0xff) << 16)
        | ((result[offset + 2] as u32 & 0xff) << 8)
        | (result[offset + 3] as u32 & 0xff);

    let otp = binary_code % 1_000_000;
    Ok(format!("{:06}", otp))
}

/// Verifica se um código TOTP é válido considerando tolerância de janela (ex: +- 30 segundos).
pub fn verify_totp_code(secret: &[u8], code: &str, current_timestamp: u64, window_steps: u64) -> bool {
    let step = 30u64;
    for i in 0..=window_steps {
        // Testar timestamp atual, anterior e posterior dentro da janela de tolerância
        let test_timestamps = [
            current_timestamp.saturating_add(i * step),
            current_timestamp.saturating_sub(i * step),
        ];

        for &ts in &test_timestamps {
            if let Ok(generated) = generate_totp_code(secret, step, ts) {
                if generated == code {
                    return true;
                }
            }
        }
    }
    false
}
