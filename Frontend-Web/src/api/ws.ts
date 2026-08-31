import { ApiError, createWsTicket } from "./client";
import type { ClientMessage, ServerMessage } from "./types";

export type WsStatus = "disconnected" | "connecting" | "connected" | "error";

export interface TableWsHandlers {
  onStatus?: (status: WsStatus, detail?: string) => void;
  onMessage?: (msg: ServerMessage) => void;
}

function wsUrl(tableId: string, ticket: string): string {
  const proto = window.location.protocol === "https:" ? "wss:" : "ws:";
  const host = window.location.host;
  return `${proto}//${host}/ws/game/${encodeURIComponent(tableId)}?ticket=${encodeURIComponent(ticket)}`;
}

export class TableSocket {
  private ws: WebSocket | null = null;
  private pingTimer: number | null = null;
  private reconnectTimer: number | null = null;
  private reconnectAttempt = 0;
  private connecting = false;
  private closedByUser = false;

  private readonly handleOnline = () => {
    if (this.closedByUser) return;
    this.clearReconnect();
    void this.connect();
  };

  constructor(
    private tableId: string,
    private handlers: TableWsHandlers,
  ) {
    window.addEventListener("online", this.handleOnline);
  }

  async connect(): Promise<void> {
    if (
      this.connecting ||
      this.ws?.readyState === WebSocket.CONNECTING ||
      this.ws?.readyState === WebSocket.OPEN
    ) {
      return;
    }

    this.closedByUser = false;
    this.connecting = true;
    this.clearReconnect();
    this.handlers.onStatus?.("connecting", "Conectando à mesa…");

    try {
      const { ticket } = await createWsTicket(this.tableId);
      if (this.closedByUser) return;

      const url = wsUrl(this.tableId, ticket);
      const ws = new WebSocket(url);
      this.ws = ws;

      ws.onopen = () => {
        this.connecting = false;
        this.reconnectAttempt = 0;
        this.clearPing();
        this.handlers.onStatus?.("connected");
        this.send({ type: "get_table_info" });
        this.pingTimer = window.setInterval(() => {
          this.send({ type: "ping" });
        }, 20_000);
      };

      ws.onmessage = (ev) => {
        try {
          const msg = JSON.parse(String(ev.data)) as ServerMessage;
          this.handlers.onMessage?.(msg);
        } catch {
          this.handlers.onStatus?.("error", "Mensagem inválida do servidor");
        }
      };

      ws.onerror = () => {
        this.handlers.onStatus?.("error", "Falha na conexão WebSocket");
      };

      ws.onclose = () => {
        this.ws = null;
        this.connecting = false;
        this.clearPing();
        if (!this.closedByUser) {
          this.handlers.onStatus?.("disconnected", "Conexão com a mesa interrompida");
          this.scheduleReconnect();
        }
      };
    } catch (error) {
      this.connecting = false;
      const message = error instanceof Error ? error.message : "Erro ao conectar";
      this.handlers.onStatus?.("error", message);
      if (!(error instanceof ApiError && error.status === 401)) {
        this.scheduleReconnect();
      }
    }
  }

  send(msg: ClientMessage): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(msg));
    }
  }

  sendAction(action: string, amount = 0): void {
    this.send({ type: "action", action, amount });
  }

  disconnect(): void {
    this.closedByUser = true;
    this.connecting = false;
    this.clearPing();
    this.clearReconnect();
    window.removeEventListener("online", this.handleOnline);
    this.ws?.close();
    this.ws = null;
    this.handlers.onStatus?.("disconnected");
  }

  private scheduleReconnect(): void {
    if (this.closedByUser || this.reconnectTimer != null) return;
    if (!navigator.onLine) {
      this.handlers.onStatus?.("disconnected", "Sem internet — aguardando reconexão");
      return;
    }

    const delay = Math.min(1_000 * 2 ** this.reconnectAttempt, 15_000);
    this.reconnectAttempt += 1;
    this.handlers.onStatus?.(
      "connecting",
      `Reconectando automaticamente em ${Math.ceil(delay / 1_000)}s…`,
    );
    this.reconnectTimer = window.setTimeout(() => {
      this.reconnectTimer = null;
      void this.connect();
    }, delay);
  }

  private clearPing(): void {
    if (this.pingTimer != null) {
      window.clearInterval(this.pingTimer);
      this.pingTimer = null;
    }
  }

  private clearReconnect(): void {
    if (this.reconnectTimer != null) {
      window.clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
  }
}
