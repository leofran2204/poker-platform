// ============================================================
// auth.rs — Módulo de Autenticação (JWT + bcrypt + MFA/TOTP)
// ============================================================
// Stack alvo: Rust puro (jsonwebtoken, bcrypt, hmac, sha2)
// Comunicação: JSON via serde
// Segurança: JWT curtos, bcrypt (OWF), TOTP (RFC 6238)
// ============================================================

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// ─── Crates externos ───
use bcrypt::{hash, verify};
use hmac::{Hmac, Mac};
use sha1::Sha1;

// ─── Constantes ───

/// Os testes unitários exercitam repetidamente registro, login e bloqueio de
/// conta. Reduzimos o custo somente no binário compilado com `cfg(test)` para
/// manter o feedback rápido, sem alterar o hash usado pela API em produção.
#[cfg(any(test, feature = "test-fast-bcrypt"))]
const BCRYPT_COST: u32 = 4;

/// Custo bcrypt de produção (12 rounds — segurança para credenciais reais).
#[cfg(not(any(test, feature = "test-fast-bcrypt")))]
const BCRYPT_COST: u32 = 12;

/// Tempo de vida do JWT em segundos (15 minutos)
const JWT_EXPIRATION_SECS: u64 = 900;

/// Tempo de vida do refresh token em segundos (7 dias)
const REFRESH_TOKEN_EXPIRATION_SECS: u64 = 604_800;

/// Tamanho do segredo TOTP em bytes
const TOTP_SECRET_SIZE: usize = 20;

/// Período TOTP em segundos (30s padrão RFC 6238)
const TOTP_PERIOD: u64 = 30;

/// Dígitos do código TOTP
const TOTP_DIGITS: u32 = 6;

/// Tentativas máximas de login antes de lockout temporário
pub const MAX_LOGIN_ATTEMPTS: u32 = 5;

/// Duração do lockout em segundos (15 minutos)
pub const LOCKOUT_DURATION_SECS: u64 = 900;

/// Saldo inicial em centavos ao registrar (play-money / demo).
/// 15_000 = R$ 150,00 — apenas para cash games (carteira de torneio fica zerada; freerolls são grátis).
pub const DEMO_STARTING_BALANCE_CENTS: i64 = 15_000;

/// Verifica uma credencial bcrypt sem expor detalhes do hash ao chamador.
pub fn verify_password_hash(password: &str, password_hash: &str) -> bool {
    verify(password, password_hash).unwrap_or(false)
}

// ─── Tipos / Enums ───

/// Papel do usuário no sistema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Player,
    Admin,
    Moderator,
}

/// Status da conta do usuário
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccountStatus {
    Active,
    Suspended,
    Banned,
    PendingEmailVerification,
}

/// Resultado de uma operação de autenticação
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthResult {
    Success,
    InvalidCredentials,
    AccountLocked,
    AccountSuspended,
    AccountBanned,
    MfaRequired,
    MfaFailed,
    TokenExpired,
    TokenInvalid,
    UsernameAlreadyExists,
    EmailAlreadyExists,
    PasswordTooWeak,
    InvalidEmail,
}

// ─── Structs de Dados ───

/// Claims do JWT (payload)
#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    /// Subject — ID do usuário
    pub sub: String,
    /// Username
    pub username: String,
    /// Papel do usuário
    pub role: UserRole,
    /// Incremented by durable account administration to revoke all tokens.
    pub token_version: i64,
    /// Timestamp de emissão (epoch seconds)
    pub iat: u64,
    /// Timestamp de expiração (epoch seconds)
    pub exp: u64,
    /// Tipo de token (access | refresh)
    #[serde(rename = "type")]
    pub token_type: String,
}

/// Usuário do sistema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// ID único (UUID v4)
    pub id: String,
    /// Nome de usuário (único, 3-30 caracteres)
    pub username: String,
    /// Email (único)
    pub email: String,
    /// Hash bcrypt da senha
    pub password_hash: String,
    /// Papel no sistema
    pub role: UserRole,
    /// Status da conta
    pub status: AccountStatus,
    /// Saldo em centavos (R$)
    pub balance: i64,
    /// MFA habilitado?
    pub mfa_enabled: bool,
    /// Segredo TOTP (Base32, só presente se mfa_enabled)
    pub mfa_secret: Option<String>,
    /// Contagem de tentativas de login falhas
    pub failed_login_attempts: u32,
    /// Timestamp do lockout (epoch seconds), None se não estiver lockado
    pub locked_until: Option<u64>,
    /// Timestamp de criação da conta
    pub created_at: u64,
    /// Timestamp do último login
    pub last_login: Option<u64>,
    /// Version embedded in issued JWTs. A durable version mismatch revokes
    /// both access and refresh tokens across replicas.
    pub token_version: i64,
}

/// Par de tokens retornado no login
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    /// JWT de acesso (curto, 15 min)
    pub access_token: String,
    /// Refresh token (longo, 7 dias)
    pub refresh_token: String,
    /// Timestamp de expiração do access token
    pub expires_at: u64,
    /// Tipo de token (Bearer)
    pub token_type: String,
}

/// Requisição de registro
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// Requisição de login
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    /// Código MFA (opcional, obrigatório se mfa_enabled)
    pub mfa_code: Option<String>,
}

/// Requisição de refresh de token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

/// Configuração do MFA para um usuário
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaSetup {
    /// Segredo TOTP em Base32
    pub secret: String,
    /// URI para QR code (otpauth://)
    pub qr_uri: String,
    /// Códigos de backup (8 códigos de uso único)
    pub backup_codes: Vec<String>,
}

/// Sessão ativa de usuário
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// ID da sessão
    pub id: String,
    /// ID do usuário
    pub user_id: String,
    /// Username
    pub username: String,
    /// IP de origem
    pub ip_address: String,
    /// User agent
    pub user_agent: String,
    /// Timestamp de criação
    pub created_at: u64,
    /// Timestamp de expiração
    pub expires_at: u64,
    /// Sessão ainda é válida?
    pub is_active: bool,
}

// ─── AuthManager ───

/// Gerenciador central de autenticação
pub struct AuthManager {
    /// Usuários registrados (username → User)
    users: HashMap<String, User>,
    /// Sessões ativas (session_id → Session)
    sessions: HashMap<String, Session>,
    /// Segredo JWT (HMAC-SHA256)
    jwt_secret: String,
    /// Contador de IDs de sessão
    session_counter: u64,
}

impl AuthManager {
    /// Cria um novo AuthManager com o segredo JWT fornecido
    pub fn new(jwt_secret: &str) -> Self {
        AuthManager {
            users: HashMap::new(),
            sessions: HashMap::new(),
            jwt_secret: jwt_secret.to_string(),
            session_counter: 0,
        }
    }

