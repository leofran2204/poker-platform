import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { SESSION_EXPIRED_EVENT } from "@/lib/sessionEvents";

// Sessão por cookie (23/11/2026): os tokens ficam em cookies HttpOnly que o
// fetch manda sozinho; no localStorage só há o marcador `poker_session`.
// `poker_jwt` e `poker_refresh` são da sessão antiga, só para a migração.
const SESSION_KEY = "poker_session";
const LEGACY_TOKEN_KEY = "poker_jwt";
const LEGACY_REFRESH_KEY = "poker_refresh";
const REFRESHED_AT_KEY = "poker_session_refreshed_at";

let store: Map<string, string>;
let fetchMock: ReturnType<typeof vi.fn>;

function json(status: number, body: unknown): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { "Content-Type": "application/json" },
  });
}

function requestedPaths(): string[] {
  return fetchMock.mock.calls.map(([path]) => String(path));
}

function headersOf(callIndex: number): Headers {
  const init = fetchMock.mock.calls[callIndex][1] as RequestInit;
  return new Headers(init.headers);
}

function bodyOf(callIndex: number): unknown {
  const init = fetchMock.mock.calls[callIndex][1] as RequestInit;
  return JSON.parse(String(init.body));
}

/** `client.ts` guarda estado de módulo; cada teste carrega uma cópia nova. */
async function loadClient() {
  vi.resetModules();
  return import("./client");
}

