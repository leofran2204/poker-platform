import { fetchMe } from "@/api/client";
import type { MeResponse } from "@/api/types";
import { getToken } from "@/lib/auth";

let cache: MeResponse | null = null;
let inflight: Promise<MeResponse> | null = null;

export function clearMeCache(): void {
  cache = null;
  inflight = null;
}

export async function getMe(force = false): Promise<MeResponse | null> {
  if (!getToken()) {
    clearMeCache();
    return null;
  }
  if (!force && cache) return cache;
  if (!force && inflight) return inflight;
  inflight = fetchMe()
    .then((me) => {
      cache = me;
      return me;
    })
    .catch((e) => {
      clearMeCache();
      throw e;
    })
    .finally(() => {
      inflight = null;
    });
  return inflight;
}

export function isAdminRole(role: string | undefined | null): boolean {
  return role === "admin";
}
