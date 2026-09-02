export interface TokenResponse {
  token: string;
  refresh_token?: string;
  expires_in?: number;
}

export type WalletMode = "play" | "real";

export interface MeResponse {
  user_id: string;
  username: string;
  role: string;
  status: string;
  balance: number;
  balance_pm_cash: number;
  balance_pm_mtt: number;
  balance_real: number;
  preferred_wallet_mode: string;
  last_pm_reset_date?: string | null;
  pm_cash_rebuy_available: boolean;
  pm_mtt_rebuy_available: boolean;
  email: string;
}

export interface AdminStatsResponse {
  users_total: number;
  users_by_status: Record<string, number>;
  users_verified: number;
  tables_open: number;
  tables_paused: number;
  tables_closed: number;
  tournaments_open: number;
  tournament_registrations: number;
  online_count: number;
  wallet_balance_sum: number;
}

export interface AdminUserResponse {
  id: string;
  username: string;
  email: string;
  role: string;
  status: string;
  balance: number;
  email_verified: boolean;
  created_at: number;
  last_login: number | null;
  mfa_enabled: boolean;
}

export interface AdminTableSeat {
  seat: number;
  user_id: string;
  username: string;
  email: string;
  chips: number;
}

export interface AdminTableListItem {
  id: string;
  name: string;
  status: string;
  visibility: string;
  small_blind: number;
  big_blind: number;
  min_buy_in: number;
  max_buy_in: number;
  max_players: number;
  current_players: number;
  seats?: AdminTableSeat[];
}

export interface AdminTournamentItem {
  id: string;
  name: string;
  buy_in: number;
  guaranteed_prize: number;
  prize_pool: number;
  status: string;
  is_freeroll: boolean;
  registered_players: number;
  max_players: number;
  table_max_players: number;
}

export interface AdminTournamentPlayer {
  player_id: string;
  player_name: string;
  stack: number;
  rebuys: number;
  registered_at: number;
  email?: string | null;
}

export interface AdminPresenceResponse {
  online_count: number;
  users: { user_id: string; username: string; last_seen: number }[];
}

export interface AuditLogItem {
  id: string;
  user_id: string;
  action: string;
  metadata: unknown;
  created_at: string;
}

export interface DepositInfoResponse {
  available: boolean;
  pix_key: string;
  receiver_name: string;
  max_cents: number;
  max_pending: number;
  presets_cents: number[];
  instructions: string;
  automated_available: boolean;
  automated_provider?: string | null;
  automated_mode?: string | null;
}

export interface PixDepositResponse {
  tx_id: string;
  amount: number;
  pix_copy_paste: string;
  qr_code_base64: string;
  expires_at: string;
  payment_url?: string | null;
}

export interface PixDepositStatusResponse {
  tx_id: string;
  amount: number;
  status: string;
  provider_status: string;
}

export interface DepositRequestResponse {
  id: string;
  user_id: string;
  username?: string | null;
  amount_cents: number;
  status: string;
  player_note?: string | null;
  proof_text: string;
  admin_note?: string | null;
  reviewed_by?: string | null;
  created_at: string;
  reviewed_at?: string | null;
}

export interface AntifraudAlertSummary {
  bot_suspects_count: number;
  collusion_alerts_count: number;
  chip_dumping_alerts_count: number;
  system_status: string;
  recent_alerts: {
    id: string;
    alert_type: string;
    player_id: string;
    risk_score: number;
    description: string;
    timestamp: string;
  }[];
}

export interface TableResponse {
  id: string;
  name: string;
  players: number;
  max_players: number;
  small_blind: number;
  big_blind: number;
  min_buy_in: number;
  max_buy_in: number;
  game_type: string;
  money_mode?: string;
  /** `holdem` | `short_deck` | `short_deck_omaha` */
  poker_variant?: string;
}

export interface BlindLevelDto {
  level: number;
  small_blind: number;
  big_blind: number;
  ante: number;
  duration_minutes: number;
}

export interface TournamentInfoResponse {
  id: string;
  name: string;
  buy_in: number;
  starting_stack: number;
  max_players: number;
  table_max_players: number;
  registered_players: number;
  status: string;
  players_remaining: number;
  prize_pool: number;
  guaranteed_prize: number;
  is_freeroll: boolean;
  allow_rebuy: boolean;
  rebuy_cost: number;
  rebuy_chips: number;
  rebuy_max_count: number;
  rebuy_stack_threshold: number;
  rebuy_max_level: number;
  blind_levels: BlindLevelDto[];
  gameplay_ready: boolean;
  money_mode?: string;
  /** `holdem` | `short_deck` | `short_deck_omaha` */
  poker_variant?: string;
  /** Variante aplicada quando começa a mesa final (ex.: `short_deck`). */
  final_table_variant?: string | null;
  final_table_max_players?: number | null;
}

export interface JoinResponse {
  seat: number;
  chips: number;
}

export interface WebSocketTicketResponse {
  ticket: string;
  expires_in: number;
}

export interface ClubResponse {
  id?: string;
  name: string;
  subdomain: string;
  custom_theme_json?: unknown;
  status: string;
}

export interface ClubFinancialsResponse {
  club_id: string;
  name: string;
  balance: number;
  total_rake_generated: number;
  net_club_rake: number;
  platform_fee_paid: number;
}

export interface ClubAgentResponse {
  agent_id: string;
  name: string;
  rakeback_percentage: number;
  total_players_referred: number;
  total_commission_earned: number;
}

export interface PlayerWsData {
  id: string;
  name: string;
  chips: number;
  bet: number;
  cards: string[];
  is_active: boolean;
  is_dealer: boolean;
  seat: number;
}

export interface PotWsData {
  name: string;
  amount: number;
  eligible_players: string[];
}

export type ServerMessage =
  | { type: "welcome"; player_id: string; seat: number }
  | {
      type: "table_state";
      players: PlayerWsData[];
      community_cards: string[];
      stage: string;
      pots: PotWsData[];
      available_actions: string[];
      call_amount: number;
      minimum_wager: number;
      maximum_wager: number;
    }
  | { type: "your_turn"; actions: string[]; time_bank: number }
  | { type: "action_result"; success: boolean; message: string }
  | { type: "pong" }
  | {
      type: "table_info";
      name: string;
      small_blind: number;
      big_blind: number;
      game_type: string;
    }
  | { type: "error"; message: string }
  | {
      type: "deflator_triggered";
      loser_name: string;
      winner_name: string;
      cashback_amount: number;
      deflator_percent?: number;
      loser_equity_percent?: number;
      odds_broken?: number;
      prevented_elimination: boolean;
      is_tournament: boolean;
    };

export type ClientMessage =
  | { type: "action"; action: string; amount?: number }
  | { type: "ping" }
  | { type: "get_table_info" };
