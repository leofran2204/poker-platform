import { ApiError } from "./errors";

/** 401/403 no ticket WS: não reconectar em loop. */
export function isFatalWsTicketError(error: unknown): error is ApiError {
  return error instanceof ApiError && (error.status === 401 || error.status === 403);
}

export function fatalWsTicketMessage(error: ApiError): string {
  return error.status === 403
    ? "Você não está mais nesta mesa (sem assento ativo) — volte ao lobby para entrar de novo."
    : error.message;
}
