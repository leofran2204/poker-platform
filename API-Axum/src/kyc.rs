use axum::extract::{Path, Query, State};
use axum::Json;
use chrono::{DateTime, NaiveDate, Utc};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use crate::error::ApiError;
use crate::middleware::auth::RequireAuth;
use crate::state::AppState;

type HmacSha256 = Hmac<Sha256>;
type KycStatusRow = (
    String,
    Option<String>,
    Option<String>,
    Option<NaiveDate>,
    Option<DateTime<Utc>>,
    Option<DateTime<Utc>>,
);

#[derive(Debug, Deserialize)]
pub struct SubmitKycBody {
    pub legal_name: String,
    pub tax_id: String,
    pub date_of_birth: String,
}

#[derive(Debug, Serialize)]
pub struct KycStatusResponse {
    pub status: String,
    pub legal_name: Option<String>,
    pub tax_id_last4: Option<String>,
    pub date_of_birth: Option<String>,
    pub submitted_at: Option<String>,
    pub reviewed_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct KycReviewBody {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct AdminKycQuery {
    pub status: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AdminKycItem {
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub status: String,
    pub legal_name: Option<String>,
    pub tax_id_last4: Option<String>,
    pub date_of_birth: Option<String>,
    pub submitted_at: Option<String>,
}

fn require_admin(auth: &crate::middleware::auth::AuthUser) -> Result<(), ApiError> {
    if auth.role != "admin" {
        return Err(ApiError::Forbidden("Admin role required".into()));
    }
    Ok(())
}

fn normalize_tax_id(value: &str) -> Result<String, ApiError> {
    let digits: String = value.chars().filter(char::is_ascii_digit).collect();
    if digits.len() != 11 || digits.bytes().all(|digit| digit == digits.as_bytes()[0]) {
        return Err(ApiError::BadRequest("CPF inválido".into()));
    }
    let nums: Vec<u32> = digits
        .bytes()
        .map(|digit| u32::from(digit - b'0'))
        .collect();
    let first_sum: u32 = nums[..9]
        .iter()
        .enumerate()
        .map(|(index, digit)| digit * (10 - index as u32))
        .sum();
    let first = if first_sum % 11 < 2 {
        0
    } else {
        11 - first_sum % 11
    };
    let second_sum: u32 = nums[..10]
        .iter()
        .enumerate()
        .map(|(index, digit)| digit * (11 - index as u32))
        .sum();
    let second = if second_sum % 11 < 2 {
        0
    } else {
        11 - second_sum % 11
    };
    if nums[9] != first || nums[10] != second {
        return Err(ApiError::BadRequest("CPF inválido".into()));
    }
    Ok(digits)
}

fn tax_id_hash(tax_id: &str) -> String {
    let pepper = std::env::var("KYC_DATA_PEPPER")
        .unwrap_or_else(|_| "development-kyc-data-pepper".to_string());
    let mut mac = HmacSha256::new_from_slice(pepper.as_bytes()).expect("HMAC accepts any key");
    mac.update(b"zero-tilt-kyc-tax-id-v1\0");
    mac.update(tax_id.as_bytes());
    format!("{:x}", mac.finalize().into_bytes())
}

pub async fn get_my_kyc(
    RequireAuth(auth): RequireAuth,
    State(state): State<AppState>,
) -> Result<Json<KycStatusResponse>, ApiError> {
    let row: KycStatusRow =
        sqlx::query_as("SELECT kyc_status,kyc_legal_name,kyc_tax_id_last4,date_of_birth,kyc_submitted_at,kyc_reviewed_at FROM users WHERE id=$1::uuid")
            .bind(&auth.user_id)
            .fetch_one(&state.db)
            .await?;
    Ok(Json(KycStatusResponse {
        status: row.0,
        legal_name: row.1,
        tax_id_last4: row.2,
        date_of_birth: row.3.map(|v| v.to_string()),
        submitted_at: row.4.map(|v| v.to_rfc3339()),
        reviewed_at: row.5.map(|v| v.to_rfc3339()),
    }))
}

pub async fn submit_kyc(
    RequireAuth(auth): RequireAuth,
    State(state): State<AppState>,
    Json(body): Json<SubmitKycBody>,
) -> Result<Json<KycStatusResponse>, ApiError> {
    let legal_name = body.legal_name.trim();
    if legal_name.chars().count() < 5 || legal_name.chars().count() > 160 {
        return Err(ApiError::BadRequest(
            "Informe seu nome civil completo".into(),
        ));
    }
    let date = NaiveDate::parse_from_str(body.date_of_birth.trim(), "%Y-%m-%d")
        .map_err(|_| ApiError::BadRequest("Data de nascimento inválida".into()))?;
    crate::responsible_gaming::validate_adult(date)?;
    let tax_id = normalize_tax_id(&body.tax_id)?;
    let hash = tax_id_hash(&tax_id);
    let duplicate: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM users WHERE kyc_tax_id_hash=$1 AND id<>$2::uuid)",
    )
    .bind(&hash)
    .bind(&auth.user_id)
    .fetch_one(&state.db)
    .await?;
    if duplicate {
        return Err(ApiError::Conflict(
            "Este CPF já está associado a outra conta".into(),
        ));
    }
    sqlx::query(
        "UPDATE users SET kyc_status='pending',kyc_legal_name=$2,kyc_tax_id_hash=$3,kyc_tax_id_last4=$4, \
         date_of_birth=$5,over_18_declared_at=COALESCE(over_18_declared_at,NOW()),kyc_submitted_at=NOW(), \
         kyc_reviewed_at=NULL,kyc_reviewed_by=NULL WHERE id=$1::uuid",
    )
    .bind(&auth.user_id)
    .bind(legal_name)
    .bind(hash)
    .bind(&tax_id[7..])
    .bind(date)
    .execute(&state.db)
    .await?;
    sqlx::query(
        "INSERT INTO audit_logs(user_id,action,metadata) VALUES($1,'KYC_SUBMITTED','{}'::jsonb)",
    )
    .bind(&auth.user_id)
    .execute(&state.db)
    .await?;
    get_my_kyc(RequireAuth(auth), State(state)).await
}

pub async fn list_kyc(
    RequireAuth(auth): RequireAuth,
    State(state): State<AppState>,
    Query(query): Query<AdminKycQuery>,
) -> Result<Json<Vec<AdminKycItem>>, ApiError> {
    require_admin(&auth)?;
    let status = query.status.unwrap_or_else(|| "pending".into());
    if !matches!(
        status.as_str(),
        "not_submitted" | "pending" | "verified" | "rejected"
    ) {
        return Err(ApiError::BadRequest("Invalid KYC status".into()));
    }
    type Row = (
        String,
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<NaiveDate>,
        Option<chrono::DateTime<chrono::Utc>>,
    );
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT id::text,username,email,kyc_status,kyc_legal_name,kyc_tax_id_last4,date_of_birth,kyc_submitted_at \
         FROM users WHERE kyc_status=$1 ORDER BY kyc_submitted_at NULLS LAST LIMIT 200",
    )
    .bind(status)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(
        rows.into_iter()
            .map(|row| AdminKycItem {
                user_id: row.0,
                username: row.1,
                email: row.2,
                status: row.3,
                legal_name: row.4,
                tax_id_last4: row.5,
                date_of_birth: row.6.map(|v| v.to_string()),
                submitted_at: row.7.map(|v| v.to_rfc3339()),
            })
            .collect(),
    ))
}

