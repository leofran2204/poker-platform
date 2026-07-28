//! Módulo de Criptografia Auditável (Provably Fair) para o Motor de Poker.
//!
//! Fornece um mecanismo transparente e auditável de embaralhamento de baralho
//! baseado em HMAC-SHA256, amplamente utilizado em cassinos online regulamentados.
//!
//! # Como Funciona:
//! 1. **Server Seed**: O servidor gera uma semente secreta criptográfica (32 bytes).
//! 2. **Server Hash**: Antes do início da partida, o servidor envia aos jogadores apenas o **SHA-256(Server Seed)**.
//! 3. **Client Seed**: Os clientes (jogadores) fornecem uma semente pública própria (ex: string aleatória ou hash de sessão do cliente).
//! 4. **Nonce**: Número sequencial da mão no jogo (0, 1, 2...).
//! 5. **Embaralhamento Determinístico**: O baralho é embaralhado com o algoritmo Fisher-Yates guiado por um stream
//!    derivado de `HMAC-SHA256(key = Server Seed, msg = "ClientSeed:Nonce:Index")`.
//! 6. **Auditoria**: Após a conclusão da mão, o `Server Seed` em texto claro é revelado. Qualquer jogador pode
//!    re-executar a função `verify_shuffle` e comprovar que o baralho não foi alterado durante o jogo.

use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

type HmacSha256 = Hmac<Sha256>;

/// Estado de uma semente Provably Fair para uma partida ou mesa.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvablyFairState {
    /// Semente secreta gerada pelo servidor (32 bytes hex)
    pub server_seed_hex: String,
    /// Hash público revelado antes da mão iniciar (SHA256 do server_seed)
    pub server_hash: String,
    /// Semente combinada fornecida pelos jogadores
    pub client_seed: String,
    /// Contador da mão (nonce)
    pub nonce: u64,
}

impl ProvablyFairState {
    /// Gera um novo estado com Server Seed aleatório seguro.
    pub fn new(client_seed: impl Into<String>, nonce: u64) -> Self {
        let server_seed_bytes = generate_server_seed();
        let server_seed_hex = hex_encode(&server_seed_bytes);
        let server_hash = hash_server_seed_bytes(&server_seed_bytes);

        Self {
            server_seed_hex,
            server_hash,
            client_seed: client_seed.into(),
            nonce,
        }
    }

    /// Incrementa o nonce para a próxima mão.
    pub fn next_hand(&mut self) {
        self.nonce += 1;
    }
}

/// Gera 32 bytes de entropia aleatória para a semente do servidor.
pub fn generate_server_seed() -> [u8; 32] {
    let mut buf = [0u8; 32];
    crate::rng_crypto::secure_random_bytes(&mut buf);
    buf
}

/// Calcula o hash SHA-256 público a partir dos bytes da semente do servidor.
pub fn hash_server_seed_bytes(server_seed: &[u8; 32]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(server_seed);
    hex_encode(&hasher.finalize())
}

/// Calcula o hash SHA-256 público a partir de uma semente hexadecimal.
pub fn hash_server_seed_hex(server_seed_hex: &str) -> Result<String, String> {
    let bytes = hex_decode(server_seed_hex)?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(hex_encode(&hasher.finalize()))
}

/// Embaralha deterministicamente uma lista de itens usando HMAC-SHA256.
///
/// O algoritmo utiliza Fisher-Yates. Para cada posição `i` de `n-1` até `1`,
/// gera um número aleatório determinístico no intervalo `[0, i]` baseado no HMAC da semente.
pub fn provably_fair_shuffle<T: Clone>(
    items: &mut [T],
    server_seed_bytes: &[u8],
    client_seed: &str,
    nonce: u64,
) -> Result<(), String> {
    let len = items.len();
    if len <= 1 {
        return Ok(());
    }

    for (round, i) in (0_u32..).zip((1..len).rev()) {
        // Deriva valor numérico uniforme em [0, i] usando HMAC
        let random_index =
            get_deterministic_u32(server_seed_bytes, client_seed, nonce, round, (i + 1) as u32)?;

        items.swap(i, random_index as usize);
    }

    Ok(())
}