beforeEach(() => {
  store = new Map([[SESSION_KEY, "1"]]);
  vi.stubGlobal("localStorage", {
    getItem: (key: string) => store.get(key) ?? null,
    setItem: (key: string, value: string) => void store.set(key, value),
    removeItem: (key: string) => void store.delete(key),
  });
  vi.stubGlobal("window", new EventTarget());
  vi.stubGlobal("navigator", {});
  fetchMock = vi.fn();
  vi.stubGlobal("fetch", fetchMock);
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("sessão por cookie", () => {
  it("não manda token e pede a sessão por cookie com o cabeçalho anti-CSRF", async () => {
    fetchMock.mockResolvedValueOnce(json(200, { username: "ana" }));
    const client = await loadClient();

    await client.fetchMe();

    const headers = headersOf(0);
    expect(headers.get("Authorization")).toBeNull();
    expect(headers.get("X-ZT-CSRF")).toBe("1");
    expect(headers.get("X-ZT-Session")).toBe("cookie");
  });

  it("renova pelo cookie, sem token no corpo, e repete a chamada", async () => {
    fetchMock
      .mockResolvedValueOnce(json(401, { error: "Token expired" }))
      .mockResolvedValueOnce(json(200, { session: "cookie", expires_in: 900 }))
      .mockResolvedValueOnce(json(200, { username: "ana" }));
    const client = await loadClient();

    await expect(client.fetchMe()).resolves.toMatchObject({ username: "ana" });

    expect(requestedPaths()).toEqual(["/api/auth/me", "/api/auth/refresh", "/api/auth/me"]);
    expect(bodyOf(1)).toEqual({});
    expect(headersOf(1).get("X-ZT-CSRF")).toBe("1");
    expect(store.get(SESSION_KEY)).toBe("1");
  });

  it("migra a sessão antiga: usa o token guardado e troca pelo cookie na renovação", async () => {
    store = new Map([
      [LEGACY_TOKEN_KEY, "access-1"],
      [LEGACY_REFRESH_KEY, "refresh-1"],
    ]);
    fetchMock
      .mockResolvedValueOnce(json(401, { error: "Token expired" }))
      .mockResolvedValueOnce(json(200, { session: "cookie", expires_in: 900 }))
      .mockResolvedValueOnce(json(200, { username: "ana" }));
    const client = await loadClient();

    await expect(client.fetchMe()).resolves.toMatchObject({ username: "ana" });

    expect(headersOf(0).get("Authorization")).toBe("Bearer access-1");
    expect(bodyOf(1)).toEqual({ refresh_token: "refresh-1" });
    expect(headersOf(2).get("Authorization")).toBeNull();
    expect(store.has(LEGACY_TOKEN_KEY)).toBe(false);
    expect(store.has(LEGACY_REFRESH_KEY)).toBe(false);
    expect(store.get(SESSION_KEY)).toBe("1");
  });

  it("chamadas simultâneas na mesma aba renovam uma única vez", async () => {
    let renewed = false;
    fetchMock.mockImplementation(async (path: string) => {
      if (path === "/api/auth/refresh") {
        renewed = true;
        return json(200, { session: "cookie", expires_in: 900 });
      }
      return renewed ? json(200, { username: "ana" }) : json(401, { error: "Token expired" });
    });
    const client = await loadClient();

    await Promise.all([client.fetchMe(), client.fetchMe(), client.fetchMe()]);

    expect(requestedPaths().filter((path) => path === "/api/auth/refresh")).toHaveLength(1);
  });

  it("não renova de novo quando outra aba acabou de renovar enquanto esperava a vez", async () => {
    vi.stubGlobal("navigator", {
      locks: {
        // A outra aba segurava a trava e renovou os cookies antes de soltá-la.
        request: async (_name: string, task: () => Promise<unknown>) => {
          store.set(REFRESHED_AT_KEY, String(Date.now()));
          return task();
        },
      },
    });
    fetchMock
      .mockResolvedValueOnce(json(401, { error: "Token expired" }))
      .mockResolvedValueOnce(json(200, { username: "ana" }));
    const client = await loadClient();

    await expect(client.fetchMe()).resolves.toMatchObject({ username: "ana" });

    expect(requestedPaths()).toEqual(["/api/auth/me", "/api/auth/me"]);
  });

  it("encerra a sessão local quando o servidor recusa a renovação", async () => {
    fetchMock
      .mockResolvedValueOnce(json(401, { error: "Token expired" }))
      .mockResolvedValueOnce(json(401, { error: "Invalid refresh token" }));
    const expired = vi.fn();
    window.addEventListener(SESSION_EXPIRED_EVENT, expired);
    const client = await loadClient();

    await expect(client.fetchMe()).rejects.toMatchObject({ status: 401 });

    expect(store.has(SESSION_KEY)).toBe(false);
    expect(expired).toHaveBeenCalledTimes(1);
  });

  it("reconhece a sessão aberta pelo corpo sem tokens", async () => {
    const client = await loadClient();
    expect(client.hasSession({ session: "cookie" })).toBe(true);
    expect(client.hasSession({ token: "jwt" })).toBe(true);
    expect(client.hasSession({})).toBe(false);
  });
});

describe("revokeSession", () => {
  it("encerra a sessão do cookie no servidor com o cabeçalho anti-CSRF", async () => {
    fetchMock.mockResolvedValueOnce(json(200, { ok: true }));
    const client = await loadClient();

    await client.revokeSession();

    expect(requestedPaths()).toEqual(["/api/auth/logout"]);
    const init = fetchMock.mock.calls[0][1] as RequestInit;
    expect(init.method).toBe("POST");
    expect(bodyOf(0)).toEqual({});
    expect(headersOf(0).get("X-ZT-CSRF")).toBe("1");
  });

  it("envia o refresh token da sessão antiga mesmo se o navegador for limpo em seguida", async () => {
    store = new Map([
      [LEGACY_TOKEN_KEY, "access-1"],
      [LEGACY_REFRESH_KEY, "refresh-1"],
    ]);
    fetchMock.mockResolvedValueOnce(json(200, { ok: true }));
    const client = await loadClient();

    const pending = client.revokeSession();
    store.clear();
    await pending;

    expect(bodyOf(0)).toEqual({ refresh_token: "refresh-1" });
  });

  it("não propaga falha de rede", async () => {
    const client = await loadClient();
    fetchMock.mockRejectedValueOnce(new TypeError("network down"));
    await expect(client.revokeSession()).resolves.toBeUndefined();
  });
});
