import { clearTokens, getRefreshToken, getToken, saveTokens } from "@/lib/auth";
import {
  emitConnectionStatus,
  emitSessionExpired,
  emitSessionRestored,
} from "@/lib/sessionEvents";
import type {
  AdminPresenceResponse,
  AdminStatsResponse,
  AdminTableListItem,
  AdminTournamentItem,
  AdminTournamentPlayer,
  AdminUserResponse,
  AntifraudAlertSummary,
  AuditLogItem,
  DepositInfoResponse,
  DepositRequestResponse,
  PixDepositResponse,
  PixDepositStatusResponse,
  ClubAgentResponse,
  ClubFinancialsResponse,
  ClubResponse,
  JoinResponse,
  MeResponse,
  TableResponse,
  TokenResponse,
  TournamentInfoResponse,
  WebSocketTicketResponse,
} from "./types";

export class ApiError extends Error {
  status: number;
  constructor(message: string, status: number) {
    super(message);
    this.status = status;
  }
}

async function parseError(res: Response): Promise<string> {
  const body = await res.text();
  try {
    const json = JSON.parse(body) as { error?: string; message?: string };
    return json.error ?? json.message ?? (body || `HTTP ${res.status}`);
  } catch {
    return body || `HTTP ${res.status}`;
  }
}

type RefreshResult = "refreshed" | "invalid";

let refreshInFlight: Promise<RefreshResult> | null = null;
let sessionExpiredNotified = false;

function connectionError(): ApiError {
  return new ApiError(
    "Sem conexão com a plataforma. Tentaremos reconectar automaticamente.",
    0,
  );
}

async function refreshAccessToken(): Promise<RefreshResult> {
  const refreshToken = getRefreshToken();
  if (!refreshToken) return "invalid";

  let res: Response;
  try {
    res = await fetch("/api/auth/refresh", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ refresh_token: refreshToken }),
    });
  } catch {
    emitConnectionStatus("offline");
    throw connectionError();
  }

  emitConnectionStatus("online");
  if (res.status === 400 || res.status === 401 || res.status === 403) {
    return "invalid";
  }
  if (!res.ok) {
    throw new ApiError(await parseError(res), res.status);
  }

  const tokens = (await res.json()) as TokenResponse;
  if (!tokens.token) return "invalid";

  saveTokens(tokens.token, tokens.refresh_token ?? refreshToken);
  sessionExpiredNotified = false;
  emitSessionRestored();
  return "refreshed";
}

function tryRefreshAccessToken(): Promise<RefreshResult> {
  if (!refreshInFlight) {
    refreshInFlight = refreshAccessToken().finally(() => {
      refreshInFlight = null;
    });
  }
  return refreshInFlight;
}

function expireSession(): void {
  clearTokens();
  if (sessionExpiredNotified) return;
  sessionExpiredNotified = true;
  emitSessionExpired();
}

async function request<T>(
  path: string,
  init: RequestInit = {},
  auth = true,
  allowRefresh = true,
): Promise<T> {
  const headers = new Headers(init.headers);
  if (!headers.has("Content-Type") && init.body) {
    headers.set("Content-Type", "application/json");
  }
  if (auth) {
    const token = getToken();
    if (token) headers.set("Authorization", `Bearer ${token}`);
  }

  let res: Response;
  try {
    res = await fetch(path, { ...init, headers });
  } catch {
    emitConnectionStatus("offline");
    throw connectionError();
  }

  emitConnectionStatus("online");
  if (res.status === 401 && auth) {
    if (allowRefresh) {
      const refreshed = await tryRefreshAccessToken();
      if (refreshed === "refreshed") {
        return request<T>(path, init, auth, false);
      }
    }
    expireSession();
  }
  if (!res.ok) {
    throw new ApiError(await parseError(res), res.status);
  }
  if (res.status === 204) return undefined as T;
  return (await res.json()) as T;
}

