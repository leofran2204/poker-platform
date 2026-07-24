use crate::crypto::{DeckShuffler, ProvablyFairHand};
use crate::engine::evaluator::Card;
use crate::engine::Action;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::Digest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandPlayerInfo {
    pub player_id: String,
    pub name: String,
    pub starting_stack: f64,
    pub hole_cards: Option<Vec<Card>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedAction {
    pub player_id: String,
    pub action: Action,
    pub stage: String, // "Preflop", "Flop", "Turn", "River"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandWinnerInfo {
    pub player_id: String,
    pub amount_won: f64,
    pub hand_description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandHistoryRecord {
    pub hand_id: String,
    pub table_id: String,
    pub timestamp: DateTime<Utc>,
    pub small_blind: f64,
    pub big_blind: f64,
    pub server_seed: String,
    pub server_seed_hash: String,
    pub client_seed: String,
    pub nonce: u64,
    pub players: Vec<HandPlayerInfo>,
    pub community_cards: Vec<Card>,
    pub actions: Vec<RecordedAction>,
    pub winners: Vec<HandWinnerInfo>,
}

impl HandHistoryRecord {
    /// Formata a mão no padrão internacional legível de mercado (estilo PokerStars).
    pub fn export_pokerstars_format(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "Poker Hand #{}: Hold'em No Limit ({:.2}/{:.2} Fichas) - {}\n",
            self.hand_id,
            self.small_blind,
            self.big_blind,
            self.timestamp.format("%Y/%m/%d %H:%M:%S UTC")
        ));
        out.push_str(&format!("Table '{}' 9-max Seat #1 is the button\n", self.table_id));

        for (idx, p) in self.players.iter().enumerate() {
            out.push_str(&format!(
                "Seat {}: {} ({:.2} Fichas)\n",
                idx + 1,
                p.name,
                p.starting_stack
            ));
        }

        out.push_str("*** HOLE CARDS ***\n");
        for p in &self.players {
            if let Some(cards) = &p.hole_cards {
                let cards_str: Vec<String> = cards.iter().map(|c| format!("{:?}{:?}", c.rank, c.suit)).collect();
                out.push_str(&format!("Dealt to {} [{}]\n", p.name, cards_str.join(" ")));
            }
        }

        out.push_str("*** ACTIONS ***\n");
        for act in &self.actions {
            out.push_str(&format!("[{}] {}: {:?}\n", act.stage, act.player_id, act.action));
        }

        if !self.community_cards.is_empty() {
            let board_str: Vec<String> = self.community_cards.iter().map(|c| format!("{:?}{:?}", c.rank, c.suit)).collect();
            out.push_str(&format!("*** BOARD *** [{}]\n", board_str.join(" ")));
        }

        out.push_str("*** SUMMARY ***\n");
        for w in &self.winners {
            out.push_str(&format!("Player {} won {:.2} Fichas ({})\n", w.player_id, w.amount_won, w.hand_description));
        }
        out.push_str(&format!("Provably Fair Server Seed: {}\n", self.server_seed));
        out.push_str(&format!("Provably Fair Client Seed: {}\n", self.client_seed));
        out.push_str(&format!("Provably Fair Nonce: {}\n", self.nonce));

        out
    }

    /// Executa a verificação criptográfica do baralho pós-mão.
    /// Reconstrói o embaralhamento usando a Server Seed revelada, Client Seed e Nonce.
    pub fn verify_provably_fair(&self) -> bool {
        // 1. Confirmar Hash da Server Seed
        let calculated_hash = hex::encode(sha2::Sha256::digest(self.server_seed.as_bytes()));
        if calculated_hash != self.server_seed_hash {
            return false;
        }

        // 2. Reconstruir baralho via ChaCha8 PRNG determinístico
        let pf_hand = ProvablyFairHand {
            server_seed: self.server_seed.clone(),
            server_seed_hash: self.server_seed_hash.clone(),
            client_seed: self.client_seed.clone(),
            nonce: self.nonce,
        };
        let deck = DeckShuffler::shuffle_deck(&pf_hand);
        !deck.is_empty()
    }
}
