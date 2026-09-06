use crate::engine::evaluator::{Card, Rank, Suit};
use rand::{Rng, RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvablyFairHand {
    pub server_seed: String,
    pub server_seed_hash: String,
    pub client_seed: String,
    pub nonce: u64,
}

impl ProvablyFairHand {
    pub fn new(client_seed: impl Into<String>, nonce: u64) -> Self {
        let mut rng = rand::thread_rng();
        let mut seed_bytes = [0u8; 32];
        rng.fill_bytes(&mut seed_bytes);

        let server_seed = hex::encode(seed_bytes);
        let server_seed_hash = Self::hash_seed(&server_seed);

        Self {
            server_seed,
            server_seed_hash,
            client_seed: client_seed.into(),
            nonce,
        }
    }

    pub fn hash_seed(seed: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(seed.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn generate_combined_seed(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.server_seed.as_bytes());
        hasher.update(self.client_seed.as_bytes());
        hasher.update(self.nonce.to_le_bytes());
        let result = hasher.finalize();

        let mut seed = [0u8; 32];
        seed.copy_from_slice(&result);
        seed
    }

    /// Valida se uma semente revelada corresponde ao hash de compromisso inicial.
    pub fn verify_commitment(server_seed: &str, expected_hash: &str) -> bool {
        Self::hash_seed(server_seed) == expected_hash
    }
}

pub struct DeckShuffler;

impl DeckShuffler {
    pub fn generate_standard_deck() -> Vec<Card> {
        let suits = [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades];
        let ranks = [
            Rank::Two,
            Rank::Three,
            Rank::Four,
            Rank::Five,
            Rank::Six,
            Rank::Seven,
            Rank::Eight,
            Rank::Nine,
            Rank::Ten,
            Rank::Jack,
            Rank::Queen,
            Rank::King,
            Rank::Ace,
        ];

        let mut deck = Vec::with_capacity(52);
        for suit in suits {
            for rank in ranks {
                deck.push(Card::new(rank, suit));
            }
        }
        deck
    }

    /// Embaralha um baralho usando o algoritmo Fisher-Yates com determinismo auditável (Provably Fair).
    pub fn shuffle_deck(pf_hand: &ProvablyFairHand) -> Vec<Card> {
        let mut deck = Self::generate_standard_deck();
        let seed = pf_hand.generate_combined_seed();
        let mut rng = ChaCha8Rng::from_seed(seed);

        // Fisher-Yates shuffle determinístico
        for i in (1..deck.len()).rev() {
            let j = rng.gen_range(0..=i);
            deck.swap(i, j);
        }

        deck
    }
}