export interface RegisterResult {
  email_verification_required?: boolean;
  email?: string;
  username?: string;
  message?: string;
  token?: string;
  refresh_token?: string;
  expires_in?: number;
}

export interface LoginResult extends RegisterResult {
  mfa_required?: boolean;
  mfa_challenge?: string;
}

export async function register(
  username: string,
  email: string,
  password: string,
  passwordConfirm: string,
): Promise<RegisterResult> {
  return request<RegisterResult>(
    "/api/auth/register",
    {
      method: "POST",
      body: JSON.stringify({
        username,
        email,
        password,
        password_confirm: passwordConfirm,
      }),
    },
    false,
  );
}

export async function login(email: string, password: string): Promise<LoginResult> {
  return request<LoginResult>(
    "/api/auth/login",
    {
      method: "POST",
      body: JSON.stringify({ email, password }),
    },
    false,
  );
}

export async function verifyMfa(
  challenge: string,
  code: string,
): Promise<TokenResponse & { mfa_verified: boolean; username?: string }> {
  return request(
    "/api/auth/mfa/verify",
    {
      method: "POST",
      body: JSON.stringify({ challenge, code }),
    },
    false,
  );
}

export async function verifyEmail(email: string, code: string): Promise<RegisterResult & { already_verified?: boolean }> {
  return request(
    "/api/auth/verify-email",
    {
      method: "POST",
      body: JSON.stringify({ email, code }),
    },
    false,
  );
}

export async function resendVerification(email: string): Promise<{ ok: boolean; message: string }> {
  return request(
    "/api/auth/resend-verification",
    {
      method: "POST",
      body: JSON.stringify({ email }),
    },
    false,
  );
}

export async function listTables(mode: "play" | "real" = "play"): Promise<TableResponse[]> {
  return request<TableResponse[]>(`/api/lobby/tables?mode=${mode}`);
}

export async function listTournaments(
  mode: "play" | "real" = "play",
): Promise<TournamentInfoResponse[]> {
  return request<TournamentInfoResponse[]>(`/api/lobby/tournaments?mode=${mode}`);
}

export async function getTournament(id: string): Promise<TournamentInfoResponse> {
  return request<TournamentInfoResponse>(`/api/tournament/${id}`);
}

export async function registerTournament(
  tournamentId: string,
  walletMode: "play" | "real" = "play",
): Promise<{
  tournament_id: string;
  player_id: string;
  stack: number;
  registered: boolean;
  gameplay_ready: boolean;
}> {
  return request("/api/tournament/register", {
    method: "POST",
    body: JSON.stringify({ tournament_id: tournamentId, wallet_mode: walletMode }),
  });
}

export async function setWalletMode(mode: "play" | "real"): Promise<{
  balance_pm_cash: number;
  balance_pm_mtt: number;
  balance_real: number;
  preferred_wallet_mode: string;
  pm_cash_rebuy_available: boolean;
  pm_mtt_rebuy_available: boolean;
}> {
  return request("/api/wallet/mode", {
    method: "POST",
    body: JSON.stringify({ mode }),
  });
}

export async function pmRebuy(kind: "cash" | "mtt"): Promise<{
  balance_pm_cash: number;
  balance_pm_mtt: number;
  balance_real: number;
  pm_cash_rebuy_available: boolean;
  pm_mtt_rebuy_available: boolean;
}> {
  return request("/api/wallet/pm-rebuy", {
    method: "POST",
    body: JSON.stringify({ kind }),
  });
}

export async function getTable(id: string): Promise<TableResponse> {
  return request<TableResponse>(`/api/lobby/tables/${id}`);
}

export async function joinTable(
  tableId: string,
  buyIn: number,
  walletMode: "play" | "real" = "play",
): Promise<JoinResponse> {
  return request<JoinResponse>("/api/lobby/join", {
    method: "POST",
    body: JSON.stringify({ table_id: tableId, buy_in: buyIn, wallet_mode: walletMode }),
  });
}