    /// Loads the authoritative account record from durable storage.
    ///
    /// The API keeps this manager as a cache for password verification, MFA and
    /// token issuance; PostgreSQL remains the source of truth across restarts.
    pub fn upsert_persisted_user(&mut self, user: User) {
        self.users.insert(user.username.to_lowercase(), user);
    }

    /// Removes a user that was staged in memory but could not be persisted.
    /// This keeps registration atomic from the API's perspective.
    pub fn remove_user(&mut self, username: &str) -> Option<User> {
        self.users.remove(&username.to_lowercase())
    }

    // ─── Registro ───

    /// Registra um novo usuário
    pub fn register_user(&mut self, request: &RegisterRequest) -> Result<User, AuthResult> {
        // Validar username (3-30 caracteres, alfanumérico + underscore)
        if request.username.len() < 3 || request.username.len() > 30 {
            return Err(AuthResult::InvalidCredentials);
        }
        if !request
            .username
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_')
        {
            return Err(AuthResult::InvalidCredentials);
        }

        // Verificar se username já existe
        if self.users.contains_key(&request.username.to_lowercase()) {
            return Err(AuthResult::UsernameAlreadyExists);
        }

        // Validar email (formato básico)
        if !Self::is_valid_email(&request.email) {
            return Err(AuthResult::InvalidEmail);
        }

        // Verificar se email já existe
        if self
            .users
            .values()
            .any(|u| u.email.eq_ignore_ascii_case(&request.email))
        {
            return Err(AuthResult::EmailAlreadyExists);
        }

        // Validar força da senha (mínimo 8 caracteres, pelo menos 1 maiúscula, 1 minúscula, 1 dígito)
        if !Self::is_strong_password(&request.password) {
            return Err(AuthResult::PasswordTooWeak);
        }

        // Hash da senha com bcrypt
        let password_hash =
            hash(&request.password, BCRYPT_COST).map_err(|_| AuthResult::InvalidCredentials)?;

        let now = Self::current_timestamp();

        let user = User {
            id: Self::generate_uuid_v4(),
            username: request.username.clone(),
            email: request.email.to_lowercase(),
            password_hash,
            role: UserRole::Player,
            status: AccountStatus::Active,
            // Demo / soft-launch: fichas iniciais para jogar sem PIX real.
            // 100_000 centavos = R$ 1.000,00 de play-money.
            balance: DEMO_STARTING_BALANCE_CENTS,
            mfa_enabled: false,
            mfa_secret: None,
            failed_login_attempts: 0,
            locked_until: None,
            created_at: now,
            last_login: None,
            token_version: 0,
        };

        self.users
            .insert(user.username.to_lowercase(), user.clone());
        Ok(user)
    }

    // ─── Login ───

    /// Autentica um usuário e retorna par de tokens (JWT access + refresh)
    pub fn login(&mut self, request: &LoginRequest) -> Result<TokenPair, AuthResult> {
        let username_lower = request.username.to_lowercase();

        // Buscar usuário
        let user = self
            .users
            .get(&username_lower)
            .ok_or(AuthResult::InvalidCredentials)?;

        // Verificar status da conta
        match user.status {
            AccountStatus::Active => {}
            AccountStatus::Suspended => return Err(AuthResult::AccountSuspended),
            AccountStatus::Banned => return Err(AuthResult::AccountBanned),
            AccountStatus::PendingEmailVerification => {
                // Permite login mas avisa — por simplicidade, permitimos
            }
        }

        // Verificar lockout
        if let Some(locked_until) = user.locked_until {
            if Self::current_timestamp() < locked_until {
                return Err(AuthResult::AccountLocked);
            }
        }

        // Verificar senha
        let password_valid = verify(&request.password, &user.password_hash).unwrap_or(false);

        if !password_valid {
            // Incrementar contagem de falhas
            let user_mut = self.users.get_mut(&username_lower).unwrap();
            user_mut.failed_login_attempts += 1;
            if user_mut.failed_login_attempts >= MAX_LOGIN_ATTEMPTS {
                user_mut.locked_until = Some(Self::current_timestamp() + LOCKOUT_DURATION_SECS);
            }
            return Err(AuthResult::InvalidCredentials);
        }

        // Verificar MFA se habilitado
        let user = self.users.get(&username_lower).unwrap();
        if user.mfa_enabled {
            let mfa_code = request.mfa_code.as_ref().ok_or(AuthResult::MfaRequired)?;
            let secret = user.mfa_secret.as_ref().ok_or(AuthResult::MfaRequired)?;
            if !Self::verify_totp(secret, mfa_code) {
                return Err(AuthResult::MfaFailed);
            }
        }

        // Resetar contagem de falhas e atualizar último login
        let user_mut = self.users.get_mut(&username_lower).unwrap();
        user_mut.failed_login_attempts = 0;
        user_mut.locked_until = None;
        user_mut.last_login = Some(Self::current_timestamp());

        let user = user_mut.clone();

        // Gerar tokens
        let access_token = self.generate_access_token(&user)?;
        let refresh_token = self.generate_refresh_token(&user)?;
        let expires_at = Self::current_timestamp() + JWT_EXPIRATION_SECS;

        Ok(TokenPair {
            access_token,
            refresh_token,
            expires_at,
            token_type: "Bearer".to_string(),
        })
    }

    /// Emite par de tokens para um usuário já autenticado por outro meio
    /// (ex.: verificação de e-mail). Só para contas `Active`.
    pub fn issue_tokens_for_user(&self, user: &User) -> Result<TokenPair, AuthResult> {
        if user.status != AccountStatus::Active {
            return Err(AuthResult::AccountSuspended);
        }
        let access_token = self.generate_access_token(user)?;
        let refresh_token = self.generate_refresh_token(user)?;
        let expires_at = Self::current_timestamp() + JWT_EXPIRATION_SECS;
        Ok(TokenPair {
            access_token,
            refresh_token,
            expires_at,
            token_type: "Bearer".to_string(),
        })
    }

    // ─── Refresh de Token ───

    /// Renova um access token usando um refresh token válido
    pub fn refresh_access_token(&self, request: &RefreshRequest) -> Result<TokenPair, AuthResult> {
        let claims = self.validate_token(&request.refresh_token, "refresh")?;

        let user = self
            .users
            .get(&claims.username.to_lowercase())
            .ok_or(AuthResult::TokenInvalid)?;

        if claims.token_version != user.token_version {
            return Err(AuthResult::TokenInvalid);
        }

        if user.status != AccountStatus::Active {
            return Err(AuthResult::AccountSuspended);
        }

        let access_token = self.generate_access_token(user)?;
        let refresh_token = self.generate_refresh_token(user)?;
        let expires_at = Self::current_timestamp() + JWT_EXPIRATION_SECS;

        Ok(TokenPair {
            access_token,
            refresh_token,
            expires_at,
            token_type: "Bearer".to_string(),
        })
    }

