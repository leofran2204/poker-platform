/**
 * Sessão do site (item 2 de segurança, 23/11/2026): os tokens ficam em cookies
 * `HttpOnly` gravados pela API e o JavaScript não os lê. Aqui fica só um
 * marcador sem segredo ("há sessão") e o nome exibido.
 *
 * `poker_jwt` e `poker_refresh` são da sessão antiga, guardada no
 * `localStorage`: servem só para migrar quem já estava logado (a primeira
 * renovação troca pelo cookie e apaga os dois).
 */
const LEGACY_TOKEN_KEY = "poker_jwt";
const LEGACY_REFRESH_KEY = "poker_refresh";
const USER_KEY = "poker_username";
const SESSION_KEY = "poker_session";

/** Marca a sessão aberta (cookies gravados pela API) e apaga os tokens antigos. */
export function markSession(): void {
  localStorage.setItem(SESSION_KEY, "1");
  localStorage.removeItem(LEGACY_TOKEN_KEY);
  localStorage.removeItem(LEGACY_REFRESH_KEY);
}

/** Token de acesso da sessão antiga, enquanto ela não migra para o cookie. */
export function getLegacyToken(): string | null {
  const t = localStorage.getItem(LEGACY_TOKEN_KEY);
  return t && t.length > 0 ? t : null;
}

/** Refresh token da sessão antiga, usado uma vez para migrar para o cookie. */
export function getLegacyRefreshToken(): string | null {
  const t = localStorage.getItem(LEGACY_REFRESH_KEY);
  return t && t.length > 0 ? t : null;
}

export function clearTokens(): void {
  localStorage.removeItem(SESSION_KEY);
  localStorage.removeItem(LEGACY_TOKEN_KEY);
  localStorage.removeItem(LEGACY_REFRESH_KEY);
  localStorage.removeItem(USER_KEY);
}

export function isAuthenticated(): boolean {
  return localStorage.getItem(SESSION_KEY) === "1" || getLegacyToken() !== null;
}

export function saveUsername(username: string): void {
  localStorage.setItem(USER_KEY, username);
}

export function getUsername(): string | null {
  return localStorage.getItem(USER_KEY);
}
