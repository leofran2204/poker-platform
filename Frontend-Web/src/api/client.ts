import { clearTokens, getLegacyRefreshToken, getLegacyToken, markSession } from "@/lib/auth";
import { ApiError } from "./errors";
import {
  emitConnectionStatus,
  emitSessionExpired,
  emitSessionRestored,
} from "@/lib/sessionEvents";
import type {
  AdminPresenceResponse,
  AdminKycItem,
  AdminLiabilitiesResponse,
  AdminStatsResponse,
  AdminTableListItem,
  AdminTournamentItem,
  AdminTournamentPlayer,
  AdminUserResponse,
  AgentCloseResponse,
  AgentModeSummary,
  AgentConfigResponse,
  EstruturaBackfillPreview,
  EstruturaBackfillResult,
  AntifraudAlertSummary,
  AuditLogItem,
  DepositInfoResponse,
  DepositRequestResponse,
  EstruturaResponse,
  PixDepositResponse,
  PixDepositStatusResponse,
  ClubAgentResponse,
  ClubFinancialsResponse,
  ClubResponse,
  JoinResponse,
  KycStatusResponse,
  MeResponse,
  PlayHeartbeatResponse,
  ResponsibleGamingStatus,
  ResponsibleLimits,
  SupportTicketResponse,
  TableResponse,
  TokenResponse,
  TournamentInfoResponse,
  WalletLedgerEntry,
  WalletTransactionItem,
  WebSocketTicketResponse,
} from "./types";

export { ApiError } from "./errors";

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

/**
 * Sessão por cookie: o par de tokens fica em cookies HttpOnly gravados pela
 * API. Cada refresh token vale uma única renovação e as abas dividem os mesmos
 * cookies; a trava entre abas deixa uma renovar por vez e as que esperavam
 * reaproveitam a renovação recém-feita. Navegador sem Web Locks segue sem a
 * trava: a API tolera a corrida curta.
 */
const SESSION_HEADERS = { "X-ZT-CSRF": "1", "X-ZT-Session": "cookie" } as const;
const REFRESHED_AT_KEY = "poker_session_refreshed_at";

async function withRefreshLock(task: () => Promise<RefreshResult>): Promise<RefreshResult> {
  const locks = typeof navigator !== "undefined" ? navigator.locks : undefined;
  if (!locks) return task();
  const result = await locks.request("zt-auth-refresh", task);
  return result;
}

async function refreshAccessToken(): Promise<RefreshResult> {
  // Outra aba renovou enquanto esta esperava a vez.
  const refreshedAt = Number(localStorage.getItem(REFRESHED_AT_KEY) ?? 0);
  if (Date.now() - refreshedAt < 5_000) return "refreshed";

  // Sessão antiga (tokens no localStorage): a renovação migra para o cookie.
  const legacyRefresh = getLegacyRefreshToken();
  let res: Response;
  try {
    res = await fetch("/api/auth/refresh", {
      method: "POST",
      headers: { "Content-Type": "application/json", ...SESSION_HEADERS },
      body: JSON.stringify(legacyRefresh ? { refresh_token: legacyRefresh } : {}),
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

  markSession();
  localStorage.setItem(REFRESHED_AT_KEY, String(Date.now()));
  sessionExpiredNotified = false;
  emitSessionRestored();
  return "refreshed";
}

function tryRefreshAccessToken(): Promise<RefreshResult> {
  if (!refreshInFlight) {
    refreshInFlight = withRefreshLock(() => refreshAccessToken()).finally(() => {
      refreshInFlight = null;
    });
  }
  return refreshInFlight;
}

/**
 * Encerra a sessão no servidor: o refresh token do cookie (ou o da sessão
 * antiga) deixa de renovar e a API apaga os cookies. Lê o token antigo antes de
 * qualquer `await`; quem chama pode limpar o `localStorage` logo em seguida.
 * Sem rede, a sessão expira sozinha.
 */
export async function revokeSession(): Promise<void> {
  const legacyRefresh = getLegacyRefreshToken();
  try {
    await fetch("/api/auth/logout", {
      method: "POST",
      headers: { "Content-Type": "application/json", ...SESSION_HEADERS },
      body: JSON.stringify(legacyRefresh ? { refresh_token: legacyRefresh } : {}),
      keepalive: true,
    });
  } catch {
    /* O logout local já aconteceu. */
  }
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
  // O cookie da sessão vai sozinho (mesma origem). Os cabeçalhos pedem a sessão
  // por cookie e liberam as escritas na API (proteção contra CSRF).
  for (const [name, value] of Object.entries(SESSION_HEADERS)) headers.set(name, value);
  const legacyToken = auth ? getLegacyToken() : null;
  if (legacyToken) headers.set("Authorization", `Bearer ${legacyToken}`);

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
  /** `cookie`: a sessão veio em cookies HttpOnly e o corpo não traz tokens. */
  session?: string;
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
  inviteCode?: string,
  dateOfBirth?: string,
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
        invite_code: inviteCode || undefined,
        date_of_birth: dateOfBirth || undefined,
        over_18: true,
      }),
    },
    false,
  );
}

