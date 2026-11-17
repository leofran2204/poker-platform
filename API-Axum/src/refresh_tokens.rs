// Rotação do refresh token (migration 063).
//
// Um refresh token vale uma única renovação: ao ser usado é marcado como
// consumido e o cliente recebe outro, da mesma família (um login = uma
// família). Reapresentar um token já consumido, passada a tolerância de
// corrida, é tratado como cópia indevida: a família inteira é revogada e a
// pessoa precisa entrar de novo. O JWT nunca é gravado, só o SHA-256 dele.

use sha2::{Digest, Sha256};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::error::ApiError;

/// Janela em que o mesmo token pode chegar mais de uma vez sem ser tratado
/// como cópia indevida: duas abas renovando juntas, ou resposta perdida na
/// rede seguida de nova tentativa. Cada chegada recebe um token novo da mesma
/// família.
pub const REUSE_GRACE_SECS: i64 = 30;

/// Linhas vencidas removidas por login; mantém a limpeza curta e indexada.
const EXPIRED_SWEEP_LIMIT: i64 = 500;

/// Refresh token recebido do cliente, já com assinatura, validade e tipo
/// conferidos pelo `AuthManager`.
pub struct PresentedToken<'a> {
    pub token: &'a str,
    pub user_id: Uuid,
    /// `false` para tokens emitidos antes da rotação (sem `jti`): eles não têm
    /// linha na tabela e entram no controle na primeira vez em que aparecem.
    pub tracked: bool,
    pub issued_at: i64,
    pub expires_at: i64,
}