/// Verifica se a sequência final obtida corresponde exatamente ao resultado determinístico
/// que deveria ser gerado pela semente revelada.
pub fn verify_shuffle<T: Clone + PartialEq>(
    original_items: &[T],
    final_items: &[T],
    server_seed_hex: &str,
    client_seed: &str,
    nonce: u64,
) -> Result<bool, String> {
    if original_items.len() != final_items.len() {
        return Ok(false);
    }

    let server_seed_bytes = hex_decode(server_seed_hex)?;
    let mut items_to_reconstruct = original_items.to_vec();

    provably_fair_shuffle(
        &mut items_to_reconstruct,
        &server_seed_bytes,
        client_seed,
        nonce,
    )?;

    Ok(items_to_reconstruct == final_items)
}

// ─── Helpers Internos de Criptografia ───

fn get_deterministic_u32(
    server_seed: &[u8],
    client_seed: &str,
    nonce: u64,
    round: u32,
    bound: u32,
) -> Result<u32, String> {
    let mut mac = HmacSha256::new_from_slice(server_seed)
        .map_err(|e| format!("Erro ao inicializar HMAC: {e}"))?;

    let message = format!("{client_seed}:{nonce}:{round}");
    mac.update(message.as_bytes());
    let result = mac.finalize().into_bytes();

    // Pega os primeiros 4 bytes do hash HMAC para formar um u32
    let mut num_bytes = [0u8; 4];
    num_bytes.copy_from_slice(&result[0..4]);
    let raw_u32 = u32::from_be_bytes(num_bytes);

    // Mapeia uniformemente para [0, bound - 1]
    Ok(raw_u32 % bound)
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn hex_decode(hex_str: &str) -> Result<Vec<u8>, String> {
    if !hex_str.len().is_multiple_of(2) {
        return Err("String hexadecimal inválida: tamanho ímpar".to_string());
    }

    (0..hex_str.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex_str[i..i + 2], 16)
                .map_err(|e| format!("Byte hex inválido na posição {i}: {e}"))
        })
        .collect()
}