export async function forgotPassword(email: string): Promise<{ ok: boolean; message: string }> {
  return request(
    "/api/auth/forgot-password",
    { method: "POST", body: JSON.stringify({ email }) },
    false,
  );
}

export async function resetPassword(body: {
  email: string;
  code: string;
  password: string;
  password_confirm: string;
}): Promise<{ ok: boolean; message: string }> {
  return request(
    "/api/auth/reset-password",
    { method: "POST", body: JSON.stringify(body) },
    false,
  );
}

export async function fetchResponsibleGaming(): Promise<ResponsibleGamingStatus> {
  return request("/api/responsible-gaming/settings");
}

export async function updateResponsibleLimits(
  limits: ResponsibleLimits,
): Promise<{ ok: boolean; delayed: boolean; message: string }> {
  return request("/api/responsible-gaming/settings", {
    method: "PUT",
    body: JSON.stringify(limits),
  });
}

export async function startSelfExclusion(
  duration: "24h" | "7d" | "30d" | "180d" | "permanent",
  confirmation: string,
): Promise<{ ok: boolean; self_excluded_until: string | null; permanent: boolean }> {
  return request("/api/responsible-gaming/self-exclusion", {
    method: "POST",
    body: JSON.stringify({ duration, confirmation }),
  });
}

export async function sendPlayHeartbeat(): Promise<PlayHeartbeatResponse> {
  return request("/api/responsible-gaming/play-heartbeat", { method: "POST" });
}

export async function fetchKyc(): Promise<KycStatusResponse> {
  return request("/api/kyc");
}

export async function submitKyc(body: {
  legal_name: string;
  tax_id: string;
  date_of_birth: string;
}): Promise<KycStatusResponse> {
  return request("/api/kyc", { method: "POST", body: JSON.stringify(body) });
}

export async function listMySupportTickets(): Promise<SupportTicketResponse[]> {
  return request("/api/support/tickets");
}

export async function createSupportTicket(body: {
  category: string;
  subject: string;
  message: string;
}): Promise<SupportTicketResponse> {
  return request("/api/support/tickets", { method: "POST", body: JSON.stringify(body) });
}

export async function listAdminKyc(status = "pending"): Promise<AdminKycItem[]> {
  return request(`/api/admin/kyc?status=${encodeURIComponent(status)}`);
}

export async function reviewAdminKyc(
  userId: string,
  status: "verified" | "rejected",
): Promise<{ ok: boolean; status: string }> {
  return request(`/api/admin/kyc/${encodeURIComponent(userId)}`, {
    method: "PATCH",
    body: JSON.stringify({ status }),
  });
}

export async function listAdminSupportTickets(status = "open"): Promise<SupportTicketResponse[]> {
  return request(`/api/admin/support/tickets?status=${encodeURIComponent(status)}`);
}