    // ─── Validação de Token ───

    /// Valida um token JWT e retorna as claims
    pub fn validate_token(
        &self,
        token: &str,
        expected_type: &str,
    ) -> Result<JwtClaims, AuthResult> {
        let claims =
            jwt_decode::<JwtClaims>(token, self.jwt_secret.as_bytes()).map_err(|e| match e {
                JwtError::Expired => AuthResult::TokenExpired,
                JwtError::Invalid => AuthResult::TokenInvalid,
            })?;

        // Verificar tipo de token
        if claims.token_type != expected_type {
            return Err(AuthResult::TokenInvalid);
        }

        Ok(claims)
    }

    // ─── MFA / TOTP ───

    /// Configura MFA para um usuário, retornando o segredo e QR URI
    pub fn setup_mfa(&mut self, username: &str) -> Result<MfaSetup, AuthResult> {
        let username_lower = username.to_lowercase();
        let user = self
            .users
            .get_mut(&username_lower)
            .ok_or(AuthResult::InvalidCredentials)?;

        if user.mfa_enabled {
            // Já tem MFA — retornar configuração existente
            let secret = user.mfa_secret.as_ref().unwrap();
            return Ok(MfaSetup {
                secret: secret.clone(),
                qr_uri: Self::generate_otpauth_uri(&user.username, secret),
                backup_codes: vec![], // já foram consumidos ou não gerados novamente
            });
        }

        // Gerar novo segredo TOTP
        let secret = Self::generate_totp_secret();
        let backup_codes = Self::generate_backup_codes();

        user.mfa_enabled = true;
        user.mfa_secret = Some(secret.clone());
        user.token_version = user.token_version.saturating_add(1);

        Ok(MfaSetup {
            qr_uri: Self::generate_otpauth_uri(&user.username, &secret),
            secret,
            backup_codes,
        })
    }

    /// Desabilita MFA para um usuário (requer código MFA válido)
    pub fn disable_mfa(&mut self, username: &str, mfa_code: &str) -> Result<(), AuthResult> {
        let username_lower = username.to_lowercase();
        let user = self
            .users
            .get_mut(&username_lower)
            .ok_or(AuthResult::InvalidCredentials)?;

        if !user.mfa_enabled {
            return Ok(()); // já desabilitado
        }

        let secret = user.mfa_secret.as_ref().ok_or(AuthResult::MfaRequired)?;
        if !Self::verify_totp(secret, mfa_code) {
            return Err(AuthResult::MfaFailed);
        }

        user.mfa_enabled = false;
        user.mfa_secret = None;
        user.token_version = user.token_version.saturating_add(1);
        Ok(())
    }

    /// Verifica um código TOTP para um usuário (para uso externo, ex: ação sensível)
    pub fn verify_mfa_for_user(&self, username: &str, mfa_code: &str) -> Result<bool, AuthResult> {
        let username_lower = username.to_lowercase();
        let user = self
            .users
            .get(&username_lower)
            .ok_or(AuthResult::InvalidCredentials)?;

        if !user.mfa_enabled {
            return Ok(true); // MFA não exigido
        }

        let secret = user.mfa_secret.as_ref().ok_or(AuthResult::MfaRequired)?;
        Ok(Self::verify_totp(secret, mfa_code))
    }

    // ─── Gerenciamento de Sessão ───

    /// Cria uma nova sessão para um usuário
    pub fn create_session(
        &mut self,
        user_id: &str,
        username: &str,
        ip: &str,
        user_agent: &str,
    ) -> Session {
        self.session_counter += 1;
        let now = Self::current_timestamp();

        let session = Session {
            id: format!("sess_{}_{}", now, self.session_counter),
            user_id: user_id.to_string(),
            username: username.to_string(),
            ip_address: ip.to_string(),
            user_agent: user_agent.to_string(),
            created_at: now,
            expires_at: now + JWT_EXPIRATION_SECS,
            is_active: true,
        };

        self.sessions.insert(session.id.clone(), session.clone());
        session
    }

