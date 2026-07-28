//! Cliente HTTP para a API Axum.
//!
//! Fornece funções tipadas para todos os endpoints REST da API.
//! Usa `gloo-net` para requisições HTTP no navegador (WASM).
//!
//! # URLs
//!
//! - **Dev (debug_assertions):** `https://localhost`
//! - **Prod:** `https://api.pokerplatform.com`
//!
//! # Gerenciamento de Token
//!
//! O JWT é armazenado em `localStorage` e automaticamente anexado
//! como header `Authorization: Bearer <token>` em requisições autenticadas.

use gloo_net::http::{Request, Response};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

// ─── Constantes ───

/// Base URL da API — sempre HTTPS; o Caddy encaminha internamente à API.
const API_BASE: &str = if cfg!(debug_assertions) {
    "https://localhost"
} else {
    "https://api.pokerplatform.com"
};

/// Chave no localStorage para o token JWT.
const STORAGE_TOKEN_KEY: &str = "poker_jwt";
/// Chave no localStorage para o refresh token.
const STORAGE_REFRESH_KEY: &str = "poker_refresh";

// ─── DTOs de Requisição ───

#[derive(Debug, Serialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct MfaVerifyRequest {
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct JoinTableRequest {
    pub table_id: String,
    pub buy_in: u64,
}

// ─── DTOs de Resposta ───

#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub token: String,
    #[serde(default)]
    pub refresh_token: String,
    #[serde(default)]
    pub expires_in: u64,
}

#[derive(Debug, Deserialize)]
pub struct MfaRequiredResponse {
    pub mfa_required: bool,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct TableResponse {
    pub id: String,
    pub name: String,
    pub players: u8,
    pub max_players: u8,
    pub small_blind: u64,
    pub big_blind: u64,
    pub min_buy_in: u64,
    pub max_buy_in: u64,
    pub game_type: String,
}

#[derive(Debug, Deserialize)]
pub struct JoinResponse {
    pub seat: u8,
    pub chips: u64,
}

#[derive(Debug, Deserialize)]
pub struct WebSocketTicketResponse {
    pub ticket: String,
    pub expires_in: u64,
}

#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

// ─── Gerenciamento de Token ───

/// Salva o token JWT e refresh token no localStorage.
pub fn save_tokens(token: &str, refresh: &str) {
    if let Some(Some(storage)) = web_sys::window().map(|w| w.local_storage().ok().flatten()) {
        let _ = storage.set_item(STORAGE_TOKEN_KEY, token);
        let _ = storage.set_item(STORAGE_REFRESH_KEY, refresh);
    }
}

/// Recupera o token JWT do localStorage.
pub fn get_token() -> Option<String> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|storage| storage.get_item(STORAGE_TOKEN_KEY).ok().flatten())
        .filter(|t| !t.is_empty())
}

/// Recupera o refresh token do localStorage.
pub fn get_refresh_token() -> Option<String> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|storage| storage.get_item(STORAGE_REFRESH_KEY).ok().flatten())
        .filter(|t| !t.is_empty())
}

/// Remove os tokens do localStorage (logout).
pub fn clear_tokens() {
    if let Some(Some(storage)) = web_sys::window().map(|w| w.local_storage().ok().flatten()) {
        let _ = storage.remove_item(STORAGE_TOKEN_KEY);
        let _ = storage.remove_item(STORAGE_REFRESH_KEY);
    }
}

/// Verifica se há um token JWT armazenado.
pub fn is_authenticated() -> bool {
    get_token().is_some()
}

// ─── Helpers Internos ───

/// Monta a URL completa para um path da API.
fn api_url(path: &str) -> String {
    format!("{API_BASE}{path}")
}

/// Cria headers padrão, incluindo Authorization se token existir.
fn default_headers() -> Vec<(&'static str, String)> {
    let mut headers = Vec::new();
    headers.push(("Content-Type", "application/json".to_string()));
    if let Some(token) = get_token() {
        headers.push(("Authorization", format!("Bearer {token}")));
    }
    headers
}