pub async fn review_kyc(
    RequireAuth(auth): RequireAuth,
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Json(body): Json<KycReviewBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_admin(&auth)?;
    if !matches!(body.status.as_str(), "verified" | "rejected") {
        return Err(ApiError::BadRequest(
            "KYC review must be verified or rejected".into(),
        ));
    }
    let result = sqlx::query(
        "UPDATE users SET kyc_status=$2,kyc_reviewed_at=NOW(),kyc_reviewed_by=$3::uuid WHERE id=$1::uuid AND kyc_status='pending'",
    )
    .bind(&user_id)
    .bind(&body.status)
    .bind(&auth.user_id)
    .execute(&state.db)
    .await?;
    if result.rows_affected() != 1 {
        return Err(ApiError::Conflict("KYC is not pending".into()));
    }
    sqlx::query("INSERT INTO audit_logs(user_id,action,metadata) VALUES($1,'KYC_REVIEWED',$2)")
        .bind(&auth.user_id)
        .bind(serde_json::json!({"subject_user_id": user_id, "status": body.status}))
        .execute(&state.db)
        .await?;
    Ok(Json(serde_json::json!({"ok": true, "status": body.status})))
}

#[cfg(test)]
mod tests {
    use super::normalize_tax_id;

    #[test]
    fn validates_cpf_check_digits() {
        assert!(normalize_tax_id("529.982.247-25").is_ok());
        assert!(normalize_tax_id("111.111.111-11").is_err());
        assert!(normalize_tax_id("529.982.247-24").is_err());
    }
}
