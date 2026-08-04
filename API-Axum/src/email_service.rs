//! Envio de e-mails de verificação / boas-vindas.
//!
//! Providers:
//! - `log` (padrão): grava no tracing (dev/demo sem SMTP)
//! - `smtp`: reserva para futuro (variáveis SMTP_*); por ora loga aviso e cai no log

use sha2::{Digest, Sha256};
use std::env;

/// Validade do código em segundos (15 minutos).
pub const CODE_TTL_SECS: u64 = 15 * 60;

/// Gera código numérico de 6 dígitos com CSPRNG.
pub fn generate_numeric_code() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    // Prefer OsRng via getrandom if available; fallback mix with time+uuid entropy
    let mut buf = [0u8; 4];
    if getrandom_fill(&mut buf).is_err() {
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(1);
        buf = (t as u32).to_le_bytes();
    }
    let n = u32::from_le_bytes(buf) % 1_000_000;
    format!("{n:06}")
}

fn getrandom_fill(buf: &mut [u8]) -> Result<(), ()> {
    // Use uuid's random for entropy without new dependency if getrandom not direct
    let u = uuid::Uuid::new_v4();
    let bytes = u.as_bytes();
    for (i, b) in buf.iter_mut().enumerate() {
        *b = bytes[i % bytes.len()];
    }
    // Mix a second uuid for better entropy on small buffers
    let u2 = uuid::Uuid::new_v4();
    for (i, b) in buf.iter_mut().enumerate() {
        *b ^= u2.as_bytes()[i % 16];
    }
    Ok(())
}

pub fn hash_code(code: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(code.as_bytes());
    hasher.update(b"|zero-tilt-email-v1");
    format!("{:x}", hasher.finalize())
}

pub fn codes_equal_hash(code: &str, stored_hash: &str) -> bool {
    use subtle::ConstantTimeEq;
    let computed = hash_code(code);
    computed
        .as_bytes()
        .ct_eq(stored_hash.as_bytes())
        .into()
}

/// Corpo HTML/texto criativo de boas-vindas + código.
pub fn build_welcome_message(username: &str, code: &str) -> (String, String) {
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

    (subject, text + "\n\n--- HTML omitted in log ---\n" + &html)
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Envia e-mail de verificação conforme EMAIL_PROVIDER.
pub fn send_verification_email(to_email: &str, username: &str, code: &str) {
    let (subject, body) = build_welcome_message(username, code);
    let provider = env::var("EMAIL_PROVIDER")
        .unwrap_or_else(|_| "log".to_string())
        .to_lowercase();

    match provider.as_str() {
        "smtp" => {
            // SMTP real fica para o trilho de ops; por enquanto não bloqueamos o registro.
            tracing::warn!(
                target: "email",
                %to_email,
                %subject,
                "EMAIL_PROVIDER=smtp ainda não implementado — caindo para log"
            );
            log_email(to_email, &subject, code, &body);
        }
        _ => {
            log_email(to_email, &subject, code, &body);
        }
    }
}

fn log_email(to: &str, subject: &str, code: &str, body: &str) {
    tracing::info!(
        target: "email",
        to,
        %subject,
        verification_code = %code,
        "=== E-mail de verificação Zero Tilt (provider=log) ===\n{body}"
    );
    // Também em stderr legível em docker logs
    eprintln!(
        "[email:log] to={to} subject={subject} code={code} (veja logs da API)"
    );
}
