export interface TokenResponse {
  token: string;
  refresh_token?: string;
  expires_in?: number;
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
