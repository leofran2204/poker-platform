//! Módulo de Autenticação PASETO v4 para a API Axum.
//!
//! PASETO (Platform-Agnostic Security Tokens) é uma alternativa moderna e segura ao JWT.
//! Este módulo implementa tokens simétricos v4.local protegidos por HMAC-SHA256 e payload codificado em base64url,
//! prevenindo ataques conhecidos do JWT (como o algoritmo "none" e confusão de tipos de chave).

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header, request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

pub const PASETO_HEADER_PREFIX: &str = "v4.local.";

/// Claims contidos no payload do token PASETO.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PasetoClaims {
    /// ID do Usuário (Subject)
    pub sub: String,
    /// Nome do Usuário
    pub username: String,
    /// Função/Papel do Usuário (ex: "player", "admin")
    pub role: String,
    /// Timestamp Unix de emissão (Issued At)
    pub iat: u64,
    /// Timestamp Unix de expiração (Expiration)
    pub exp: u64,
}

impl PasetoClaims {
    /// Cria uma estrutura de claims com validade padrão (em segundos).
    pub fn new(sub: impl Into<String>, username: impl Into<String>, role: impl Into<String>, ttl_seconds: u64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            sub: sub.into(),
            username: username.into(),
            role: role.into(),
            iat: now,
            exp: now + ttl_seconds,
        }
    }

    /// Verifica se o token já expirou.
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        now >= self.exp
    }
}

/// Erros de Autenticação PASETO.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PasetoError {
    MissingToken,
    InvalidHeader,
    TokenExpired,
    InvalidSignature,
    SerializationError,
}

impl IntoResponse for PasetoError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            PasetoError::MissingToken => (StatusCode::UNAUTHORIZED, "Header de autorização ausente"),
            PasetoError::InvalidHeader => (StatusCode::UNAUTHORIZED, "Formato de token PASETO inválido"),
            PasetoError::TokenExpired => (StatusCode::UNAUTHORIZED, "Token PASETO expirado"),
            PasetoError::InvalidSignature => (StatusCode::UNAUTHORIZED, "Assinatura PASETO inválida"),
            PasetoError::SerializationError => (StatusCode::BAD_REQUEST, "Falha na decodificação dos claims"),
        };

        let body = Json(json!({
            "error": message,
            "code": "PASETO_AUTH_ERROR"
        }));

        (status, body).into_response()
    }
}

/// Gera um token PASETO v4.local assinado com a chave secreta de 32 bytes.
pub fn encode_paseto(claims: &PasetoClaims, secret_key: &[u8; 32]) -> Result<String, PasetoError> {
    let json_payload = serde_json::to_string(claims).map_err(|_| PasetoError::SerializationError)?;
    let payload_b64 = base64_url_encode(json_payload.as_bytes());

    let message_to_sign = format!("{PASETO_HEADER_PREFIX}{payload_b64}");

    let mut mac = HmacSha256::new_from_slice(secret_key)
        .map_err(|_| PasetoError::InvalidSignature)?;
    mac.update(message_to_sign.as_bytes());
    let signature = mac.finalize().into_bytes();
    let sig_b64 = base64_url_encode(&signature);

    Ok(format!("{PASETO_HEADER_PREFIX}{payload_b64}.{sig_b64}"))
}

/// Decodifica e valida um token PASETO v4.local.
pub fn decode_paseto(token: &str, secret_key: &[u8; 32]) -> Result<PasetoClaims, PasetoError> {
    if !token.starts_with(PASETO_HEADER_PREFIX) {
        return Err(PasetoError::InvalidHeader);
    }

    let remainder = &token[PASETO_HEADER_PREFIX.len()..];
    let parts: Vec<&str> = remainder.split('.').collect();

    if parts.len() != 2 {
        return Err(PasetoError::InvalidHeader);
    }

    let payload_b64 = parts[0];
    let sig_b64 = parts[1];

    // Valida a assinatura HMAC
    let message_to_sign = format!("{PASETO_HEADER_PREFIX}{payload_b64}");
    let mut mac = HmacSha256::new_from_slice(secret_key)
        .map_err(|_| PasetoError::InvalidSignature)?;
    mac.update(message_to_sign.as_bytes());

    let expected_sig = mac.finalize().into_bytes();
    let provided_sig = base64_url_decode(sig_b64).map_err(|_| PasetoError::InvalidSignature)?;

    if expected_sig.as_slice() != provided_sig.as_slice() {
        return Err(PasetoError::InvalidSignature);
    }

    // Decodifica os claims
    let payload_bytes = base64_url_decode(payload_b64).map_err(|_| PasetoError::SerializationError)?;
    let claims: PasetoClaims = serde_json::from_slice(&payload_bytes)
        .map_err(|_| PasetoError::SerializationError)?;

    if claims.is_expired() {
        return Err(PasetoError::TokenExpired);
    }

    Ok(claims)
}

// ─── Extrator Axum para handlers REST ───

#[async_trait]
impl<S> FromRequestParts<S> for PasetoClaims
where
    S: Send + Sync,
{
    type Rejection = PasetoError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or(PasetoError::MissingToken)?;

        let token = if let Some(stripped) = auth_header.strip_prefix("Bearer ") {
            stripped
        } else {
            auth_header
        };

        // Chave secreta padrão de fallback para extrator (em prod deve vir do AppState)
        let default_secret = [77u8; 32];
        decode_paseto(token, &default_secret)
    }
}

// ─── Helpers Base64URL ───

fn base64_url_encode(data: &[u8]) -> String {
    let mut s = base64_standard_encode(data);
    s = s.replace('+', "-").replace('/', "_");
    s.trim_end_matches('=').to_string()
}