/// Processa a resposta HTTP, retornando o JSON tipado ou um erro.
async fn handle_response<T: DeserializeOwned>(res: Response) -> Result<T, String> {
    let status = res.status();
    if (200..300).contains(&status) {
        res.json::<T>()
            .await
            .map_err(|e| format!("Erro ao parsear resposta: {e}"))
    } else {
        let body = res.text().await.unwrap_or_default();
        // Tenta extrair mensagem de erro do JSON
        if let Ok(err) = serde_json::from_str::<ErrorResponse>(&body) {
            Err(err.error)
        } else {
            Err(format!("Erro HTTP {status}: {body}"))
        }
    }
}

/// Faz uma requisição GET autenticada.
async fn get_authenticated<T: DeserializeOwned>(path: &str) -> Result<T, String> {
    let url = api_url(path);
    let headers = default_headers();

    let mut req = Request::get(&url);
    for (key, val) in &headers {
        req = req.header(key, val);
    }

    let res = req.send().await.map_err(|e| format!("Erro de rede: {e}"))?;
    handle_response::<T>(res).await
}

/// Faz uma requisição POST autenticada com body JSON.
async fn post_authenticated<T: DeserializeOwned>(
    path: &str,
    body: &impl Serialize,
) -> Result<T, String> {
    let url = api_url(path);
    let headers = default_headers();
    let body_str =
        serde_json::to_string(body).map_err(|e| format!("Erro ao serializar body: {e}"))?;

    let mut req = Request::post(&url);
    for (key, val) in &headers {
        req = req.header(key, val);
    }

    let res = req
        .body(body_str)
        .map_err(|e| format!("Erro ao montar requisição: {e}"))?
        .send()
        .await
        .map_err(|e| format!("Erro de rede: {e}"))?;
    handle_response::<T>(res).await
}

/// Faz uma requisição POST pública (sem auth) com body JSON.
async fn post_public<T: DeserializeOwned>(path: &str, body: &impl Serialize) -> Result<T, String> {
    let url = api_url(path);
    let body_str =
        serde_json::to_string(body).map_err(|e| format!("Erro ao serializar body: {e}"))?;

    let res = Request::post(&url)
        .header("Content-Type", "application/json")
        .body(body_str)
        .map_err(|e| format!("Erro ao montar requisição: {e}"))?
        .send()
        .await
        .map_err(|e| format!("Erro de rede: {e}"))?;
    handle_response::<T>(res).await
}

// ─── Endpoints de Auth ───

/// POST /api/auth/register
///
/// Cria uma nova conta. Retorna tokens JWT em caso de sucesso.
pub async fn register(req: &RegisterRequest) -> Result<TokenResponse, String> {
    post_public::<TokenResponse>("/api/auth/register", req).await
}

/// POST /api/auth/login
///
/// Autentica o usuário. Pode retornar `MfaRequiredResponse` se MFA estiver ativo.
pub async fn login(req: &LoginRequest) -> Result<TokenResponse, String> {
    post_public::<TokenResponse>("/api/auth/login", req).await
}

/// POST /api/auth/mfa/verify
///
/// Verifica o código TOTP após login com MFA.
pub async fn mfa_verify(req: &MfaVerifyRequest) -> Result<TokenResponse, String> {
    post_public::<TokenResponse>("/api/auth/mfa/verify", req).await
}

/// POST /api/auth/refresh
///
/// Renova o token JWT usando o refresh token.
pub async fn refresh_token(req: &RefreshRequest) -> Result<TokenResponse, String> {
    post_public::<TokenResponse>("/api/auth/refresh", req).await
}

// ─── Endpoints de Lobby ───

/// GET /api/lobby/tables
///
/// Lista todas as mesas disponíveis no lobby.
pub async fn list_tables() -> Result<Vec<TableResponse>, String> {
    get_authenticated::<Vec<TableResponse>>("/api/lobby/tables").await
}

/// GET /api/lobby/tables/{id}
///
/// Obtém detalhes de uma mesa específica.
pub async fn get_table(table_id: &str) -> Result<TableResponse, String> {
    get_authenticated::<TableResponse>(&format!("/api/lobby/tables/{table_id}")).await
}

