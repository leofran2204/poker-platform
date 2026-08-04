const TOKEN_KEY = "poker_jwt";
const REFRESH_KEY = "poker_refresh";
const USER_KEY = "poker_username";

export function saveTokens(token: string, refresh: string): void {
  localStorage.setItem(TOKEN_KEY, token);
  if (refresh) localStorage.setItem(REFRESH_KEY, refresh);
}

export function getToken(): string | null {
  const t = localStorage.getItem(TOKEN_KEY);
  return t && t.length > 0 ? t : null;
}

export function getRefreshToken(): string | null {
  const t = localStorage.getItem(REFRESH_KEY);
  return t && t.length > 0 ? t : null;
}

export function clearTokens(): void {
  localStorage.removeItem(TOKEN_KEY);
  localStorage.removeItem(REFRESH_KEY);
  localStorage.removeItem(USER_KEY);
}

export function isAuthenticated(): boolean {
  return getToken() !== null;
}

export function saveUsername(username: string): void {
  localStorage.setItem(USER_KEY, username);
}

export function getUsername(): string | null {
  return localStorage.getItem(USER_KEY);
}
