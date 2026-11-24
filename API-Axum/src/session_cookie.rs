//! Sessão por cookie (item 2 de segurança, 23/11/2026).
//!
//! O site recebe o par de tokens em cookies `HttpOnly; Secure; SameSite=Strict`
//! e não guarda no navegador nada que dê acesso à conta: um script injetado na
//! página não consegue ler o token. `Authorization: Bearer` continua valendo
//! para scripts, bots e testes, e tem precedência sobre o cookie.
//!
//! Proteção contra CSRF: pedido que altera dados autenticado pelo cookie
//! precisa do cabeçalho `X-ZT-CSRF`. Um formulário ou imagem de outro site não
//! consegue mandar cabeçalho próprio, e um `fetch` de outra origem com ele cai
//! no preflight do CORS. O `SameSite=Strict` já impede o envio do cookie em
//! navegação vinda de outro site; o cabeçalho cobre navegadores antigos.

use axum::http::{header, HeaderMap, HeaderValue, Method};
use axum::response::{IntoResponse, Response};
use axum::Json;

/// JWT de acesso (15 min), enviado em toda a API.
pub const ACCESS_COOKIE: &str = "zt_access";
/// Refresh token (7 dias), enviado só para `/api/auth`.
pub const REFRESH_COOKIE: &str = "zt_refresh";
/// Cabeçalho exigido nos pedidos que alteram dados autenticados pelo cookie.
pub const CSRF_HEADER: &str = "x-zt-csrf";
/// `X-ZT-Session: cookie` pede a sessão só por cookie: os tokens não vêm no corpo.
pub const SESSION_HEADER: &str = "x-zt-session";

const REFRESH_TTL_SECS: u64 = 7 * 24 * 3600;

fn secure_attribute() -> &'static str {
    // `COOKIE_SECURE=false` só para laboratório em HTTP fora de localhost.
    if std::env::var("COOKIE_SECURE").is_ok_and(|value| value.eq_ignore_ascii_case("false")) {
        ""
    } else {
        "; Secure"
    }
}

fn cookie(name: &str, value: &str, path: &str, max_age: u64) -> Option<HeaderValue> {
    HeaderValue::from_str(&format!(
        "{name}={value}; Path={path}; Max-Age={max_age}; HttpOnly; SameSite=Strict{}",
        secure_attribute()
    ))
    .ok()
}

/// Valor de um cookie do pedido.
pub fn read_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(key, _)| *key == name)
        .map(|(_, value)| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

/// O site pediu a sessão só por cookie.
pub fn wants_cookie_session(headers: &HeaderMap) -> bool {
    headers
        .get(SESSION_HEADER)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.eq_ignore_ascii_case("cookie"))
}

/// Pedido autenticado pelo cookie que altera dados precisa do cabeçalho anti-CSRF.
pub fn csrf_ok(method: &Method, headers: &HeaderMap) -> bool {
    matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS)
        || headers.get(CSRF_HEADER).is_some()
}

/// Resposta de entrada na conta (login, MFA, verificação, renovação): grava os
/// cookies e, para quem pediu sessão por cookie, tira os tokens do corpo.
pub fn with_session(
    headers: &HeaderMap,
    mut body: serde_json::Value,
    access_token: &str,
    access_ttl: u64,
    refresh_token: &str,
) -> Response {
    if wants_cookie_session(headers) {
        if let Some(object) = body.as_object_mut() {
            object.remove("token");
            object.remove("refresh_token");
            object.insert("session".into(), serde_json::Value::from("cookie"));
        }
    }
    let mut response = Json(body).into_response();
    let jar = response.headers_mut();
    for value in [
        cookie(ACCESS_COOKIE, access_token, "/", access_ttl.max(1)),
        cookie(REFRESH_COOKIE, refresh_token, "/api/auth", REFRESH_TTL_SECS),
    ]
    .into_iter()
    .flatten()
    {
        jar.append(header::SET_COOKIE, value);
    }
    response
}

/// Resposta que apaga os cookies da sessão (logout e renovação recusada).
pub fn clear_session(response: impl IntoResponse) -> Response {
    let mut response = response.into_response();
    let jar = response.headers_mut();
    for value in [
        cookie(ACCESS_COOKIE, "", "/", 0),
        cookie(REFRESH_COOKIE, "", "/api/auth", 0),
    ]
    .into_iter()
    .flatten()
    {
        jar.append(header::SET_COOKIE, value);
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers(pairs: &[(&str, &str)]) -> HeaderMap {
        let mut map = HeaderMap::new();
        for (key, value) in pairs {
            map.append(
                axum::http::HeaderName::from_bytes(key.as_bytes()).unwrap(),
                HeaderValue::from_str(value).unwrap(),
            );
        }
        map
    }

    #[test]
    fn reads_the_named_cookie_only() {
        let map = headers(&[
            ("cookie", "a=1; zt_access=abc.def; zt_refresh=r1"),
            ("cookie", "b=2"),
        ]);
        assert_eq!(read_cookie(&map, ACCESS_COOKIE).as_deref(), Some("abc.def"));
        assert_eq!(read_cookie(&map, REFRESH_COOKIE).as_deref(), Some("r1"));
        assert_eq!(read_cookie(&map, "b").as_deref(), Some("2"));
        assert!(read_cookie(&map, "zt").is_none());
        assert!(read_cookie(&headers(&[("cookie", "zt_access=")]), ACCESS_COOKIE).is_none());
    }

    #[test]
    fn csrf_header_is_required_only_for_writes() {
        let none = HeaderMap::new();
        assert!(csrf_ok(&Method::GET, &none));
        assert!(!csrf_ok(&Method::POST, &none));
        assert!(csrf_ok(&Method::POST, &headers(&[(CSRF_HEADER, "1")])));
        assert!(!csrf_ok(&Method::DELETE, &none));
    }

    #[test]
    fn cookie_session_hides_tokens_and_sets_strict_cookies() {
        let response = with_session(
            &headers(&[(SESSION_HEADER, "cookie")]),
            serde_json::json!({"token": "jwt", "refresh_token": "rt", "expires_in": 900}),
            "jwt",
            900,
            "rt",
        );
        let cookies: Vec<String> = response
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .map(|v| v.to_str().unwrap().to_string())
            .collect();
        assert_eq!(cookies.len(), 2);
        assert!(
            cookies[0].starts_with("zt_access=jwt; Path=/; Max-Age=900; HttpOnly; SameSite=Strict")
        );
        assert!(cookies[1].starts_with("zt_refresh=rt; Path=/api/auth; Max-Age=604800; HttpOnly"));
        assert!(cookies.iter().all(|c| c.contains("SameSite=Strict")));

        let cleared = clear_session(Json(serde_json::json!({"ok": true})));
        assert!(cleared
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .all(|v| v.to_str().unwrap().contains("Max-Age=0")));
    }
}