impl<'a> PresentedToken<'a> {
    /// `None` quando as claims não descrevem uma conta ou um prazo válidos.
    pub fn from_claims(token: &'a str, claims: &poker_engine::auth::JwtClaims) -> Option<Self> {
        Some(Self {
            token,
            user_id: Uuid::parse_str(&claims.sub).ok()?,
            tracked: claims.jti.is_some(),
            issued_at: i64::try_from(claims.iat).ok()?,
            expires_at: i64::try_from(claims.exp).ok()?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rotation {
    /// Token aceito; o substituto foi registrado na mesma família.
    Rotated,
    /// Token desconhecido, de outra conta ou de família já revogada.
    Rejected,
    /// Token já trocado e fora da tolerância: a família foi revogada agora.
    ReuseDetected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Decision {
    Rotate,
    RevokeFamily,
    Reject,
}

fn decide(consumed_at: Option<i64>, revoked_at: Option<i64>, now: i64) -> Decision {
    if revoked_at.is_some() {
        return Decision::Reject;
    }
    match consumed_at {
        None => Decision::Rotate,
        Some(consumed_at) if now.saturating_sub(consumed_at) <= REUSE_GRACE_SECS => {
            Decision::Rotate
        }
        Some(_) => Decision::RevokeFamily,
    }
}

pub fn hash_token(token: &str) -> String {
    format!("{:x}", Sha256::digest(token.as_bytes()))
}

fn refresh_token_expiry(now: i64) -> i64 {
    now.saturating_add(
        i64::try_from(poker_engine::auth::REFRESH_TOKEN_EXPIRATION_SECS)
            .expect("refresh token lifetime fits i64"),
    )
}

async fn insert_token(
    tx: &mut Transaction<'_, Postgres>,
    token_hash: &str,
    user_id: Uuid,
    family_id: Uuid,
    created_at: i64,
    expires_at: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO auth_refresh_tokens (token_hash, user_id, family_id, created_at, expires_at) \
         VALUES ($1, $2, $3, $4, $5) \
         ON CONFLICT (token_hash) DO NOTHING",
    )
    .bind(token_hash)
    .bind(user_id)
    .bind(family_id)
    // A tabela exige expires_at > created_at.
    .bind(created_at.min(expires_at.saturating_sub(1)))
    .bind(expires_at)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Token anterior à rotação: passa a ser rastreado, em família própria. Duas
/// apresentações simultâneas disputam a mesma chave e só uma cria a linha.
async fn track_untracked(
    tx: &mut Transaction<'_, Postgres>,
    presented: &PresentedToken<'_>,
    token_hash: &str,
) -> Result<(), sqlx::Error> {
    if presented.tracked {
        return Ok(());
    }
    insert_token(
        tx,
        token_hash,
        presented.user_id,
        Uuid::new_v4(),
        presented.issued_at,
        presented.expires_at,
    )
    .await
}

/// Serializa, entre todas as réplicas da API, o que mexe na mesma família.
/// Sem isso uma renovação concorrente poderia registrar um token novo logo
/// depois de a família ser revogada, e ele sobreviveria à revogação.
async fn lock_family(
    tx: &mut Transaction<'_, Postgres>,
    token_hash: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let family_id: Option<Uuid> =
        sqlx::query_scalar("SELECT family_id FROM auth_refresh_tokens WHERE token_hash = $1")
            .bind(token_hash)
            .fetch_optional(&mut **tx)
            .await?;
    if let Some(family_id) = family_id {
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
            .bind(family_id.to_string())
            .execute(&mut **tx)
            .await?;
    }
    Ok(family_id)
}

/// Registra o refresh token de um novo login, abrindo uma família.
pub async fn open_family(
    db: &PgPool,
    user_id: &str,
    refresh_token: &str,
    now: i64,
) -> Result<(), ApiError> {
    let user_id = Uuid::parse_str(user_id)
        .map_err(|_| ApiError::Internal("Persisted user id is not a UUID".to_string()))?;

    // Limpeza oportunista: um token vencido já é recusado pela validade do JWT,
    // então a falha aqui não pode impedir o login.
    if let Err(error) = sqlx::query(
        "DELETE FROM auth_refresh_tokens WHERE token_hash IN ( \
           SELECT token_hash FROM auth_refresh_tokens \
           WHERE expires_at < $1 ORDER BY expires_at LIMIT $2)",
    )
    .bind(now)
    .bind(EXPIRED_SWEEP_LIMIT)
    .execute(db)
    .await
    {
        tracing::warn!(?error, "failed to sweep expired refresh tokens");
    }

    let mut tx = db.begin().await?;
    insert_token(
        &mut tx,
        &hash_token(refresh_token),
        user_id,
        Uuid::new_v4(),
        now,
        refresh_token_expiry(now),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

/// Consome o token apresentado e registra `replacement` na mesma família.
///
/// O chamador só entrega `replacement` ao cliente quando o retorno é
/// [`Rotation::Rotated`]; nos demais casos ele nunca foi gravado e, por não
/// ter linha na tabela, não serve para renovar.
pub async fn rotate(
    db: &PgPool,
    presented: &PresentedToken<'_>,
    replacement: &str,
    now: i64,
) -> Result<Rotation, ApiError> {
    let presented_hash = hash_token(presented.token);
    let mut tx = db.begin().await?;
    track_untracked(&mut tx, presented, &presented_hash).await?;
    let Some(family_id) = lock_family(&mut tx, &presented_hash).await? else {
        return Ok(Rotation::Rejected);
    };

    // Lido depois da trava da família: já enxerga o que a renovação ou a
    // revogação anterior gravou.
    let row: Option<(Uuid, Option<i64>, Option<i64>)> = sqlx::query_as(
        "SELECT user_id, consumed_at, revoked_at \
         FROM auth_refresh_tokens WHERE token_hash = $1 FOR UPDATE",
    )
    .bind(&presented_hash)
    .fetch_optional(&mut *tx)
    .await?;
    let Some((owner, consumed_at, revoked_at)) = row else {
        return Ok(Rotation::Rejected);
    };
    if owner != presented.user_id {
        return Ok(Rotation::Rejected);
    }

    match decide(consumed_at, revoked_at, now) {
        Decision::Reject => Ok(Rotation::Rejected),
        Decision::RevokeFamily => {
            sqlx::query(
                "UPDATE auth_refresh_tokens SET revoked_at = $1 \
                 WHERE family_id = $2 AND revoked_at IS NULL",
            )
            .bind(now)
            .bind(family_id)
            .execute(&mut *tx)
            .await?;
            sqlx::query(
                "INSERT INTO audit_logs (user_id, action, metadata) \
                 VALUES ($1, 'REFRESH_TOKEN_REUSE', jsonb_build_object('family_id', $2::text))",
            )
            .bind(presented.user_id.to_string())
            .bind(family_id.to_string())
            .execute(&mut *tx)
            .await?;
            tx.commit().await?;
            tracing::warn!(
                user_id = %presented.user_id,
                %family_id,
                "refresh token reuse detected; session family revoked"
            );
            Ok(Rotation::ReuseDetected)
        }
        Decision::Rotate => {
            if consumed_at.is_none() {
                sqlx::query(
                    "UPDATE auth_refresh_tokens SET consumed_at = $1 WHERE token_hash = $2",
                )
                .bind(now)
                .bind(&presented_hash)
                .execute(&mut *tx)
                .await?;
            }
            insert_token(
                &mut tx,
                &hash_token(replacement),
                presented.user_id,
                family_id,
                now,
                refresh_token_expiry(now),
            )
            .await?;
            tx.commit().await?;
            Ok(Rotation::Rotated)
        }
    }
}

/// Revoga a família do token apresentado (saída explícita da conta).
pub async fn revoke_family(
    db: &PgPool,
    presented: &PresentedToken<'_>,
    now: i64,
) -> Result<(), ApiError> {
    let presented_hash = hash_token(presented.token);
    let mut tx = db.begin().await?;
    track_untracked(&mut tx, presented, &presented_hash).await?;
    let Some(family_id) = lock_family(&mut tx, &presented_hash).await? else {
        return Ok(());
    };
    sqlx::query(
        "UPDATE auth_refresh_tokens SET revoked_at = $1 \
         WHERE family_id = $2 AND user_id = $3 AND revoked_at IS NULL",
    )
    .bind(now)
    .bind(family_id)
    .bind(presented.user_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unused_token_rotates() {
        assert_eq!(decide(None, None, 1_000), Decision::Rotate);
    }

    #[test]
    fn token_presented_again_inside_the_grace_window_still_rotates() {
        assert_eq!(decide(Some(1_000), None, 1_000), Decision::Rotate);
        assert_eq!(
            decide(Some(1_000), None, 1_000 + REUSE_GRACE_SECS),
            Decision::Rotate
        );
    }

    #[test]
    fn token_presented_again_after_the_grace_window_revokes_the_family() {
        assert_eq!(
            decide(Some(1_000), None, 1_001 + REUSE_GRACE_SECS),
            Decision::RevokeFamily
        );
    }

    #[test]
    fn revoked_token_is_rejected_whether_consumed_or_not() {
        assert_eq!(decide(None, Some(900), 1_000), Decision::Reject);
        assert_eq!(decide(Some(950), Some(960), 1_000), Decision::Reject);
        assert_eq!(decide(Some(1_000), Some(1_000), 1_000), Decision::Reject);
    }

    #[test]
    fn hash_is_sha256_hex_and_never_the_token_itself() {
        let hash = hash_token("header.payload.signature");
        assert_eq!(hash.len(), 64);
        assert!(hash.bytes().all(|byte| byte.is_ascii_hexdigit()));
        assert_ne!(hash, hash_token("header.payload.signaturf"));
    }
}
