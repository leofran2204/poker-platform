export const CONNECTION_STATUS_EVENT = "poker:connection-status";
export const SESSION_EXPIRED_EVENT = "poker:session-expired";
export const SESSION_RESTORED_EVENT = "poker:session-restored";

export type ConnectionStatus = "online" | "offline";

export interface ConnectionStatusDetail {
  status: ConnectionStatus;
}

export function emitConnectionStatus(status: ConnectionStatus): void {
  window.dispatchEvent(
    new CustomEvent<ConnectionStatusDetail>(CONNECTION_STATUS_EVENT, {
      detail: { status },
    }),
  );
}

export function emitSessionExpired(): void {
  window.dispatchEvent(new Event(SESSION_EXPIRED_EVENT));
}

export function emitSessionRestored(): void {
  window.dispatchEvent(new Event(SESSION_RESTORED_EVENT));
}