export async function leaveTable(tableId: string): Promise<void> {
  await request<unknown>("/api/lobby/leave", {
    method: "POST",
    body: JSON.stringify({ table_id: tableId }),
  });
}

export async function createWsTicket(tableId: string): Promise<WebSocketTicketResponse> {
  return request<WebSocketTicketResponse>(`/api/lobby/tables/${tableId}/ws-ticket`, {
    method: "POST",
  });
}

export async function listAdminClubs(): Promise<ClubResponse[]> {
  return request<ClubResponse[]>("/api/admin/clubs");
}

export async function getClubFinancials(clubId: string): Promise<ClubFinancialsResponse> {
  return request<ClubFinancialsResponse>(`/api/admin/clubs/${clubId}/financials`);
}

export async function listClubAgents(clubId: string): Promise<ClubAgentResponse[]> {
  return request<ClubAgentResponse[]>(`/api/admin/clubs/${clubId}/agents`);
}

export async function createClubAgent(
  clubId: string,
  name: string,
  rakebackPercentage: number,
): Promise<ClubAgentResponse> {
  return request<ClubAgentResponse>(`/api/admin/clubs/${clubId}/agents`, {
    method: "POST",
    body: JSON.stringify({
      name,
      rakeback_percentage: rakebackPercentage,
    }),
  });
}

export async function fetchMe(): Promise<MeResponse> {
  return request<MeResponse>("/api/auth/me");
}

export async function fetchAdminStats(): Promise<AdminStatsResponse> {
  return request<AdminStatsResponse>("/api/admin/stats");
}

export async function listAdminUsers(params?: {
  q?: string;
  status?: string;
  limit?: number;
  offset?: number;
}): Promise<{ users: AdminUserResponse[]; total: number }> {
  const sp = new URLSearchParams();
  if (params?.q) sp.set("q", params.q);
  if (params?.status) sp.set("status", params.status);
  if (params?.limit != null) sp.set("limit", String(params.limit));
  if (params?.offset != null) sp.set("offset", String(params.offset));
  const qs = sp.toString();
  return request(`/api/admin/users${qs ? `?${qs}` : ""}`);
}

export async function patchAdminUser(
  id: string,
  body: { status?: string; role?: string },
): Promise<AdminUserResponse> {
  return request(`/api/admin/users/${id}`, {
    method: "PATCH",
    body: JSON.stringify(body),
  });
}

export async function adjustUserBalance(
  id: string,
  deltaCents: number,
  reason: string,
): Promise<{ user_id: string; balance: number }> {
  return request(`/api/admin/users/${id}/adjust-balance`, {
    method: "POST",
    body: JSON.stringify({ delta_cents: deltaCents, reason }),
  });
}

export async function listAdminTables(): Promise<AdminTableListItem[]> {
  return request<AdminTableListItem[]>("/api/admin/tables");
}

export async function createAdminCashTable(body: {
  name: string;
  small_blind: number;
  big_blind: number;
  min_buy_in: number;
  max_buy_in: number;
  max_players: number;
  rake_basis_points: number;
  rake_cap: number;
}): Promise<unknown> {
  return request("/api/admin/tables", {
    method: "POST",
    body: JSON.stringify(body),
  });
}

export async function patchAdminTableStatus(id: string, status: string): Promise<unknown> {
  return request(`/api/admin/tables/${id}/status`, {
    method: "PATCH",
    body: JSON.stringify({ status }),
  });
}

export async function listAdminTournaments(): Promise<AdminTournamentItem[]> {
  return request<AdminTournamentItem[]>("/api/admin/tournaments");
}

export async function listAdminTournamentPlayers(
  id: string,
): Promise<AdminTournamentPlayer[]> {
  return request(`/api/admin/tournaments/${id}/players`);
}

