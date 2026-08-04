import { createWsTicket } from "./client";
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
  private closedByUser = false;

  constructor(
    private tableId: string,
    private handlers: TableWsHandlers,
  ) {}

  async connect(): Promise<void> {
    this.closedByUser = false;
    this.handlers.onStatus?.("connecting");
    try {
      const { ticket } = await createWsTicket(this.tableId);
      const url = wsUrl(this.tableId, ticket);
      const ws = new WebSocket(url);
      this.ws = ws;

      ws.onopen = () => {
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
        this.clearPing();
        if (!this.closedByUser) {
          this.handlers.onStatus?.("disconnected");
        }
      };
    } catch (e) {
      const message = e instanceof Error ? e.message : "Erro ao conectar";
      this.handlers.onStatus?.("error", message);
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
    this.clearPing();
    this.ws?.close();
    this.ws = null;
    this.handlers.onStatus?.("disconnected");
  }

  private clearPing(): void {
    if (this.pingTimer != null) {
      window.clearInterval(this.pingTimer);
      this.pingTimer = null;
    }
  }
}