export async function replyAdminSupportTicket(
  id: string,
  status: "in_progress" | "resolved" | "closed",
  response: string,
): Promise<SupportTicketResponse> {
  return request(`/api/admin/support/tickets/${encodeURIComponent(id)}`, {
    method: "PATCH",
    body: JSON.stringify({ status, response }),
  });
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

export async function unregisterTournament(tournamentId: string): Promise<{
  tournament_id: string;
  player_id: string;
  refunded_buy_in_cents: number;
  refunded_fee_cents: number;
}> {
  return request("/api/tournament/unregister", {
    method: "POST",
    body: JSON.stringify({ tournament_id: tournamentId }),
  });
}

export interface TournamentRegistration {
  registered: boolean;
  eliminated?: boolean;
  rebuys?: number;
  addon_done?: boolean;
  /** O servidor aceitaria um rebuy agora. */
  can_rebuy?: boolean;
  /** O servidor aceitaria o add-on agora. */
  can_addon?: boolean;
}

export async function fetchTournamentRegistration(
  tournamentId: string,
): Promise<TournamentRegistration> {
  return request<TournamentRegistration>(`/api/tournament/${tournamentId}/registration`);
}

export interface ChipPurchaseResult {
  tournament_id: string;
  player_id: string;
  kind: "rebuy" | "addon";
  chips: number;
  cost_cents: number;
  /** `seated` = voltou a uma mesa agora; `next_hand` = fichas na próxima mão. */
  delivery: "seated" | "next_hand";
  table_id: string | null;
  rebuys: number;
  addon_done: boolean;
  prize_pool: number;
}

export async function buyTournamentChips(
  tournamentId: string,
  kind: "rebuy" | "addon",
): Promise<ChipPurchaseResult> {
  return request(`/api/tournament/${kind}`, {
    method: "POST",
    body: JSON.stringify({ tournament_id: tournamentId }),
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

export async function joinWaitlist(tableId: string): Promise<{
  table_id: string;
  position: number;
  length: number;
}> {
  return request("/api/lobby/waitlist", {
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

export async function fetchEstrutura(cycleStart?: string): Promise<EstruturaResponse> {
  return request<EstruturaResponse>(`/api/estrutura${cycleStart ? `?cycle_start=${encodeURIComponent(cycleStart)}` : ""}`);
}

export async function previewEstruturaBackfill(): Promise<EstruturaBackfillPreview> {
  return request<EstruturaBackfillPreview>("/api/admin/estrutura/backfill");
}

export async function executeEstruturaBackfill(
  rootUserId: string,
): Promise<EstruturaBackfillResult> {
  return request<EstruturaBackfillResult>("/api/admin/estrutura/backfill", {
    method: "POST",
    body: JSON.stringify({ root_user_id: rootUserId, confirm: true }),
  });
}

export async function configureAgent(
  id: string,
  body: {
    status?: "inactive" | "active" | "suspended";
    cycle_start?: string;
    money_mode?: "play" | "real";
    target_ngr_cents?: number;
  },
): Promise<AgentConfigResponse> {
  return request<AgentConfigResponse>(`/api/admin/agentes/${id}`, {
    method: "PATCH",
    body: JSON.stringify(body),
  });
}

export async function addAgentAdjustment(
  id: string,
  body: {
    request_id: string;
    cycle_start: string;
    money_mode: "play" | "real";
    category: "reward" | "refund" | "chargeback" | "tax" | "payment_cost" | "other";
    amount_cents: number;
    note: string;
  },
): Promise<{ id: string; amount_cents: number }> {
  return request(`/api/admin/agentes/${id}/adjustments`, {
    method: "POST",
    body: JSON.stringify(body),
  });
}

export async function previewAgentCycle(id: string, cycleStart: string, moneyMode: "play" | "real"): Promise<AgentModeSummary> {
  const query = new URLSearchParams({ cycle_start: cycleStart, money_mode: moneyMode });
  return request<AgentModeSummary>(`/api/admin/agentes/${id}/cycle?${query}`);
}

export async function closeAgentCycle(
  id: string,
  cycleStart: string,
  moneyMode: "play" | "real",
  preview: AgentModeSummary,
): Promise<AgentCloseResponse> {
  return request<AgentCloseResponse>(`/api/admin/agentes/${id}/close`, {
    method: "POST",
    body: JSON.stringify({
      cycle_start: cycleStart, money_mode: moneyMode, reconciled: true,
      expected_gross_revenue_cents: preview.gross_revenue_cents,
      expected_deductions_cents: preview.deductions_cents,
      expected_target_ngr_cents: preview.target_ngr_cents,
    }),
  });
}

export async function fetchAdminLiabilities(): Promise<AdminLiabilitiesResponse> {
  return request<AdminLiabilitiesResponse>("/api/admin/liabilities");
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
  wallet: "pm_cash" | "pm_mtt" | "real",
): Promise<{ user_id: string; balance: number; wallet: string }> {
  return request(`/api/admin/users/${id}/adjust-balance`, {
    method: "POST",
    body: JSON.stringify({ delta_cents: deltaCents, reason, wallet }),
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

export interface BotsPoolResponse {
  created: number;
  total: number;
  pool_size: number;
}

export interface BotsStartResponse {
  table_id: string;
  table_name: string;
  strategy: string;
  bots: string[];
}

export interface BotGroup {
  personality: string;
  count: number;
}

export interface BotSeatStatus {
  username: string;
  chips: number;
  personality?: string | null;
}

export interface BotTableStatus {
  table_id: string;
  table_name: string;
  strategy: string;
  variant: string;
  started_at: number;
  bots_total: number;
  bots_alive: number;
  hands_played: number;
  leader: BotSeatStatus | null;
  seats: BotSeatStatus[];
  personalities: Record<string, string>;
}

export interface BotTournamentPlayer {
  username: string;
  personality?: string | null;
}

export interface BotTournamentDeploy {
  tournament_id: string;
  tournament_name: string;
  strategy: string;
  bots_total: number;
  personalities: Record<string, string>;
  players: BotTournamentPlayer[];
}

export interface BotsStatusResponse {
  pool_total: number;
  pool_free: number;
  strategies: string[];
  tables: BotTableStatus[];
  tournaments: BotTournamentDeploy[];
}

export async function ensureBotsPool(): Promise<BotsPoolResponse> {
  return request<BotsPoolResponse>("/api/admin/bots/ensure-pool", { method: "POST" });
}

export async function startBots(
  tableId: string,
  count: number,
  strategy: string,
  groups?: BotGroup[],
): Promise<BotsStartResponse> {
  return request<BotsStartResponse>("/api/admin/bots/start", {
    method: "POST",
    body: JSON.stringify({ table_id: tableId, count, strategy, groups }),
  });
}

export async function stopBots(tableId: string): Promise<{ table_id: string; refunded_chips: number }> {
  return request("/api/admin/bots/stop", {
    method: "POST",
    body: JSON.stringify({ table_id: tableId }),
  });
}

export async function fetchBotsStatus(): Promise<BotsStatusResponse> {
  return request<BotsStatusResponse>("/api/admin/bots/status");
}

export interface BotsTournamentStartResponse {
  tournament_id: string;
  tournament_name: string;
  strategy: string;
  bots: string[];
}

export async function startTournamentBots(
  tournamentId: string,
  count: number,
  strategy: string,
  groups?: BotGroup[],
): Promise<BotsTournamentStartResponse> {
  return request<BotsTournamentStartResponse>("/api/admin/bots/start-tournament", {
    method: "POST",
    body: JSON.stringify({ tournament_id: tournamentId, count, strategy, groups }),
  });
}

export async function stopTournamentBots(
  tournamentId: string,
): Promise<{ tournament_id: string; bots_stopped: number }> {
  return request("/api/admin/bots/stop-tournament", {
    method: "POST",
    body: JSON.stringify({ tournament_id: tournamentId }),
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

/** Rebuy e add-on do evento (centavos e fichas). Só antes do início. */
export interface RebuyAddonInput {
  allow_rebuy?: boolean;
  rebuy_cost_cents?: number;
  rebuy_chips?: number;
  /** 0 = sem limite. */
  rebuy_max_count?: number;
  rebuy_max_level?: number;
  rebuy_stack_threshold?: number;
  allow_addon?: boolean;
  addon_cost_cents?: number;
  addon_chips?: number;
  addon_max_level?: number;
}

export async function updateAdminTournamentRebuyAddon(
  id: string,
  input: RebuyAddonInput,
): Promise<AdminTournamentItem> {
  return request(`/api/admin/tournaments/${id}`, {
    method: "PATCH",
    body: JSON.stringify(input),
  });
}

export async function rescheduleAdminTournament(
  id: string,
  scheduledStartAt: number,
): Promise<AdminTournamentItem> {
  return request(`/api/admin/tournaments/${id}`, {
    method: "PATCH",
    body: JSON.stringify({ scheduled_start_at: scheduledStartAt }),
  });
}

export interface CreateTournamentInput {
  name: string;
  buy_in_cents: number;
  starting_stack: number;
  table_max_players: number;
  poker_variant: string;
  money_mode: string;
  guaranteed_prize_cents: number;
  scheduled_start_at: number;
}

export type CreateTournamentRequest = CreateTournamentInput & RebuyAddonInput;

export async function createAdminTournament(
  input: CreateTournamentRequest,
): Promise<AdminTournamentItem> {
  return request<AdminTournamentItem>("/api/admin/tournaments", {
    method: "POST",
    body: JSON.stringify(input),
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

export async function createPixWithdraw(body: {
  amount: number;
  pix_key: string;
  pix_key_type: string;
  /** CPF do titular (o mesmo do KYC); a API recusa outro CPF ou CNPJ. */
  tax_number: string;
}): Promise<{ tx_id: string; amount: number; status: string; message: string }> {
  return request("/api/payments/pix/withdraw", {
    method: "POST",
    body: JSON.stringify(body),
  });
}
export async function listMyDepositRequests(): Promise<DepositRequestResponse[]> {
  return request<DepositRequestResponse[]>("/api/wallet/deposit-requests");
}

export async function listMyWalletTransactions(): Promise<WalletTransactionItem[]> {
  return request<WalletTransactionItem[]>("/api/wallet/transactions");
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

/** A resposta de entrada na conta abriu a sessão (cookies gravados pela API). */
export function hasSession(response: { token?: string; session?: string }): boolean {
  return response.session === "cookie" || Boolean(response.token);
}

/** Depois do login, da verificação ou do MFA: os cookies já vieram na resposta. */
export function applyAuthTokens(_tokens?: Partial<TokenResponse>): void {
  markSession();
  localStorage.setItem(REFRESHED_AT_KEY, String(Date.now()));
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

/** Encerra a presença no servidor antes que o TTL natural expire. */
export async function sendPresenceOffline(): Promise<PresenceHeartbeatResponse> {
  return request<PresenceHeartbeatResponse>("/api/presence/offline", {
    method: "POST",
    keepalive: true,
  });
}

export interface CourseProgressItem {
  lesson_id: string;
  status: string;
  best_score: number;
  attempts: number;
  updated_at: string;
}

/** Progresso do curso do usuário logado. */
export async function fetchCourseProgress(): Promise<CourseProgressItem[]> {
  return request<CourseProgressItem[]>("/api/course/progress");
}

/** Envia as respostas do quiz; o servidor corrige e devolve a melhor nota da aula. */
export async function saveCourseProgress(
  lessonId: string,
  answers: Record<string, string>,
): Promise<CourseProgressItem> {
  return request<CourseProgressItem>("/api/course/progress", {
    method: "POST",
    body: JSON.stringify({ lesson_id: lessonId, answers }),
  });
}

/** Extrato completo da conta: toda mudança de saldo das três carteiras (até 200). */
export async function fetchWalletLedger(): Promise<WalletLedgerEntry[]> {
  return request("/api/wallet/ledger?limit=200");
}
