//! Envio de e-mails de verificação / boas-vindas.
//!
//! Providers (`EMAIL_PROVIDER`):
//! - `log` (padrão): grava no tracing — só para dev/lab
//! - `resend`: envia via [Resend](https://resend.com) API (`RESEND_API_KEY`, `EMAIL_FROM`)
//!
//! Falhas do provider real nunca expõem o código em logs.

use hmac::{Hmac, Mac};
use serde_json::json;
use sha2::Sha256;
use std::env;
use std::time::Duration;

type HmacSha256 = Hmac<Sha256>;

/// Validade do código em segundos (15 minutos).
pub const CODE_TTL_SECS: u64 = 15 * 60;

/// Gera código numérico de 6 dígitos com CSPRNG e rejection sampling.
pub fn generate_numeric_code() -> String {
    const RANGE: u32 = 1_000_000;
    const UNBIASED_ZONE: u32 = u32::MAX - (u32::MAX % RANGE);

    loop {
        let random = uuid::Uuid::new_v4();
        let bytes = random.as_bytes();
        let sample = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        if sample < UNBIASED_ZONE {
            return format!("{:06}", sample % RANGE);
        }
    }
}

/// HMAC com pepper impede força bruta offline caso somente a tabela de códigos
/// seja exposta. Em produção, o boot guardian exige um pepper dedicado.
pub fn hash_code(code: &str) -> String {
    let pepper = env::var("EMAIL_CODE_PEPPER")
        .unwrap_or_else(|_| "development-email-code-pepper".to_string());
    let mut mac =
        HmacSha256::new_from_slice(pepper.as_bytes()).expect("HMAC accepts keys of any size");
    mac.update(b"zero-tilt-email-v2\0");
    mac.update(code.as_bytes());
    format!("{:x}", mac.finalize().into_bytes())
}

pub fn codes_equal_hash(code: &str, stored_hash: &str) -> bool {
    use subtle::ConstantTimeEq;
    let computed = hash_code(code);
    computed.as_bytes().ct_eq(stored_hash.as_bytes()).into()
}

/// Subject + texto plano + HTML do e-mail de boas-vindas.
pub fn build_welcome_message(username: &str, code: &str) -> (String, String, String) {
    let subject = "♠ Zero Tilt — confirme sua conta e puxe a cadeira".to_string();
    let text = format!(
        r#"Olá, {username}!

Bem-vindo à sala Zero Tilt Poker.

Este não é só um e-mail de "clique aqui". É o dealer pedindo para você
confirmar que a cadeira é realmente sua — e-mail verificado, jogo limpo.

Seu código de verificação (válido por 15 minutos):

    {code}

Cole esse código na tela de verificação do site e libere o lobby.
Se você não criou uma conta, ignore esta mensagem.

— A casa Zero Tilt
  Texas Hold'em · espírito Full Tilt · motor em Rust
"#,
        username = username,
        code = code
    );

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="pt-BR">
<head><meta charset="utf-8"><title>Zero Tilt</title></head>
<body style="margin:0;padding:0;background:#0f2a0f;font-family:Segoe UI,Tahoma,sans-serif;color:#e8e0d0;">
  <table width="100%" cellpadding="0" cellspacing="0" style="background:#0f2a0f;padding:32px 16px;">
    <tr><td align="center">
      <table width="520" cellpadding="0" cellspacing="0" style="background:#1e4a1e;border:2px solid #8b6914;border-radius:4px;">
        <tr><td style="padding:24px 28px;border-bottom:2px solid #8b6914;">
          <div style="font-size:12px;letter-spacing:0.15em;color:#e8d48b;text-transform:uppercase;">Zero Tilt Poker</div>
          <h1 style="margin:8px 0 0;font-size:22px;color:#d4a843;">Puxe a cadeira, {username}</h1>
        </td></tr>
        <tr><td style="padding:24px 28px;font-size:15px;line-height:1.55;color:#e8e0d0;">
          <p style="margin:0 0 16px;">Bem-vindo à sala. Antes de saturar o pot, precisamos
          confirmar que este e-mail é seu — proteção simples, jogo sério.</p>
          <p style="margin:0 0 8px;font-size:12px;color:#7ab87a;text-transform:uppercase;letter-spacing:0.08em;">Código de verificação</p>
          <div style="font-size:32px;font-weight:700;letter-spacing:0.35em;color:#d4a843;font-family:Consolas,monospace;padding:12px 0;">{code}</div>
          <p style="margin:16px 0 0;font-size:13px;color:#a8d0a8;">Válido por 15 minutos. Volte ao site, digite o código e libere o lobby.</p>
        </td></tr>
        <tr><td style="padding:16px 28px;border-top:1px solid #2d5a2d;font-size:11px;color:#7ab87a;">
          Se você não criou esta conta, ignore este e-mail.<br>
          Zero Tilt · Texas Hold'em · inspirado no clássico Full Tilt
        </td></tr>
      </table>
    </td></tr>
  </table>
</body>
</html>
"#,
        username = html_escape(username),
        code = code
    );

    (subject, text, html)
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn resolve_provider() -> String {
    let explicit = env::var("EMAIL_PROVIDER")
        .unwrap_or_default()
        .to_lowercase();
    if !explicit.is_empty() {
        return explicit;
    }
    // Atalho: se há chave Resend e não forçou log, usa resend
    if env::var("RESEND_API_KEY")
        .map(|k| !k.trim().is_empty())
        .unwrap_or(false)
    {
        return "resend".to_string();
    }
    "log".to_string()
}