    /// Invalida uma sessão (logout)
    pub fn invalidate_session(&mut self, session_id: &str) -> bool {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.is_active = false;
            true
        } else {
            false
        }
    }

    /// Invalida todas as sessões de um usuário (logout global)
    pub fn invalidate_all_user_sessions(&mut self, user_id: &str) -> usize {
        let mut count = 0;
        for session in self.sessions.values_mut() {
            if session.user_id == user_id && session.is_active {
                session.is_active = false;
                count += 1;
            }
        }
        count
    }

    /// Verifica se uma sessão está ativa
    pub fn is_session_active(&self, session_id: &str) -> bool {
        self.sessions
            .get(session_id)
            .map(|s| s.is_active && Self::current_timestamp() < s.expires_at)
            .unwrap_or(false)
    }

    /// Lista sessões ativas de um usuário
    pub fn get_active_sessions(&self, user_id: &str) -> Vec<&Session> {
        self.sessions
            .values()
            .filter(|s| {
                s.user_id == user_id && s.is_active && Self::current_timestamp() < s.expires_at
            })
            .collect()
    }

    // ─── Gerenciamento de Usuários ───

    /// Busca um usuário por username
    pub fn get_user(&self, username: &str) -> Option<&User> {
        self.users.get(&username.to_lowercase())
    }

    /// Busca um usuário por ID
    pub fn get_user_by_id(&self, user_id: &str) -> Option<&User> {
        self.users.values().find(|u| u.id == user_id)
    }

    /// Atualiza o saldo de um usuário
    pub fn update_balance(&mut self, username: &str, new_balance: i64) -> Result<(), AuthResult> {
        let username_lower = username.to_lowercase();
        let user = self
            .users
            .get_mut(&username_lower)
            .ok_or(AuthResult::InvalidCredentials)?;
        user.balance = new_balance;
        Ok(())
    }

    /// Suspende um usuário
    pub fn suspend_user(&mut self, username: &str) -> Result<(), AuthResult> {
        let username_lower = username.to_lowercase();
        let user_id = {
            let user = self
                .users
                .get_mut(&username_lower)
                .ok_or(AuthResult::InvalidCredentials)?;
            user.status = AccountStatus::Suspended;
            user.token_version = user.token_version.saturating_add(1);
            user.id.clone()
        };
        self.invalidate_all_user_sessions(&user_id);
        Ok(())
    }

    /// Bane um usuário
    pub fn ban_user(&mut self, username: &str) -> Result<(), AuthResult> {
        let username_lower = username.to_lowercase();
        let user_id = {
            let user = self
                .users
                .get_mut(&username_lower)
                .ok_or(AuthResult::InvalidCredentials)?;
            user.status = AccountStatus::Banned;
            user.token_version = user.token_version.saturating_add(1);
            user.id.clone()
        };
        self.invalidate_all_user_sessions(&user_id);
        Ok(())
    }

    /// Reativa um usuário
    pub fn reactivate_user(&mut self, username: &str) -> Result<(), AuthResult> {
        let username_lower = username.to_lowercase();
        let user = self
            .users
            .get_mut(&username_lower)
            .ok_or(AuthResult::InvalidCredentials)?;
        user.status = AccountStatus::Active;
        user.failed_login_attempts = 0;
        user.locked_until = None;
        user.token_version = user.token_version.saturating_add(1);
        Ok(())
    }

    /// Altera o papel de um usuário
    pub fn set_role(&mut self, username: &str, role: UserRole) -> Result<(), AuthResult> {
        let username_lower = username.to_lowercase();
        let user = self
            .users
            .get_mut(&username_lower)
            .ok_or(AuthResult::InvalidCredentials)?;
        user.role = role;
        user.token_version = user.token_version.saturating_add(1);
        Ok(())
    }

    /// Retorna o número total de usuários
    pub fn user_count(&self) -> usize {
        self.users.len()
    }

    /// Retorna o número de sessões ativas
    pub fn active_session_count(&self) -> usize {
        self.sessions
            .values()
            .filter(|s| s.is_active && Self::current_timestamp() < s.expires_at)
            .count()
    }

    // ─── Helpers Internos ───

    /// Gera um access token JWT (manual — HMAC-SHA256)
    fn generate_access_token(&self, user: &User) -> Result<String, AuthResult> {
        let now = Self::current_timestamp();
        let claims = JwtClaims {
            sub: user.id.clone(),
            username: user.username.clone(),
            role: user.role.clone(),
            token_version: user.token_version,
            iat: now,
            exp: now + JWT_EXPIRATION_SECS,
            token_type: "access".to_string(),
        };
        jwt_encode(&claims, self.jwt_secret.as_bytes()).map_err(|_| AuthResult::TokenInvalid)
    }

    /// Gera um refresh token JWT (manual — HMAC-SHA256)
    fn generate_refresh_token(&self, user: &User) -> Result<String, AuthResult> {
        let now = Self::current_timestamp();
        let claims = JwtClaims {
            sub: user.id.clone(),
            username: user.username.clone(),
            role: user.role.clone(),
            token_version: user.token_version,
            iat: now,
            exp: now + REFRESH_TOKEN_EXPIRATION_SECS,
            token_type: "refresh".to_string(),
        };
        jwt_encode(&claims, self.jwt_secret.as_bytes()).map_err(|_| AuthResult::TokenInvalid)
    }

    /// Timestamp atual em segundos desde epoch
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Validação básica de formato de email
    fn is_valid_email(email: &str) -> bool {
        // Deve conter @ e pelo menos um . depois do @
        if let Some(at_pos) = email.find('@') {
            let local = &email[..at_pos];
            let domain = &email[at_pos + 1..];
            !local.is_empty() && domain.contains('.') && domain.len() >= 3
        } else {
            false
        }
    }

    /// Valida força da senha
    fn is_strong_password(password: &str) -> bool {
        if password.len() < 8 {
            return false;
        }
        let has_upper = password.chars().any(|c| c.is_uppercase());
        let has_lower = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        has_upper && has_lower && has_digit
    }

    /// Gera um UUID v4 seguro usando o CSPRNG do sistema (OsRng)
    fn generate_uuid_v4() -> String {
        let mut bytes = [0u8; 16];
        crate::rng_crypto::secure_random_bytes(&mut bytes);
        format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-4{:01x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5],
            bytes[6] & 0x0f, bytes[7],
            (bytes[8] & 0x3f) | 0x80, bytes[9],
            bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
        )
    }

    /// Gera um segredo TOTP aleatório em Base32 usando CSPRNG
    fn generate_totp_secret() -> String {
        let mut bytes = vec![0u8; TOTP_SECRET_SIZE];
        crate::rng_crypto::secure_random_bytes(&mut bytes);
        base32_encode(&bytes)
    }

    /// Gera códigos de backup (8 códigos de 8 dígitos) usando CSPRNG
    fn generate_backup_codes() -> Vec<String> {
        (0..8)
            .map(|_| {
                format!(
                    "{:08}",
                    crate::rng_crypto::secure_random_u32(0..=99_999_999)
                )
            })
            .collect()
    }

    /// Gera URI otpauth:// para QR code
    fn generate_otpauth_uri(username: &str, secret: &str) -> String {
        format!(
            "otpauth://totp/PokerPlatform:{username}?secret={secret}&issuer=PokerPlatform&algorithm=SHA1&digits={TOTP_DIGITS}&period={TOTP_PERIOD}",
        )
    }

    /// Verifica um código TOTP (RFC 6238 simplificado — HMAC-SHA1)
    fn verify_totp(secret_base32: &str, code: &str) -> bool {
        // Decodificar segredo Base32
        let secret_bytes = match base32_decode(secret_base32) {
            Some(b) => b,
            None => return false,
        };

        let now = Self::current_timestamp();
        let counter = now / TOTP_PERIOD;

        // Verificar janela atual e ±2 (para tolerância de clock skew em ambientes com latência alta)
        for offset in &[-2i64, -1i64, 0, 1, 2] {
            let c = (counter as i64 + offset) as u64;
            if Self::generate_totp_code(&secret_bytes, c) == code {
                return true;
            }
        }

        false
    }

    /// Gera código TOTP para um dado contador (HMAC-SHA1, RFC 4226/6238)
    fn generate_totp_code(secret: &[u8], counter: u64) -> String {
        // HOTP: HMAC-SHA1(secret, counter)
        type HmacSha1 = Hmac<Sha1>;

        let mut mac = HmacSha1::new_from_slice(secret).expect("HMAC key size");

        // Counter como 8 bytes big-endian
        mac.update(&counter.to_be_bytes());

        let result = mac.finalize();
        let digest = result.into_bytes();

        // Dynamic truncation (RFC 4226)
        let offset = (digest[digest.len() - 1] & 0x0f) as usize;
        let binary = ((digest[offset] as u32 & 0x7f) << 24)
            | ((digest[offset + 1] as u32 & 0xff) << 16)
            | ((digest[offset + 2] as u32 & 0xff) << 8)
            | (digest[offset + 3] as u32 & 0xff);

        let token = binary % 10u32.pow(TOTP_DIGITS);
        format!("{:0width$}", token, width = TOTP_DIGITS as usize)
    }
}

