import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { SESSION_EXPIRED_EVENT } from "@/lib/sessionEvents";

const TOKEN_KEY = "poker_jwt";
const REFRESH_KEY = "poker_refresh";

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

function bearerOf(callIndex: number): string | null {
  const init = fetchMock.mock.calls[callIndex][1] as RequestInit;
  return new Headers(init.headers).get("Authorization");
}

/** `client.ts` guarda estado de módulo; cada teste carrega uma cópia nova. */
async function loadClient() {
  vi.resetModules();
  return import("./client");
}

beforeEach(() => {
  store = new Map([
    [TOKEN_KEY, "access-1"],
    [REFRESH_KEY, "refresh-1"],
  ]);
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

describe("renovação da sessão", () => {
  it("guarda o refresh token novo e repete a chamada com o access token novo", async () => {
    fetchMock
      .mockResolvedValueOnce(json(401, { error: "Token expired" }))
      .mockResolvedValueOnce(json(200, { token: "access-2", refresh_token: "refresh-2", expires_in: 900 }))
      .mockResolvedValueOnce(json(200, { username: "ana" }));
    const client = await loadClient();

    await expect(client.fetchMe()).resolves.toMatchObject({ username: "ana" });

    expect(requestedPaths()).toEqual(["/api/auth/me", "/api/auth/refresh", "/api/auth/me"]);
    const refreshInit = fetchMock.mock.calls[1][1] as RequestInit;
    expect(JSON.parse(String(refreshInit.body))).toEqual({ refresh_token: "refresh-1" });
    expect(bearerOf(2)).toBe("Bearer access-2");
    expect(store.get(TOKEN_KEY)).toBe("access-2");
    expect(store.get(REFRESH_KEY)).toBe("refresh-2");
  });

  it("chamadas simultâneas na mesma aba apresentam o refresh token uma única vez", async () => {
    fetchMock.mockImplementation(async (path: string, init?: RequestInit) => {
      if (path === "/api/auth/refresh") {
        return json(200, { token: "access-2", refresh_token: "refresh-2", expires_in: 900 });
      }
      const bearer = new Headers(init?.headers).get("Authorization");
      return bearer === "Bearer access-2"
        ? json(200, { username: "ana" })
        : json(401, { error: "Token expired" });
    });
    const client = await loadClient();

    await Promise.all([client.fetchMe(), client.fetchMe(), client.fetchMe()]);

    expect(requestedPaths().filter((path) => path === "/api/auth/refresh")).toHaveLength(1);
    expect(store.get(REFRESH_KEY)).toBe("refresh-2");
  });

  it("não reapresenta o refresh token quando outra aba renovou enquanto esperava a vez", async () => {
    vi.stubGlobal("navigator", {
      locks: {
        // A outra aba segurava a trava e gravou os tokens novos antes de soltá-la.
        request: async (_name: string, task: () => Promise<unknown>) => {
          store.set(TOKEN_KEY, "access-2");
          store.set(REFRESH_KEY, "refresh-2");
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
    expect(bearerOf(1)).toBe("Bearer access-2");
    expect(store.get(REFRESH_KEY)).toBe("refresh-2");
  });

  it("encerra a sessão local quando o servidor recusa o refresh token", async () => {
    fetchMock
      .mockResolvedValueOnce(json(401, { error: "Token expired" }))
      .mockResolvedValueOnce(json(401, { error: "Invalid refresh token" }));
    const expired = vi.fn();
    window.addEventListener(SESSION_EXPIRED_EVENT, expired);
    const client = await loadClient();

    await expect(client.fetchMe()).rejects.toMatchObject({ status: 401 });

    expect(store.has(TOKEN_KEY)).toBe(false);
    expect(store.has(REFRESH_KEY)).toBe(false);
    expect(expired).toHaveBeenCalledTimes(1);
  });
});

describe("revokeSession", () => {
  it("envia o refresh token guardado mesmo se o navegador for limpo em seguida", async () => {
    fetchMock.mockResolvedValueOnce(json(200, { ok: true }));
    const client = await loadClient();

    const pending = client.revokeSession();
    store.clear();
    await pending;

    expect(requestedPaths()).toEqual(["/api/auth/logout"]);
    const init = fetchMock.mock.calls[0][1] as RequestInit;
    expect(init.method).toBe("POST");
    expect(JSON.parse(String(init.body))).toEqual({ refresh_token: "refresh-1" });
  });

  it("não chama o servidor sem refresh token e não propaga falha de rede", async () => {
    const client = await loadClient();
    fetchMock.mockRejectedValueOnce(new TypeError("network down"));
    await expect(client.revokeSession()).resolves.toBeUndefined();
    expect(fetchMock).toHaveBeenCalledTimes(1);

    store.delete(REFRESH_KEY);
    await client.revokeSession();
    expect(fetchMock).toHaveBeenCalledTimes(1);
  });
});