/// Envia e-mail de verificação conforme EMAIL_PROVIDER / RESEND_API_KEY.
pub async fn send_verification_email(
    to_email: &str,
    username: &str,
    code: &str,
) -> Result<(), String> {
    let (subject, text, html) = build_welcome_message(username, code);
    let provider = resolve_provider();

    match provider.as_str() {
        "resend" => match send_via_resend(to_email, &subject, &text, &html).await {
            Ok(id) => {
                tracing::info!(
                    target: "email",
                    to = to_email,
                    %subject,
                    resend_id = %id,
                    "E-mail de verificação enviado via Resend"
                );
                Ok(())
            }
            Err(err) => {
                tracing::error!(
                    target: "email",
                    to = to_email,
                    error = %err,
                    "Falha ao enviar e-mail de verificação via Resend"
                );
                Err(err)
            }
        },
        "log" if !is_production() => {
            log_email(to_email, &subject, code, &text);
            Ok(())
        }
        "log" => Err("EMAIL_PROVIDER=log is forbidden in production".to_string()),
        "smtp" => Err("EMAIL_PROVIDER=smtp is not implemented".to_string()),
        other => Err(format!("Unsupported EMAIL_PROVIDER: {other}")),
    }
}

fn is_production() -> bool {
    env::var("ENVIRONMENT").is_ok_and(|value| value.eq_ignore_ascii_case("production"))
}

async fn send_via_resend(
    to_email: &str,
    subject: &str,
    text: &str,
    html: &str,
) -> Result<String, String> {
    let api_key = env::var("RESEND_API_KEY")
        .map_err(|_| "RESEND_API_KEY não configurada".to_string())?
        .trim()
        .to_string();
    if api_key.is_empty() {
        return Err("RESEND_API_KEY vazia".to_string());
    }

    // Domínio verificado no Resend, ou onboarding@resend.dev (só para o e-mail da conta Resend).
    let from = env::var("EMAIL_FROM")
        .unwrap_or_else(|_| "Zero Tilt Poker <onboarding@resend.dev>".to_string());

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|e| format!("cliente HTTP: {e}"))?;

    let body = json!({
        "from": from,
        "to": [to_email],
        "subject": subject,
        "text": text,
        "html": html,
    });

    let response = client
        .post("https://api.resend.com/emails")
        .bearer_auth(&api_key)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("rede Resend: {e}"))?;

    let status = response.status();
    let response_text = response
        .text()
        .await
        .unwrap_or_else(|_| String::from("(sem corpo)"));

    if !status.is_success() {
        return Err(format!("Resend HTTP {status}: {response_text}"));
    }

    // {"id":"re_..."}
    let parsed: serde_json::Value =
        serde_json::from_str(&response_text).unwrap_or_else(|_| json!({}));
    Ok(parsed
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("ok")
        .to_string())
}

fn log_email(to: &str, subject: &str, code: &str, body: &str) {
    tracing::info!(
        target: "email",
        to,
        %subject,
        verification_code = %code,
        "=== E-mail de verificação Zero Tilt (provider=log) ===\n{body}"
    );
    eprintln!("[email:log] to={to} subject={subject} code={code} (veja logs da API)");
}

#[cfg(test)]
mod tests {
    use super::{codes_equal_hash, generate_numeric_code, hash_code};

    #[test]
    fn verification_code_is_always_six_ascii_digits() {
        for _ in 0..1_000 {
            let code = generate_numeric_code();
            assert_eq!(code.len(), 6);
            assert!(code.bytes().all(|byte| byte.is_ascii_digit()));
        }
    }

    #[test]
    fn verification_hash_is_deterministic_and_constant_time_comparable() {
        let stored = hash_code("123456");
        assert_eq!(stored.len(), 64);
        assert!(codes_equal_hash("123456", &stored));
        assert!(!codes_equal_hash("123457", &stored));
    }
}