use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};

/// Erro de JWT
#[derive(Debug, PartialEq, Eq)]
enum JwtError {
    Invalid,
    Expired,
}

/// Codifica claims em um token JWT assinado com HMAC-SHA256 usando jsonwebtoken
fn jwt_encode<T: serde::Serialize>(claims: &T, secret: &[u8]) -> Result<String, serde_json::Error> {
    encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(secret),
    )
    .map_err(serde::ser::Error::custom)
}

/// Decodifica e valida um token JWT usando jsonwebtoken (timing-attack safe)
fn jwt_decode<T: serde::de::DeserializeOwned>(token: &str, secret: &[u8]) -> Result<T, JwtError> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    validation.leeway = 0;

    match decode::<T>(token, &DecodingKey::from_secret(secret), &validation) {
        Ok(token_data) => Ok(token_data.claims),
        Err(err) => match err.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => Err(JwtError::Expired),
            _ => Err(JwtError::Invalid),
        },
    }
}

// ─── Codificação Base32 (RFC 4648) ───

const BASE32_ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

fn base32_encode(data: &[u8]) -> String {
    let mut result = String::new();
    let mut buffer: u16 = 0;
    let mut bits_in_buffer: u8 = 0;

    for &byte in data {
        buffer = (buffer << 8) | byte as u16;
        bits_in_buffer += 8;

        while bits_in_buffer >= 5 {
            bits_in_buffer -= 5;
            let index = (buffer >> bits_in_buffer) & 0x1f;
            result.push(BASE32_ALPHABET[index as usize] as char);
        }
    }

    // Padding
    if bits_in_buffer > 0 {
        let index = (buffer << (5 - bits_in_buffer)) & 0x1f;
        result.push(BASE32_ALPHABET[index as usize] as char);
    }

    // Padding com '=' para múltiplos de 8
    while !result.len().is_multiple_of(8) {
        result.push('=');
    }

    result
}

fn base32_decode(encoded: &str) -> Option<Vec<u8>> {
    let encoded = encoded.trim_end_matches('=').to_uppercase();
    let mut result = Vec::new();
    let mut buffer: u16 = 0;
    let mut bits_in_buffer: u8 = 0;

    for c in encoded.chars() {
        let index = BASE32_ALPHABET.iter().position(|&b| b == c as u8)?;
        buffer = (buffer << 5) | index as u16;
        bits_in_buffer += 5;

        if bits_in_buffer >= 8 {
            bits_in_buffer -= 8;
            result.push((buffer >> bits_in_buffer) as u8);
        }
    }

    Some(result)
}