export async function patchAdminTournament(
  id: string,
  status: string,
): Promise<AdminTournamentItem> {
  return request(`/api/admin/tournaments/${id}`, {
    method: "PATCH",
    body: JSON.stringify({ status }),
  });
}

export async function fetchAdminPresence(): Promise<AdminPresenceResponse> {
  return request<AdminPresenceResponse>("/api/admin/presence");
}

export async function listAuditLogs(params?: {
  limit?: number;
  action?: string;
}): Promise<AuditLogItem[]> {
  const sp = new URLSearchParams();
  if (params?.limit != null) sp.set("limit", String(params.limit));
  if (params?.action) sp.set("action", params.action);
  const qs = sp.toString();
  return request(`/api/admin/audit-logs${qs ? `?${qs}` : ""}`);
}

export async function fetchAntifraudAlerts(): Promise<AntifraudAlertSummary> {
  return request<AntifraudAlertSummary>("/api/admin/antifraud/alerts");
}

export async function fetchDepositInfo(): Promise<DepositInfoResponse> {
  return request<DepositInfoResponse>("/api/wallet/deposit-info");
}

export async function createPixDeposit(
  body: { amount: number; payer_tax_number: string },
  idempotencyKey: string,
): Promise<PixDepositResponse> {
  return request("/api/payments/pix/deposit", {
    method: "POST",
    headers: { "Idempotency-Key": idempotencyKey },
    body: JSON.stringify(body),
  });
}

export async function getPixDepositStatus(txId: string): Promise<PixDepositStatusResponse> {
  return request(`/api/payments/pix/deposit/${encodeURIComponent(txId)}`);
}

export async function simulatePixDeposit(txId: string): Promise<PixDepositStatusResponse> {
  return request(`/api/payments/pix/deposit/${encodeURIComponent(txId)}/simulate`, {
    method: "POST",
  });
}
export async function listMyDepositRequests(): Promise<DepositRequestResponse[]> {
  return request<DepositRequestResponse[]>("/api/wallet/deposit-requests");
}

export async function createDepositRequest(body: {
  amount_cents: number;
  proof_text: string;
  player_note?: string;
}): Promise<DepositRequestResponse> {
  return request("/api/wallet/deposit-requests", {
    method: "POST",
    body: JSON.stringify(body),
  });
}

export async function listAdminDepositRequests(params?: {
  status?: string;
}): Promise<DepositRequestResponse[]> {
  const sp = new URLSearchParams();
  if (params?.status) sp.set("status", params.status);
  const qs = sp.toString();
  return request(`/api/admin/deposit-requests${qs ? `?${qs}` : ""}`);
}

export async function approveDepositRequest(id: string): Promise<DepositRequestResponse> {
  return request(`/api/admin/deposit-requests/${id}/approve`, { method: "POST" });
}

export async function rejectDepositRequest(
  id: string,
  adminNote?: string,
): Promise<DepositRequestResponse> {
  return request(`/api/admin/deposit-requests/${id}/reject`, {
    method: "POST",
    body: JSON.stringify({ admin_note: adminNote ?? "" }),
  });
}

export function applyAuthTokens(tokens: TokenResponse): void {
  saveTokens(tokens.token, tokens.refresh_token ?? "");
  sessionExpiredNotified = false;
  emitSessionRestored();
}

export interface OnlinePresenceResponse {
  online_count: number;
  ttl_seconds: number;
}

export interface PresenceHeartbeatResponse extends OnlinePresenceResponse {
  ok: boolean;
}

/** Contagem pública de usuários autenticados com heartbeat recente. */
export async function getOnlinePresence(): Promise<OnlinePresenceResponse> {
  return request<OnlinePresenceResponse>("/api/presence/online", {}, false);
}

/** Renova presença do usuário logado e devolve a contagem atualizada. */
export async function sendPresenceHeartbeat(): Promise<PresenceHeartbeatResponse> {
  return request<PresenceHeartbeatResponse>("/api/presence/heartbeat", {
    method: "POST",
  });
}