/// POST /api/lobby/join
///
/// Entra em uma mesa do lobby.
pub async fn join_table(table_id: &str, buy_in: u64) -> Result<JoinResponse, String> {
    post_authenticated::<JoinResponse>(
        "/api/lobby/join",
        &JoinTableRequest {
            table_id: table_id.to_string(),
            buy_in,
        },
    )
    .await
}

/// POST /api/lobby/tables/{id}/ws-ticket
///
/// Obtém um ticket de curta duração para o handshake WebSocket sem expor o
/// token JWT na URL. O token é passado explicitamente porque o WsClient já o
/// possui e não depende de localStorage durante reconexões.
pub async fn create_ws_ticket(
    table_id: &str,
    token: &str,
) -> Result<WebSocketTicketResponse, String> {
    let url = api_url(&format!("/api/lobby/tables/{table_id}/ws-ticket"));
    let res = Request::post(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(|error| format!("Erro de rede ao solicitar ticket WebSocket: {error}"))?;
    handle_response::<WebSocketTicketResponse>(res).await
}

// ─── Endpoints de Torneio ───

/// GET /api/tournament/{id}
///
/// Obtém informações de um torneio.
pub async fn get_tournament(tournament_id: &str) -> Result<serde_json::Value, String> {
    get_authenticated::<serde_json::Value>(&format!("/api/tournament/{tournament_id}")).await
}

/// POST /api/tournament/register
///
/// Registra o usuário autenticado em um torneio.
pub async fn register_tournament(tournament_id: &str) -> Result<serde_json::Value, String> {
    post_authenticated::<serde_json::Value>(
        "/api/tournament/register",
        &serde_json::json!({ "tournament_id": tournament_id }),
    )
    .await
}

// ─── Endpoints de Hand History ───

/// GET /api/hand-history/{hand_id}
///
/// Obtém o histórico de uma mão específica.
pub async fn get_hand_history(hand_id: &str) -> Result<serde_json::Value, String> {
    get_authenticated::<serde_json::Value>(&format!("/api/hand-history/{hand_id}")).await
}

// ─── Health Check ───

/// GET /health
///
/// Verifica se a API está online.
pub async fn health_check() -> Result<String, String> {
    let url = api_url("/health");
    let res = Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Erro de rede: {e}"))?;
    res.text()
        .await
        .map_err(|e| format!("Erro ao ler resposta: {e}"))
}

// ─── Testes ───

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_url_format() {
        let url = api_url("/api/auth/login");
        assert!(url.contains("/api/auth/login"));
        assert!(url.starts_with("http"));
    }

    #[test]
    fn test_token_storage_roundtrip() {
        // Estes testes usam `web_sys::window()` (localStorage), que só
        // existe em alvos WASM. Em testes nativos são pulados.
        #[cfg(target_arch = "wasm32")]
        {
            // Limpa estado inicial
            clear_tokens();
            assert!(!is_authenticated());

            // Salva token
            save_tokens("test-jwt-token", "test-refresh-token");
            assert!(is_authenticated());
            assert_eq!(get_token(), Some("test-jwt-token".to_string()));
            assert_eq!(get_refresh_token(), Some("test-refresh-token".to_string()));

            // Limpa
            clear_tokens();
            assert!(!is_authenticated());
        }
    }

    #[test]
    fn test_default_headers_inclui_auth_quando_token_existe() {
        #[cfg(target_arch = "wasm32")]
        {
            clear_tokens();
            save_tokens("my-token", "my-refresh");
            let headers = default_headers();
            assert!(
                headers
                    .iter()
                    .any(|(k, v)| *k == "Authorization" && v == "Bearer my-token")
            );
            clear_tokens();
        }
    }

    #[test]
    fn test_default_headers_sem_auth_quando_sem_token() {
        #[cfg(target_arch = "wasm32")]
        {
            clear_tokens();
            let headers = default_headers();
            assert!(!headers.iter().any(|(k, _)| *k == "Authorization"));
        }
    }
}