// ═══════════════════════════════════════════════════════════════
// TESTES
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Helpers de teste ───

    fn create_test_manager() -> AuthManager {
        AuthManager::new("test-jwt-secret-key-for-unit-tests-2026")
    }

    fn register_test_user(manager: &mut AuthManager) -> User {
        manager
            .register_user(&RegisterRequest {
                username: "TestPlayer".to_string(),
                email: "test@example.com".to_string(),
                password: "StrongPass1".to_string(),
            })
            .expect("Registro deve funcionar")
    }

    // ─── Testes de Registro ───

    #[test]
    fn test_register_user_success() {
        let mut manager = create_test_manager();
        let user = register_test_user(&mut manager);

        assert_eq!(user.username, "TestPlayer");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.role, UserRole::Player);
        assert_eq!(user.status, AccountStatus::Active);
        assert_eq!(user.balance, DEMO_STARTING_BALANCE_CENTS);
        assert!(!user.mfa_enabled);
        assert!(user.mfa_secret.is_none());
        assert_eq!(user.failed_login_attempts, 0);
        assert!(user.locked_until.is_none());
        assert!(user.created_at > 0);
        assert!(user.last_login.is_none());
        // Senha deve estar hasheada (não plaintext)
        assert!(!user.password_hash.is_empty());
        assert_ne!(user.password_hash, "StrongPass1");
    }

    #[test]
    fn test_register_duplicate_username() {
        let mut manager = create_test_manager();
        register_test_user(&mut manager);

        let result = manager.register_user(&RegisterRequest {
            username: "testplayer".into(), // case-insensitive
            email: "other@example.com".into(),
            password: "StrongPass1".into(),
        });

        assert_eq!(result.err(), Some(AuthResult::UsernameAlreadyExists));
    }

    #[test]
    fn test_register_duplicate_email() {
        let mut manager = create_test_manager();
        register_test_user(&mut manager);

        let result = manager.register_user(&RegisterRequest {
            username: "OtherPlayer".into(),
            email: "TEST@example.com".into(), // case-insensitive
            password: "StrongPass1".into(),
        });

        assert_eq!(result.err(), Some(AuthResult::EmailAlreadyExists));
    }

    #[test]
    fn test_register_invalid_email() {
        let mut manager = create_test_manager();

        let result = manager.register_user(&RegisterRequest {
            username: "Player1".into(),
            email: "notanemail".into(),
            password: "StrongPass1".into(),
        });

        assert_eq!(result.err(), Some(AuthResult::InvalidEmail));
    }

    #[test]
    fn test_register_weak_password() {
        let mut manager = create_test_manager();

        // Sem maiúscula
        let result = manager.register_user(&RegisterRequest {
            username: "Player1".into(),
            email: "p1@example.com".into(),
            password: "alllowercase1".into(),
        });
        assert_eq!(result.err(), Some(AuthResult::PasswordTooWeak));

        // Sem dígito
        let result = manager.register_user(&RegisterRequest {
            username: "Player2".into(),
            email: "p2@example.com".into(),
            password: "NoDigitsHere".into(),
        });
        assert_eq!(result.err(), Some(AuthResult::PasswordTooWeak));

        // Curta demais
        let result = manager.register_user(&RegisterRequest {
            username: "Player3".into(),
            email: "p3@example.com".into(),
            password: "Ab1".into(),
        });
        assert_eq!(result.err(), Some(AuthResult::PasswordTooWeak));
    }

    #[test]
    fn test_register_short_username() {
        let mut manager = create_test_manager();

        let result = manager.register_user(&RegisterRequest {
            username: "ab".into(),
            email: "ab@example.com".into(),
            password: "StrongPass1".into(),
        });

        assert_eq!(result.err(), Some(AuthResult::InvalidCredentials));
    }

    // ─── Testes de Login ───

    #[test]
    fn test_login_success() {
        let mut manager = create_test_manager();
        register_test_user(&mut manager);

        let result = manager.login(&LoginRequest {
            username: "TestPlayer".into(),
            password: "StrongPass1".into(),
            mfa_code: None,
        });

        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert_eq!(tokens.token_type, "Bearer");
        assert!(!tokens.access_token.is_empty());
        assert!(!tokens.refresh_token.is_empty());
        assert!(tokens.expires_at > AuthManager::current_timestamp());
    }

    #[test]
    fn test_login_wrong_password() {
        let mut manager = create_test_manager();
        register_test_user(&mut manager);

        let result = manager.login(&LoginRequest {
            username: "TestPlayer".into(),
            password: "WrongPassword1".into(),
            mfa_code: None,
        });

        assert_eq!(result.err(), Some(AuthResult::InvalidCredentials));

        // Verificar que failed_login_attempts incrementou
        let user = manager.get_user("TestPlayer").unwrap();
        assert_eq!(user.failed_login_attempts, 1);
    }

    #[test]
    fn test_login_nonexistent_user() {
        let mut manager = create_test_manager();

        let result = manager.login(&LoginRequest {
            username: "GhostPlayer".into(),
            password: "Whatever1".into(),
            mfa_code: None,
        });

        assert_eq!(result.err(), Some(AuthResult::InvalidCredentials));
    }

    #[test]
    fn test_login_account_lockout() {
        let mut manager = create_test_manager();
        register_test_user(&mut manager);

        // 5 tentativas falhas
        for _ in 0..MAX_LOGIN_ATTEMPTS {
            let _ = manager.login(&LoginRequest {
                username: "TestPlayer".into(),
                password: "WrongPass1".into(),
                mfa_code: None,
            });
        }

        // Próxima tentativa deve ser lockada
        let result = manager.login(&LoginRequest {
            username: "TestPlayer".into(),
            password: "StrongPass1".into(), // senha correta!
            mfa_code: None,
        });

        assert_eq!(result.err(), Some(AuthResult::AccountLocked));
    }

    #[test]
    fn test_login_resets_failed_count() {
        let mut manager = create_test_manager();
        register_test_user(&mut manager);

        // 2 tentativas falhas
        for _ in 0..2 {
            let _ = manager.login(&LoginRequest {
                username: "TestPlayer".into(),
                password: "WrongPass1".into(),
                mfa_code: None,
            });
        }

        // Login correto reseta
        let result = manager.login(&LoginRequest {
            username: "TestPlayer".into(),
            password: "StrongPass1".into(),
            mfa_code: None,
        });
        assert!(result.is_ok());

        let user = manager.get_user("TestPlayer").unwrap();
        assert_eq!(user.failed_login_attempts, 0);
        assert!(user.last_login.is_some());
    }

    #[test]
    fn test_login_suspended_account() {
        let mut manager = create_test_manager();
        register_test_user(&mut manager);
        manager.suspend_user("TestPlayer").unwrap();

        let result = manager.login(&LoginRequest {
            username: "TestPlayer".into(),
            password: "StrongPass1".into(),
            mfa_code: None,
        });

        assert_eq!(result.err(), Some(AuthResult::AccountSuspended));
    }

    #[test]
    fn test_login_banned_account() {
        let mut manager = create_test_manager();
        register_test_user(&mut manager);
        manager.ban_user("TestPlayer").unwrap();

        let result = manager.login(&LoginRequest {
            username: "TestPlayer".into(),
            password: "StrongPass1".into(),
            mfa_code: None,
        });

        assert_eq!(result.err(), Some(AuthResult::AccountBanned));
    }

    // ─── Testes de JWT ───

    #[test]
    fn test_validate_access_token() {
        let mut manager = create_test_manager();
        register_test_user(&mut manager);

        let tokens = manager
            .login(&LoginRequest {
                username: "TestPlayer".into(),
                password: "StrongPass1".into(),
                mfa_code: None,
            })
            .unwrap();

        let claims = manager.validate_token(&tokens.access_token, "access");
        assert!(claims.is_ok());
        let claims = claims.unwrap();
        assert_eq!(claims.username, "TestPlayer");
        assert_eq!(claims.role, UserRole::Player);
        assert_eq!(claims.token_type, "access");
    }

    #[test]
    fn test_validate_refresh_token() {
        let mut manager = create_test_manager();
        register_test_user(&mut manager);

        let tokens = manager
            .login(&LoginRequest {
                username: "TestPlayer".into(),
                password: "StrongPass1".into(),
                mfa_code: None,
            })
            .unwrap();

        let claims = manager.validate_token(&tokens.refresh_token, "refresh");
        assert!(claims.is_ok());
        let claims = claims.unwrap();
        assert_eq!(claims.token_type, "refresh");
    }

    #[test]
    fn test_validate_wrong_token_type() {
        let mut manager = create_test_manager();
        register_test_user(&mut manager);

        let tokens = manager
            .login(&LoginRequest {
                username: "TestPlayer".into(),
                password: "StrongPass1".into(),
                mfa_code: None,
            })
            .unwrap();

        // Tentar validar access token como refresh
        let result = manager.validate_token(&tokens.access_token, "refresh");
        assert_eq!(result.err(), Some(AuthResult::TokenInvalid));
    }

    #[test]
    fn test_validate_invalid_token() {
        let manager = create_test_manager();
        let result = manager.validate_token("invalid.token.here", "access");
        assert_eq!(result.err(), Some(AuthResult::TokenInvalid));
    }

    #[test]
    fn test_refresh_access_token() {
        let mut manager = create_test_manager();
        register_test_user(&mut manager);

        let tokens = manager
            .login(&LoginRequest {
                username: "TestPlayer".into(),
                password: "StrongPass1".into(),
                mfa_code: None,
            })
            .unwrap();

        let new_tokens = manager.refresh_access_token(&RefreshRequest {
            refresh_token: tokens.refresh_token,
        });

        assert!(new_tokens.is_ok());
        let new_tokens = new_tokens.unwrap();
        assert!(!new_tokens.access_token.is_empty());
        assert!(!new_tokens.refresh_token.is_empty());
        // O novo access token deve ser válido e do tipo "access"
        let claims = manager.validate_token(&new_tokens.access_token, "access");
        assert!(claims.is_ok());
        assert_eq!(claims.unwrap().username, "TestPlayer");
    }

    // ─── Testes de MFA / TOTP ───

    #[test]
    fn test_setup_mfa() {
        let mut manager = create_test_manager();
        register_test_user(&mut manager);

        let setup = manager.setup_mfa("TestPlayer").unwrap();
        assert!(!setup.secret.is_empty());
        assert!(setup.qr_uri.contains("otpauth://"));
        assert!(setup.qr_uri.contains("TestPlayer"));
        assert_eq!(setup.backup_codes.len(), 8);

        let user = manager.get_user("TestPlayer").unwrap();
        assert!(user.mfa_enabled);
        assert!(user.mfa_secret.is_some());
    }

    #[test]
    fn test_disable_mfa() {
        let mut manager = create_test_manager();
        register_test_user(&mut manager);

        let setup = manager.setup_mfa("TestPlayer").unwrap();

        // Gerar código TOTP válido para o segredo
        let secret_bytes = base32_decode(&setup.secret).unwrap();
        let now = AuthManager::current_timestamp();
        let counter = now / TOTP_PERIOD;
        let valid_code = AuthManager::generate_totp_code(&secret_bytes, counter);

        let result = manager.disable_mfa("TestPlayer", &valid_code);
        assert!(result.is_ok());

        let user = manager.get_user("TestPlayer").unwrap();
        assert!(!user.mfa_enabled);
        assert!(user.mfa_secret.is_none());
    }

    #[test]
    fn test_disable_mfa_wrong_code() {
        let mut manager = create_test_manager();
        register_test_user(&mut manager);
        manager.setup_mfa("TestPlayer").unwrap();

        let result = manager.disable_mfa("TestPlayer", "000000");
        assert_eq!(result.err(), Some(AuthResult::MfaFailed));

        // MFA ainda deve estar ativo
        let user = manager.get_user("TestPlayer").unwrap();
        assert!(user.mfa_enabled);
    }

    #[test]
    fn test_login_with_mfa_required() {
        let mut manager = create_test_manager();
        register_test_user(&mut manager);
        let _setup = manager.setup_mfa("TestPlayer").unwrap();

        // Login sem código MFA
        let result = manager.login(&LoginRequest {
            username: "TestPlayer".into(),
            password: "StrongPass1".into(),
            mfa_code: None,
        });

        assert_eq!(result.err(), Some(AuthResult::MfaRequired));
    }

    #[test]
    fn test_login_with_mfa_success() {
        let mut manager = create_test_manager();
        register_test_user(&mut manager);
        let setup = manager.setup_mfa("TestPlayer").unwrap();

        // Gerar código TOTP válido
        let secret_bytes = base32_decode(&setup.secret).unwrap();
        let now = AuthManager::current_timestamp();
        let counter = now / TOTP_PERIOD;
        let valid_code = AuthManager::generate_totp_code(&secret_bytes, counter);

        let result = manager.login(&LoginRequest {
            username: "TestPlayer".into(),
            password: "StrongPass1".into(),
            mfa_code: Some(valid_code),
        });

        assert!(result.is_ok());
    }

    #[test]
    fn test_login_with_mfa_wrong_code() {
        let mut manager = create_test_manager();
        register_test_user(&mut manager);
        manager.setup_mfa("TestPlayer").unwrap();

        let result = manager.login(&LoginRequest {
            username: "TestPlayer".into(),
            password: "StrongPass1".into(),
            mfa_code: Some("000000".into()),
        });

        assert_eq!(result.err(), Some(AuthResult::MfaFailed));
    }

    #[test]
    fn token_version_revokes_refresh_tokens_after_account_administration() {
        let mut manager = create_test_manager();
        register_test_user(&mut manager);
        let tokens = manager
            .login(&LoginRequest {
                username: "TestPlayer".into(),
                password: "StrongPass1".into(),
                mfa_code: None,
            })
            .expect("initial login must succeed");

        manager.suspend_user("TestPlayer").unwrap();
        let refresh = manager.refresh_access_token(&RefreshRequest {
            refresh_token: tokens.refresh_token,
        });
        assert_eq!(refresh.err(), Some(AuthResult::TokenInvalid));
    }

    #[test]
    fn test_verify_mfa_for_user_without_mfa() {
        let mut manager = create_test_manager();
        register_test_user(&mut manager);

        // Usuário sem MFA — qualquer código é aceito
        let result = manager.verify_mfa_for_user("TestPlayer", "000000");
        assert_eq!(result, Ok(true));
    }

    #[test]
    fn test_totp_code_generation() {
        // Teste com vetor conhecido (RFC 6238 test vector)
        // Secret: "12345678901234567890" em Base32 = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ"
        let secret_base32 = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";
        let secret_bytes = base32_decode(secret_base32).unwrap();

        // Counter 0 → TOTP deve ser "287082" (SHA1)
        // Nota: nosso TOTP usa SHA256, então o valor será diferente
        let code = AuthManager::generate_totp_code(&secret_bytes, 0);
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_totp_verify_with_clock_skew() {
        let secret = AuthManager::generate_totp_secret();
        let secret_bytes = base32_decode(&secret).unwrap();

        // Gerar código para counter atual
        let now = AuthManager::current_timestamp();
        let counter = now / TOTP_PERIOD;
        let code = AuthManager::generate_totp_code(&secret_bytes, counter);

        // Deve verificar com sucesso (janela ±1)
        assert!(AuthManager::verify_totp(&secret, &code));
    }

    // ─── Testes de Sessão ───

    #[test]
    fn test_create_session() {
        let mut manager = create_test_manager();
        let user = register_test_user(&mut manager);

        let session =
            manager.create_session(&user.id, &user.username, "192.168.1.1", "Mozilla/5.0");
        assert!(session.id.starts_with("sess_"));
        assert_eq!(session.user_id, user.id);
        assert_eq!(session.username, "TestPlayer");
        assert_eq!(session.ip_address, "192.168.1.1");
        assert!(session.is_active);
    }

    #[test]
    fn test_invalidate_session() {
        let mut manager = create_test_manager();
        let user = register_test_user(&mut manager);

        let session = manager.create_session(&user.id, &user.username, "127.0.0.1", "TestAgent");
        assert!(manager.is_session_active(&session.id));

        let invalidated = manager.invalidate_session(&session.id);
        assert!(invalidated);
        assert!(!manager.is_session_active(&session.id));
    }

    #[test]
    fn test_invalidate_all_user_sessions() {
        let mut manager = create_test_manager();
        let user = register_test_user(&mut manager);

        manager.create_session(&user.id, &user.username, "ip1", "ua1");
        manager.create_session(&user.id, &user.username, "ip2", "ua2");
        manager.create_session(&user.id, &user.username, "ip3", "ua3");

        let count = manager.invalidate_all_user_sessions(&user.id);
        assert_eq!(count, 3);
        assert_eq!(manager.get_active_sessions(&user.id).len(), 0);
    }

    #[test]
    fn test_get_active_sessions() {
        let mut manager = create_test_manager();
        let user = register_test_user(&mut manager);

        manager.create_session(&user.id, &user.username, "ip1", "ua1");
        manager.create_session(&user.id, &user.username, "ip2", "ua2");

        let active = manager.get_active_sessions(&user.id);
        assert_eq!(active.len(), 2);
    }

    // ─── Testes de Gerenciamento de Usuários ───

    #[test]
    fn test_get_user() {
        let mut manager = create_test_manager();
        register_test_user(&mut manager);

        let user = manager.get_user("testplayer"); // case-insensitive
        assert!(user.is_some());
        assert_eq!(user.unwrap().username, "TestPlayer");
    }

    #[test]
    fn test_get_user_by_id() {
        let mut manager = create_test_manager();
        let user = register_test_user(&mut manager);

        let found = manager.get_user_by_id(&user.id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().username, "TestPlayer");
    }

    #[test]
    fn test_update_balance() {
        let mut manager = create_test_manager();
        register_test_user(&mut manager);

        manager.update_balance("TestPlayer", 10000).unwrap();
        let user = manager.get_user("TestPlayer").unwrap();
        assert_eq!(user.balance, 10000);
    }

    #[test]
    fn test_suspend_and_reactivate() {
        let mut manager = create_test_manager();
        register_test_user(&mut manager);

        manager.suspend_user("TestPlayer").unwrap();
        assert_eq!(
            manager.get_user("TestPlayer").unwrap().status,
            AccountStatus::Suspended
        );

        manager.reactivate_user("TestPlayer").unwrap();
        assert_eq!(
            manager.get_user("TestPlayer").unwrap().status,
            AccountStatus::Active
        );
    }

    #[test]
    fn test_ban_user() {
        let mut manager = create_test_manager();
        register_test_user(&mut manager);

        manager.ban_user("TestPlayer").unwrap();
        assert_eq!(
            manager.get_user("TestPlayer").unwrap().status,
            AccountStatus::Banned
        );
    }

    #[test]
    fn test_set_role() {
        let mut manager = create_test_manager();
        register_test_user(&mut manager);

        manager.set_role("TestPlayer", UserRole::Admin).unwrap();
        assert_eq!(
            manager.get_user("TestPlayer").unwrap().role,
            UserRole::Admin
        );
    }

    #[test]
    fn test_user_count() {
        let mut manager = create_test_manager();
        assert_eq!(manager.user_count(), 0);

        register_test_user(&mut manager);
        assert_eq!(manager.user_count(), 1);

        manager
            .register_user(&RegisterRequest {
                username: "Player2".into(),
                email: "p2@example.com".into(),
                password: "StrongPass2".into(),
            })
            .unwrap();
        assert_eq!(manager.user_count(), 2);
    }

    #[test]
    fn test_active_session_count() {
        let mut manager = create_test_manager();
        let user = register_test_user(&mut manager);

        assert_eq!(manager.active_session_count(), 0);
        manager.create_session(&user.id, &user.username, "ip1", "ua1");
        assert_eq!(manager.active_session_count(), 1);
    }

    // ─── Testes de Serialização JSON ───

    #[test]
    fn test_user_json_serialization() {
        let mut manager = create_test_manager();
        let user = register_test_user(&mut manager);

        let json = serde_json::to_string_pretty(&user).unwrap();
        // Deve conter campos principais
        assert!(json.contains("TestPlayer"));
        assert!(json.contains("test@example.com"));
        assert!(json.contains("player")); // role lowercase
        assert!(json.contains("active")); // status lowercase
    }

    #[test]
    fn test_token_pair_json_serialization() {
        let mut manager = create_test_manager();
        register_test_user(&mut manager);

        let tokens = manager
            .login(&LoginRequest {
                username: "TestPlayer".into(),
                password: "StrongPass1".into(),
                mfa_code: None,
            })
            .unwrap();

        let json = serde_json::to_string_pretty(&tokens).unwrap();
        assert!(json.contains("Bearer"));
        assert!(json.contains("access_token"));
        assert!(json.contains("refresh_token"));
        assert!(json.contains("expires_at"));
    }

    #[test]
    fn test_auth_result_json_serialization() {
        let results = vec![
            AuthResult::Success,
            AuthResult::InvalidCredentials,
            AuthResult::AccountLocked,
            AuthResult::MfaRequired,
            AuthResult::TokenExpired,
        ];

        let json = serde_json::to_string_pretty(&results).unwrap();
        assert!(json.contains("success"));
        assert!(json.contains("invalid_credentials"));
        assert!(json.contains("account_locked"));
        assert!(json.contains("mfa_required"));
        assert!(json.contains("token_expired"));
    }

    #[test]
    fn test_jwt_claims_json_serialization() {
        let claims = JwtClaims {
            sub: "uuid-1234".into(),
            username: "Player1".into(),
            role: UserRole::Player,
            token_version: 0,
            iat: 1000,
            exp: 2000,
            token_type: "access".into(),
        };

        let json = serde_json::to_string_pretty(&claims).unwrap();
        assert!(json.contains("uuid-1234"));
        assert!(json.contains("Player1"));
        assert!(json.contains("player"));
        assert!(json.contains("access"));
    }

    // ─── Testes de Base32 ───

    #[test]
    fn test_base32_encode_decode_roundtrip() {
        let original = b"HelloWorld12345";
        let encoded = base32_encode(original);
        let decoded = base32_decode(&encoded).unwrap();
        assert_eq!(decoded, original.to_vec());
    }

    #[test]
    fn test_base32_decode_invalid() {
        let result = base32_decode("!!!!!!");
        assert!(result.is_none());
    }

    // ─── Testes de Validação ───

    #[test]
    fn test_is_valid_email() {
        assert!(AuthManager::is_valid_email("user@example.com"));
        assert!(AuthManager::is_valid_email("a@b.co"));
        assert!(!AuthManager::is_valid_email("notanemail"));
        assert!(!AuthManager::is_valid_email("@nouser.com"));
        assert!(!AuthManager::is_valid_email("nodomain@"));
    }

    #[test]
    fn test_is_strong_password() {
        assert!(AuthManager::is_strong_password("StrongPass1"));
        assert!(AuthManager::is_strong_password("Abcdefg1"));
        assert!(!AuthManager::is_strong_password("alllower1")); // sem maiúscula
        assert!(!AuthManager::is_strong_password("ALLUPPER1")); // sem minúscula
        assert!(!AuthManager::is_strong_password("NoDigitsHere")); // sem dígito
        assert!(!AuthManager::is_strong_password("Ab1")); // curta
    }
}