// ─── Testes Unitários ───

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provably_fair_determinism() {
        let server_seed = generate_server_seed();
        let client_seed = "player1_seed_abc123";
        let nonce = 42;

        let original_deck: Vec<u32> = (0..52).collect();

        let mut deck1 = original_deck.clone();
        let mut deck2 = original_deck.clone();

        provably_fair_shuffle(&mut deck1, &server_seed, client_seed, nonce).unwrap();
        provably_fair_shuffle(&mut deck2, &server_seed, client_seed, nonce).unwrap();

        // O embaralhamento com as mesmas sementes deve ser 100% idêntico
        assert_eq!(deck1, deck2);
        assert_ne!(deck1, original_deck);
    }

    #[test]
    fn test_provably_fair_seed_change_produces_different_results() {
        let server_seed1 = generate_server_seed();
        let server_seed2 = generate_server_seed();
        let client_seed = "player1_seed";
        let nonce = 1;

        let original_deck: Vec<u32> = (0..52).collect();

        let mut deck1 = original_deck.clone();
        let mut deck2 = original_deck.clone();

        provably_fair_shuffle(&mut deck1, &server_seed1, client_seed, nonce).unwrap();
        provably_fair_shuffle(&mut deck2, &server_seed2, client_seed, nonce).unwrap();

        // Sementes diferentes devem resultar em embaralhamentos totalmente diferentes
        assert_ne!(deck1, deck2);
    }

    #[test]
    fn test_verification_valid_and_invalid() {
        let server_seed_bytes = generate_server_seed();
        let server_seed_hex = hex_encode(&server_seed_bytes);
        let client_seed = "community_client_seed";
        let nonce = 10;

        let original_deck: Vec<u8> = (1..=52).collect();
        let mut shuffled_deck = original_deck.clone();

        provably_fair_shuffle(&mut shuffled_deck, &server_seed_bytes, client_seed, nonce).unwrap();

        // Verificação válida
        let is_valid = verify_shuffle(
            &original_deck,
            &shuffled_deck,
            &server_seed_hex,
            client_seed,
            nonce,
        )
        .unwrap();
        assert!(is_valid, "A verificação do baralho deveria ter passado");

        // Verificação com semente alterada deve falhar
        let is_valid_tampered = verify_shuffle(
            &original_deck,
            &shuffled_deck,
            "0000000000000000000000000000000000000000000000000000000000000000",
            client_seed,
            nonce,
        )
        .unwrap();
        assert!(
            !is_valid_tampered,
            "A verificação de semente alterada deveria ter falhado"
        );
    }

    #[test]
    fn test_server_hash_verification() {
        let server_seed_bytes = generate_server_seed();
        let server_seed_hex = hex_encode(&server_seed_bytes);

        let hash_from_bytes = hash_server_seed_bytes(&server_seed_bytes);
        let hash_from_hex = hash_server_seed_hex(&server_seed_hex).unwrap();

        assert_eq!(hash_from_bytes, hash_from_hex);
        assert_eq!(hash_from_bytes.len(), 64); // SHA-256 hex string tem 64 chars
    }

    // ─── TESTES MASSIVOS DE ESTRESSE & DISTRIBUIÇÃO ESTATÍSTICA ───

    #[test]
    #[cfg_attr(
        not(feature = "massive-tests"),
        ignore = "distribuicao de 500 mil amostras; habilite a feature massive-tests manualmente"
    )]
    fn test_provably_fair_massive_distribution_chi_squared() {
        // Roda 500.000 embaralhamentos auditáveis e calcula distribuição qui-quadrado da 1ª carta
        let iterations = 500_000usize;
        let mut counts = [0usize; 52];
        let original_deck: Vec<u8> = (0..52).collect();

        for i in 0..iterations {
            let mut server_seed = [0u8; 32];
            server_seed[0..8].copy_from_slice(&(i as u64).to_le_bytes());
            let client_seed = format!("client_seed_mass_{i}");
            let mut deck = original_deck.clone();

            provably_fair_shuffle(&mut deck, &server_seed, &client_seed, i as u64).unwrap();
            counts[deck[0] as usize] += 1;
        }

        // Teste de Qui-Quadrado (Chi-Squared) com 51 graus de liberdade
        let expected = iterations as f64 / 52.0;
        let mut chi2 = 0.0f64;
        for &c in &counts {
            let diff = c as f64 - expected;
            chi2 += (diff * diff) / expected;
        }

        // Para df=51 com p=0.001 (confiança de 99.9%), o valor crítico é ~88.0.
        // Se chi2 < 88.0, a distribuição das 500.000 iterações é estatisticamente uniforme e justa.
        assert!(
            chi2 < 88.0,
            "Chi-squared de Provably Fair falhou em 500k iterações: {chi2:.2} (esperado < 88.0)"
        );
    }
}

// ─── Proptests com Alta Amostragem para Invariantes de Entrada Arbitrária ───

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(500))]

        #[test]
        fn proptest_provably_fair_always_preserves_elements(
            seed_bytes in proptest::array::uniform32(any::<u8>()),
            client_seed in "\\PC*",
            nonce in any::<u64>()
        ) {
            let original_deck: Vec<u8> = (0..52).collect();
            let mut deck = original_deck.clone();

            let res = provably_fair_shuffle(&mut deck, &seed_bytes, &client_seed, nonce);
            prop_assert!(res.is_ok());

            // Invariante 1: Tamanho preservado
            prop_assert_eq!(deck.len(), 52);

            // Invariante 2: Todos os 52 elementos continuam presentes (sem duplicação nem perda)
            let mut sorted = deck.clone();
            sorted.sort();
            prop_assert_eq!(sorted, original_deck);
        }

        #[test]
        fn proptest_provably_fair_verification_property(
            seed_bytes in proptest::array::uniform32(any::<u8>()),
            client_seed in "[a-zA-Z0-9_-]{1,64}",
            nonce in any::<u64>()
        ) {
            let original_deck: Vec<u8> = (0..52).collect();
            let mut shuffled = original_deck.clone();
            provably_fair_shuffle(&mut shuffled, &seed_bytes, &client_seed, nonce).unwrap();

            let hex_seed = hex_encode(&seed_bytes);
            let verified = verify_shuffle(&original_deck, &shuffled, &hex_seed, &client_seed, nonce).unwrap();

            // Invariante: Qualquer baralho gerado com a semente DEVE ser validado com sucesso pela verify_shuffle
            prop_assert!(verified);
        }
    }
}
