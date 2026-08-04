import { getToken, saveTokens, clearTokens } from "@/lib/auth";
import type {
  ClubAgentResponse,
  ClubFinancialsResponse,
  ClubResponse,
  JoinResponse,
  TableResponse,
  TokenResponse,
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

async function request<T>(
  path: string,
  init: RequestInit = {},
  auth = true,
): Promise<T> {
  const headers = new Headers(init.headers);
  if (!headers.has("Content-Type") && init.body) {
    headers.set("Content-Type", "application/json");
  }
  if (auth) {
    const token = getToken();
    if (token) headers.set("Authorization", `Bearer ${token}`);
  }

  const res = await fetch(path, { ...init, headers });
  if (res.status === 401 && auth) {
    // keep local state honest on hard auth failures
    if (!path.includes("/auth/")) {
      /* caller may redirect */
    }
  }
  if (!res.ok) {
    throw new ApiError(await parseError(res), res.status);
  }
  if (res.status === 204) return undefined as T;
  return (await res.json()) as T;
}

export async function register(
  username: string,
  email: string,
  password: string,
): Promise<TokenResponse> {
  return request<TokenResponse>(
    "/api/auth/register",
    {
      method: "POST",
      body: JSON.stringify({ username, email, password }),
    },
    false,
  );
}

export async function login(email: string, password: string): Promise<TokenResponse> {
  return request<TokenResponse>(
    "/api/auth/login",
    {
      method: "POST",
      body: JSON.stringify({ email, password }),
    },
    false,
  );
}

export async function listTables(): Promise<TableResponse[]> {
  return request<TableResponse[]>("/api/lobby/tables");
}

export async function getTable(id: string): Promise<TableResponse> {
  return request<TableResponse>(`/api/lobby/tables/${id}`);
}

export async function joinTable(tableId: string, buyIn: number): Promise<JoinResponse> {
  return request<JoinResponse>("/api/lobby/join", {
    method: "POST",
    body: JSON.stringify({ table_id: tableId, buy_in: buyIn }),
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

export function applyAuthTokens(tokens: TokenResponse): void {
  saveTokens(tokens.token, tokens.refresh_token ?? "");
}

export function logout(): void {
  clearTokens();
}