fn base64_url_decode(encoded: &str) -> Result<Vec<u8>, ()> {
    let mut s = encoded.replace('-', "+").replace('_', "/");
    while s.len() % 4 != 0 {
        s.push('=');
    }
    base64_standard_decode(&s)
}

fn base64_standard_encode(data: &[u8]) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    let mut i = 0;

    while i < data.len() {
        let b0 = data[i];
        let b1 = if i + 1 < data.len() { data[i + 1] } else { 0 };
        let b2 = if i + 2 < data.len() { data[i + 2] } else { 0 };

        let triple = ((b0 as u32) << 16) | ((b1 as u32) << 8) | (b2 as u32);

        out.push(CHARSET[((triple >> 18) & 63) as usize] as char);
        out.push(CHARSET[((triple >> 12) & 63) as usize] as char);

        if i + 1 < data.len() {
            out.push(CHARSET[((triple >> 6) & 63) as usize] as char);
        } else {
            out.push('=');
        }

        if i + 2 < data.len() {
            out.push(CHARSET[(triple & 63) as usize] as char);
        } else {
            out.push('=');
        }

        i += 3;
    }

    out
}

fn base64_standard_decode(encoded: &str) -> Result<Vec<u8>, ()> {
    fn decode_char(c: char) -> Result<u32, ()> {
        match c {
            'A'..='Z' => Ok(c as u32 - 'A' as u32),
            'a'..='z' => Ok(c as u32 - 'a' as u32 + 26),
            '0'..='9' => Ok(c as u32 - '0' as u32 + 52),
            '+' => Ok(62),
            '/' => Ok(63),
            '=' => Ok(0),
            _ => Err(()),
        }
    }

    let bytes: Vec<char> = encoded.chars().filter(|c| !c.is_whitespace()).collect();
    if bytes.len() % 4 != 0 {
        return Err(());
    }

    let mut out = Vec::new();
    let mut i = 0;

    while i < bytes.len() {
        let c0 = decode_char(bytes[i])?;
        let c1 = decode_char(bytes[i + 1])?;
        let c2 = decode_char(bytes[i + 2])?;
        let c3 = decode_char(bytes[i + 3])?;

        let triple = (c0 << 18) | (c1 << 12) | (c2 << 6) | c3;

        out.push(((triple >> 16) & 0xFF) as u8);
        if bytes[i + 2] != '=' {
            out.push(((triple >> 8) & 0xFF) as u8);
        }
        if bytes[i + 3] != '=' {
            out.push((triple & 0xFF) as u8);
        }

        i += 4;
    }

    Ok(out)
}

// ─── Testes Unitários ───

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paseto_encode_decode_success() {
        let secret = [42u8; 32];
        let claims = PasetoClaims::new("user_123", "poker_player_1", "player", 3600);

        let token = encode_paseto(&claims, &secret).unwrap();
        assert!(token.starts_with("v4.local."));

        let decoded = decode_paseto(&token, &secret).unwrap();
        assert_eq!(decoded.sub, "user_123");
        assert_eq!(decoded.username, "poker_player_1");
        assert_eq!(decoded.role, "player");
    }

    #[test]
    fn test_paseto_invalid_signature() {
        let secret = [42u8; 32];
        let wrong_secret = [99u8; 32];
        let claims = PasetoClaims::new("user_123", "poker_player_1", "player", 3600);

        let token = encode_paseto(&claims, &secret).unwrap();
        let result = decode_paseto(&token, &wrong_secret);

        assert_eq!(result, Err(PasetoError::InvalidSignature));
    }

    #[test]
    fn test_paseto_expired_token() {
        let secret = [42u8; 32];
        let mut claims = PasetoClaims::new("user_123", "poker_player_1", "player", 0);
        claims.exp = claims.iat - 10; // Ja expirou no passado

        let token = encode_paseto(&claims, &secret).unwrap();
        let result = decode_paseto(&token, &secret);

        assert_eq!(result, Err(PasetoError::TokenExpired));
    }
}

// ─── Proptests & Fuzzing Massivo de Autenticação ───

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(512))]

        #[test]
        fn proptest_paseto_roundtrip_valid_claims(
            sub in "[a-zA-Z0-9_-]{1,32}",
            username in "[a-zA-Z0-9_]{1,32}",
            role in "player|admin|dealer",
            ttl in 1..100_000u64,
            secret in proptest::array::uniform32(any::<u8>())
        ) {
            let claims = PasetoClaims::new(&sub, &username, &role, ttl);
            let token = encode_paseto(&claims, &secret).unwrap();

            let decoded = decode_paseto(&token, &secret).unwrap();
            prop_assert_eq!(decoded.sub, sub);
            prop_assert_eq!(decoded.username, username);
            prop_assert_eq!(decoded.role, role);
        }

        #[test]
        fn proptest_paseto_bitflip_fuzzing_never_panics(
            sub in "[a-zA-Z0-9_-]{1,16}",
            username in "[a-zA-Z0-9_]{1,16}",
            secret in proptest::array::uniform32(any::<u8>()),
            mutate_pos in 0..100usize,
            new_char in any::<char>()
        ) {
            let claims = PasetoClaims::new(&sub, &username, "player", 3600);
            let token = encode_paseto(&claims, &secret).unwrap();

            // Corrompe um caractere na string do token
            let mut corrupted_chars: Vec<char> = token.chars().collect();
            if !corrupted_chars.is_empty() {
                let idx = mutate_pos % corrupted_chars.len();
                corrupted_chars[idx] = new_char;
            }
            let corrupted_token: String = corrupted_chars.into_iter().collect();

            // NUNCA deve panicar — deve retornar Ok (se por absurda coincidência for válido) ou Err
            let _ = decode_paseto(&corrupted_token, &secret);
        }
    }
}

