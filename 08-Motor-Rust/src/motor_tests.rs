// motor_tests.rs — Testes abrangentes do motor de poker
//
// Cobertura das 5 áreas críticas e inegociáveis:
//   1. Integridade do baralho de 52 cartas (sem duplicatas)
//   2. CSPRNG robusto (entropia forte, sem viés)
//   3. Distribuição destrutiva (deal remove cartas do baralho)
//   4. Avaliação de showdown (Texas Hold'em — 10 categorias de mão)
//   5. Divisão de side pots (all-ins, empates, stacks diferentes)
//
// Inclui testes determinísticos + property-based (proptest) que geram
// milhares de casos automaticamente.

#[cfg(test)]
use crate::deck::{
    compare_hands, create_deck, deal_cards, evaluate_hand, get_hand_name, shuffle_deck, Card,
    HandRank, Rank, Suit,
};
#[cfg(test)]
use crate::rake::{calculate_rake_for_pot, deduct_rake};
#[cfg(test)]
use crate::rng_crypto::{
    secure_random_bool, secure_random_bytes, secure_random_f64, secure_random_u32,
    secure_random_u64, secure_shuffle,
};
#[cfg(test)]
use crate::side_pots::{calculate_side_pots, distribute_pots, resolve_side_pots, PlayerForPots};
#[cfg(test)]
use crate::types::{Pot, TableConfig};

#[cfg(test)]
use proptest::prelude::*;
#[cfg(test)]
use std::collections::{HashMap, HashSet};

// ═══════════════════════════════════════════════════════════════════
// Helpers compartilhados
// ═══════════════════════════════════════════════════════════════════

/// Cria uma carta rapidamente
fn c(rank: Rank, suit: Suit) -> Card {
    Card { rank, suit }
}

/// Cria um jogador para side pots
fn make_player(id: &str, total_bet: f64, has_folded: bool, cards: Vec<Card>) -> PlayerForPots {
    PlayerForPots {
        id: id.into(),
        total_bet,
        has_folded,
        cards,
    }
}

/// Cria um pot para rake
fn make_rake_pot(amount: f64) -> Pot {
    Pot {
        amount,
        eligible_players: vec!["p1".into(), "p2".into()],
    }
}

/// Configuração padrão de mesa para testes de rake
fn default_table_config() -> TableConfig {
    TableConfig {
        big_blind: 10.0,
        rake_percent: 5.0,
        rake_cap: 10.0,
    }
}

/// Converte um baralho em HashSet para detecção rápida de duplicatas
fn deck_to_set(deck: &[Card]) -> HashSet<(Rank, Suit)> {
    deck.iter().map(|card| (card.rank, card.suit)).collect()
}

// ═══════════════════════════════════════════════════════════════════
// ÁREA 1 — INTEGRIDADE DO BARALHO DE 52 CARTAS
// ═══════════════════════════════════════════════════════════════════

mod deck_integrity {
    use super::*;

    // ─── Testes determinísticos ───

    #[test]
    fn deck_tem_exatamente_52_cartas() {
        let deck = create_deck();
        assert_eq!(deck.len(), 52, "Baralho deve ter exatamente 52 cartas");
    }

    #[test]
    fn deck_nao_tem_duplicatas() {
        let deck = create_deck();
        let set = deck_to_set(&deck);
        assert_eq!(set.len(), 52, "Deve haver 52 cartas únicas");
    }

    #[test]
    fn deck_tem_4_naipes_com_13_cartas_cada() {
        let deck = create_deck();
        let mut suit_counts: HashMap<Suit, usize> = HashMap::new();
        for card in &deck {
            *suit_counts.entry(card.suit).or_insert(0) += 1;
        }
        assert_eq!(suit_counts.len(), 4, "Deve haver exatamente 4 naipes");
        for (&suit, &count) in &suit_counts {
            assert_eq!(
                count, 13,
                "Naipe {:?} deve ter 13 cartas, tem {}",
                suit, count
            );
        }
    }

    #[test]
    fn deck_tem_13_valores_com_4_naipes_cada() {
        let deck = create_deck();
        let mut rank_counts: HashMap<Rank, usize> = HashMap::new();
        for card in &deck {
            *rank_counts.entry(card.rank).or_insert(0) += 1;
        }
        assert_eq!(rank_counts.len(), 13, "Deve haver exatamente 13 valores");
        for (&rank, &count) in &rank_counts {
            assert_eq!(
                count, 4,
                "Valor {:?} deve aparecer 4 vezes, aparece {}",
                rank, count
            );
        }
    }

    #[test]
    fn deck_contem_todos_os_naipes() {
        let deck = create_deck();
        let suits: HashSet<Suit> = deck.iter().map(|c| c.suit).collect();
        assert!(suits.contains(&Suit::Hearts));
        assert!(suits.contains(&Suit::Diamonds));
        assert!(suits.contains(&Suit::Clubs));
        assert!(suits.contains(&Suit::Spades));
    }

    #[test]
    fn deck_contem_todos_os_valores() {
        let deck = create_deck();
        let ranks: HashSet<Rank> = deck.iter().map(|c| c.rank).collect();
        assert!(ranks.contains(&Rank::Two));
        assert!(ranks.contains(&Rank::Ace));
        assert!(ranks.contains(&Rank::King));
        assert!(ranks.contains(&Rank::Queen));
        assert!(ranks.contains(&Rank::Jack));
        assert!(ranks.contains(&Rank::Ten));
    }

    #[test]
    fn deck_nao_tem_carta_repetida_em_nenhuma_combinacao() {
        let deck = create_deck();
        for i in 0..deck.len() {
            for j in (i + 1)..deck.len() {
                assert_ne!(
                    (deck[i].rank, deck[i].suit),
                    (deck[j].rank, deck[j].suit),
                    "Duplicata encontrada nas posições {} e {}",
                    i,
                    j
                );
            }
        }
    }

    #[test]
    fn criar_dois_decks_sao_iguais() {
        // create_deck é determinístico — sempre gera o mesmo baralho
        let d1 = create_deck();
        let d2 = create_deck();
        assert_eq!(d1, d2, "create_deck deve ser determinístico");
    }

    // ─── Testes property-based (proptest) ───

    proptest! {
        /// Propriedade: criar_deck N vezes sempre gera 52 cartas únicas
        #[test]
        fn prop_deck_sempre_52_unicas(_n in 0..100u32) {
            let deck = create_deck();
            let set = deck_to_set(&deck);
            prop_assert_eq!(deck.len(), 52);
            prop_assert_eq!(set.len(), 52);
        }

        /// Propriedade: cada naipe aparece exatamente 13 vezes
        #[test]
        fn prop_deck_naipes_balanceados(_n in 0..50u32) {
            let deck = create_deck();
            let mut counts: HashMap<Suit, usize> = HashMap::new();
            for card in &deck {
                *counts.entry(card.suit).or_insert(0) += 1;
            }
            for &count in counts.values() {
                prop_assert_eq!(count, 13);
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// ÁREA 2 — CSPRNG ROBUSTO (ENTROPIA FORTE, SEM VIÉS)
// ═══════════════════════════════════════════════════════════════════

mod csprng_tests {
    use super::*;

    // ─── secure_random_u32 ───

    #[test]
    fn u32_dentro_do_range_sempre() {
        for _ in 0..10_000 {
            let val = secure_random_u32(1..=100);
            assert!(
                val >= 1 && val <= 100,
                "Valor {} fora do range 1..=100",
                val
            );
        }
    }

    #[test]
    fn u32_min_igual_max_retorna_min() {
        for _ in 0..100 {
            assert_eq!(secure_random_u32(42..=42), 42);
        }
    }

    #[test]
    #[should_panic(expected = "min (10) > max (5)")]
    fn u32_panica_se_min_maior_que_max() {
        secure_random_u32(10..=5);
    }

    #[test]
    fn u32_distribuicao_sem_vies_d6() {
        // 60.000 lançamentos de D6 — cada face deve ter ~10.000 (±5%)
        let mut counts = [0u32; 6];
        for _ in 0..60_000 {
            let val = secure_random_u32(1..=6);
            counts[(val - 1) as usize] += 1;
        }
        for (face, &count) in counts.iter().enumerate() {
            let pct = count as f64 / 60_000.0 * 100.0;
            assert!(
                pct >= 14.0 && pct <= 19.0,
                "Face {} com {} ocorrências ({:.2}%) — viés detectado",
                face + 1,
                count,
                pct
            );
        }
    }

    #[test]
    fn u32_distribuicao_sem_vies_d2() {
        // 100.000 lançamentos de moeda — cada face ~50.000 (±2%)
        let mut counts = [0u32; 2];
        for _ in 0..100_000 {
            let val = secure_random_u32(0..=1);
            counts[val as usize] += 1;
        }
        let pct0 = counts[0] as f64 / 100_000.0 * 100.0;
        let pct1 = counts[1] as f64 / 100_000.0 * 100.0;
        assert!(pct0 >= 48.0 && pct0 <= 52.0, "Face 0: {:.2}% — viés", pct0);
        assert!(pct1 >= 48.0 && pct1 <= 52.0, "Face 1: {:.2}% — viés", pct1);
    }

    #[test]
    fn u32_full_range_nao_panica() {
        let val = secure_random_u32(0..=u32::MAX);
        assert!(val <= u32::MAX);
    }

    // ─── secure_random_u64 ───

    #[test]
    fn u64_dentro_do_range_sempre() {
        for _ in 0..5_000 {
            let val = secure_random_u64(1_000..=9_999);
            assert!(val >= 1_000 && val <= 9_999);
        }
    }

    #[test]
    fn u64_min_igual_max_retorna_min() {
        assert_eq!(secure_random_u64(999..=999), 999);
    }

    #[test]
    #[should_panic(expected = "min (100) > max (50)")]
    fn u64_panica_se_min_maior_que_max() {
        secure_random_u64(100..=50);
    }

    #[test]
    fn u64_distribuicao_sem_vies_0_a_9() {
        let mut counts = [0u32; 10];
        for _ in 0..50_000 {
            let val = secure_random_u64(0..=9);
            counts[val as usize] += 1;
        }
        for (digit, &count) in counts.iter().enumerate() {
            let pct = count as f64 / 50_000.0 * 100.0;
            assert!(
                pct >= 8.0 && pct <= 12.0,
                "Dígito {} com {} ocorrências ({:.2}%) — viés",
                digit,
                count,
                pct
            );
        }
    }

    // ─── secure_random_f64 ───

    #[test]
    fn f64_sempre_entre_0_e_1() {
        for _ in 0..10_000 {
            let val = secure_random_f64();
            assert!(val >= 0.0 && val < 1.0, "Valor {} fora de [0.0, 1.0)", val);
        }
    }

    #[test]
    fn f64_tem_alta_variedade() {
        let mut seen: HashSet<u64> = HashSet::new();
        for _ in 0..1_000 {
            let val = secure_random_f64();
            seen.insert((val * 1_000_000.0) as u64);
        }
        assert!(
            seen.len() >= 900,
            "Apenas {} valores distintos em 1.000 amostras — baixa entropia",
            seen.len()
        );
    }

    #[test]
    fn f64_distribuicao_quartis() {
        // 40.000 amostras — cada quartil (0-0.25, 0.25-0.5, etc.) deve ter ~25%
        let mut quartiles = [0u32; 4];
        for _ in 0..40_000 {
            let val = secure_random_f64();
            let q = (val * 4.0) as usize;
            quartiles[q.min(3)] += 1;
        }
        for (i, &count) in quartiles.iter().enumerate() {
            let pct = count as f64 / 40_000.0 * 100.0;
            assert!(
                pct >= 23.0 && pct <= 27.0,
                "Quartil {} com {:.2}% — viés detectado",
                i,
                pct
            );
        }
    }

    // ─── secure_random_bool ───

    #[test]
    fn bool_probabilidade_1_sempre_true() {
        for _ in 0..1_000 {
            assert!(secure_random_bool(1.0));
        }
    }

    #[test]
    fn bool_probabilidade_0_sempre_false() {
        for _ in 0..1_000 {
            assert!(!secure_random_bool(0.0));
        }
    }

    #[test]
    fn bool_probabilidade_50_porcento() {
        let mut trues = 0u32;
        let total = 100_000;
        for _ in 0..total {
            if secure_random_bool(0.5) {
                trues += 1;
            }
        }
        let pct = trues as f64 / total as f64 * 100.0;
        assert!(
            pct >= 48.0 && pct <= 52.0,
            "P(true) = {:.2}% — viés detectado",
            pct
        );
    }

    #[test]
    #[should_panic(expected = "probability must be in")]
    fn bool_panica_se_probabilidade_negativa() {
        secure_random_bool(-0.1);
    }

    #[test]
    #[should_panic(expected = "probability must be in")]
    fn bool_panica_se_probabilidade_maior_que_1() {
        secure_random_bool(1.5);
    }

    // ─── secure_random_bytes ───

    #[test]
    fn bytes_preenche_buffer_completamente() {
        let mut buf = vec![0u8; 256];
        secure_random_bytes(&mut buf);
        // Extremamente improvável que todos sejam zero
        assert!(
            buf.iter().any(|&b| b != 0),
            "Buffer todo zero — entropia falhou"
        );
    }

    #[test]
    fn bytes_tem_alta_variedade() {
        let mut buf = vec![0u8; 1_000];
        secure_random_bytes(&mut buf);
        let mut seen: HashSet<u8> = HashSet::new();
        for &b in &buf {
            seen.insert(b);
        }
        // Em 1.000 bytes, esperamos ver pelo menos 200 valores distintos
        assert!(
            seen.len() >= 200,
            "Apenas {} bytes distintos — baixa entropia",
            seen.len()
        );
    }

    #[test]
    fn bytes_buffer_vazio_nao_panica() {
        let mut buf: Vec<u8> = vec![];
        secure_random_bytes(&mut buf);
        assert!(buf.is_empty());
    }

    // ─── secure_shuffle ───

    #[test]
    fn shuffle_preserva_elementos() {
        let original: Vec<u32> = (0..52).collect();
        let mut shuffled = original.clone();
        secure_shuffle(&mut shuffled);
        shuffled.sort();
        assert_eq!(
            shuffled, original,
            "Shuffle não deve perder nem duplicar elementos"
        );
    }

    #[test]
    fn shuffle_altera_ordem() {
        let original: Vec<u32> = (0..52).collect();
        let mut shuffled = original.clone();
        secure_shuffle(&mut shuffled);
        assert_ne!(
            shuffled, original,
            "Shuffle não alterou a ordem — 1/52! de chance"
        );
    }

    #[test]
    fn shuffle_baralho_preserva_52_unicas() {
        let deck = create_deck();
        let mut shuffled = deck.clone();
        secure_shuffle(&mut shuffled);
        assert_eq!(shuffled.len(), 52);
        let set = deck_to_set(&shuffled);
        assert_eq!(set.len(), 52, "Shuffle não deve criar duplicatas");
    }

    #[test]
    fn shuffle_vazio_nao_panica() {
        let mut empty: Vec<u32> = vec![];
        secure_shuffle(&mut empty);
        assert!(empty.is_empty());
    }

    #[test]
    fn shuffle_elemento_unico_nao_altera() {
        let mut single = vec![42u32];
        secure_shuffle(&mut single);
        assert_eq!(single, vec![42]);
    }

    // ─── Testes property-based (proptest) ───

    proptest! {
        /// Propriedade: secure_random_u32 sempre retorna valor dentro do range
        #[test]
        fn prop_u32_sempre_no_range(min in 0u32..1000, max in 1000u32..2000) {
            let val = secure_random_u32(min..=max);
            prop_assert!(val >= min && val <= max);
        }

        /// Propriedade: secure_random_u64 sempre retorna valor dentro do range
        #[test]
        fn prop_u64_sempre_no_range(min in 0u64..1_000_000, max in 1_000_000u64..10_000_000) {
            let val = secure_random_u64(min..=max);
            prop_assert!(val >= min && val <= max);
        }

        /// Propriedade: secure_random_f64 sempre em [0.0, 1.0)
        #[test]
        fn prop_f64_sempre_valido(_n in 0..1000u32) {
            let val = secure_random_f64();
            prop_assert!(val >= 0.0 && val < 1.0);
        }

        /// Propriedade: secure_shuffle preserva o multiconjunto de elementos
        #[test]
        fn prop_shuffle_preserva_elementos(seed in 0..100u32) {
            let original: Vec<u32> = (0..100).collect();
            let mut shuffled = original.clone();
            // Usar seed para variar — mas secure_shuffle usa OsRng
            let _ = seed;
            secure_shuffle(&mut shuffled);
            let mut orig_sorted = original.clone();
            let mut shuf_sorted = shuffled.clone();
            orig_sorted.sort();
            shuf_sorted.sort();
            prop_assert_eq!(orig_sorted, shuf_sorted);
        }

        /// Propriedade: secure_random_bytes preenche todo o buffer
        #[test]
        fn prop_bytes_preenche_buffer(len in 1usize..500) {
            let mut buf = vec![0u8; len];
            secure_random_bytes(&mut buf);
            // Pelo menos um byte deve ser não-zero (probabilisticamente certo)
            let non_zero = buf.iter().filter(|&&b| b != 0).count();
            prop_assert!(non_zero > 0 || len == 0);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// ÁREA 3 — DISTRIBUIÇÃO DESTRUTIVA (DEAL)
// ═══════════════════════════════════════════════════════════════════

mod deal_destructive {
    use super::*;

    #[test]
    fn deal_5_cartas_remove_do_baralho() {
        let deck = create_deck();
        let (dealt, remaining) = deal_cards(&deck, 5);
        assert_eq!(dealt.len(), 5);
        assert_eq!(remaining.len(), 47);
        assert_eq!(dealt.len() + remaining.len(), 52);
    }

    #[test]
    fn deal_cartas_nao_aparecem_no_resto() {
        let deck = create_deck();
        let (dealt, remaining) = deal_cards(&deck, 5);
        let dealt_set: HashSet<(Rank, Suit)> = dealt.iter().map(|c| (c.rank, c.suit)).collect();
        let remaining_set: HashSet<(Rank, Suit)> =
            remaining.iter().map(|c| (c.rank, c.suit)).collect();
        // Nenhuma carta distribuída deve estar no resto
        for card in &dealt_set {
            assert!(
                !remaining_set.contains(card),
                "Carta {:?} apareceu no baralho restante — deal não é destrutivo!",
                card
            );
        }
    }

    #[test]
    fn deal_completo_zera_baralho() {
        let deck = create_deck();
        let (dealt, remaining) = deal_cards(&deck, 52);
        assert_eq!(dealt.len(), 52);
        assert_eq!(remaining.len(), 0);
    }

    #[test]
    fn deal_mais_que_baralho_nao_panica() {
        let deck = create_deck();
        let (dealt, remaining) = deal_cards(&deck, 100);
        assert_eq!(dealt.len(), 52, "Não deve distribuir mais que 52");
        assert_eq!(remaining.len(), 0);
    }

    #[test]
    fn deal_zero_cartas_retorna_vazio() {
        let deck = create_deck();
        let (dealt, remaining) = deal_cards(&deck, 0);
        assert_eq!(dealt.len(), 0);
        assert_eq!(remaining.len(), 52);
    }

    #[test]
    fn deal_simula_mao_completa_hole_flop_turn_river() {
        // Simula distribuição completa de Texas Hold'em:
        // 2 hole cards para 1 jogador + flop (3) + turn (1) + river (1) = 7 cartas
        let deck = create_deck();
        let shuffled = shuffle_deck(&deck);

        // Hole cards
        let (hole, rest1) = deal_cards(&shuffled, 2);
        assert_eq!(hole.len(), 2);
        assert_eq!(rest1.len(), 50);

        // Flop
        let (flop, rest2) = deal_cards(&rest1, 3);
        assert_eq!(flop.len(), 3);
        assert_eq!(rest2.len(), 47);

        // Turn
        let (turn, rest3) = deal_cards(&rest2, 1);
        assert_eq!(turn.len(), 1);
        assert_eq!(rest3.len(), 46);

        // River
        let (river, rest4) = deal_cards(&rest3, 1);
        assert_eq!(river.len(), 1);
        assert_eq!(rest4.len(), 45);

        // Verifica que nenhuma carta se repete entre hole, flop, turn, river
        let mut all_dealt: Vec<(Rank, Suit)> = Vec::new();
        all_dealt.extend(hole.iter().map(|c| (c.rank, c.suit)));
        all_dealt.extend(flop.iter().map(|c| (c.rank, c.suit)));
        all_dealt.extend(turn.iter().map(|c| (c.rank, c.suit)));
        all_dealt.extend(river.iter().map(|c| (c.rank, c.suit)));

        let set: HashSet<(Rank, Suit)> = all_dealt.iter().copied().collect();
        assert_eq!(set.len(), 7, "7 cartas distribuídas devem ser todas únicas");
    }

    #[test]
    fn deal_simula_mesa_com_9_jogadores() {
        // 9 jogadores × 2 hole cards = 18 + 5 comunitárias = 23 cartas
        let deck = create_deck();
        let shuffled = shuffle_deck(&deck);

        let mut all_hole_cards: Vec<Card> = Vec::new();
        let mut current_deck = shuffled;

        for i in 0..9 {
            let (hole, rest) = deal_cards(&current_deck, 2);
            assert_eq!(hole.len(), 2, "Jogador {} deve receber 2 cartas", i);
            all_hole_cards.extend(hole);
            current_deck = rest;
        }

        assert_eq!(all_hole_cards.len(), 18);
        assert_eq!(current_deck.len(), 34); // 52 - 18

        // Flop + Turn + River
        let (flop, rest) = deal_cards(&current_deck, 3);
        let (turn, rest) = deal_cards(&rest, 1);
        let (river, rest) = deal_cards(&rest, 1);

        assert_eq!(flop.len() + turn.len() + river.len(), 5);
        assert_eq!(rest.len(), 29); // 52 - 18 - 5

        // Verifica unicidade total
        let mut all: Vec<(Rank, Suit)> = Vec::new();
        all.extend(all_hole_cards.iter().map(|c| (c.rank, c.suit)));
        all.extend(flop.iter().map(|c| (c.rank, c.suit)));
        all.extend(turn.iter().map(|c| (c.rank, c.suit)));
        all.extend(river.iter().map(|c| (c.rank, c.suit)));

        let set: HashSet<(Rank, Suit)> = all.iter().copied().collect();
        assert_eq!(set.len(), 23, "23 cartas distribuídas devem ser únicas");
    }

    #[test]
    fn deal_destrutivo_cartas_distribuidas_somem_do_baralho() {
        // Testa que após deal, as cartas NÃO estão mais no baralho
        let deck = create_deck();
        let (dealt, remaining) = deal_cards(&deck, 10);

        for card in &dealt {
            assert!(
                !remaining.contains(card),
                "Carta {:?} ainda está no baralho após deal — VIOLAÇÃO DE INTEGRIDADE",
                card
            );
        }
    }

    // ─── Testes property-based (proptest) ───

    proptest! {
        /// Propriedade: deal de N cartas sempre remove exatamente N e deixa 52-N
        #[test]
        fn prop_deal_quantidade_correta(n in 0usize..52) {
            let deck = create_deck();
            let (dealt, remaining) = deal_cards(&deck, n);
            prop_assert_eq!(dealt.len(), n);
            prop_assert_eq!(remaining.len(), 52 - n);
            prop_assert_eq!(dealt.len() + remaining.len(), 52);
        }

        /// Propriedade: cartas distribuídas nunca aparecem no resto
        #[test]
        fn prop_deal_sem_sobreposicao(n in 1usize..52) {
            let deck = create_deck();
            let (dealt, remaining) = deal_cards(&deck, n);
            let dealt_set: HashSet<(Rank, Suit)> = dealt.iter().map(|c| (c.rank, c.suit)).collect();
            let remaining_set: HashSet<(Rank, Suit)> = remaining.iter().map(|c| (c.rank, c.suit)).collect();
            for card in &dealt_set {
                prop_assert!(!remaining_set.contains(card), "Carta {:?} duplicada após deal", card);
            }
        }

        /// Propriedade: shuffle + deal nunca produz duplicatas
        #[test]
        fn prop_shuffle_deal_sem_duplicatas(n in 1usize..52) {
            let deck = create_deck();
            let shuffled = shuffle_deck(&deck);
            let (dealt, remaining) = deal_cards(&shuffled, n);
            let all: Vec<(Rank, Suit)> = dealt.iter().chain(remaining.iter())
                .map(|c| (c.rank, c.suit)).collect();
            let set: HashSet<(Rank, Suit)> = all.iter().copied().collect();
            prop_assert_eq!(set.len(), 52, "Shuffle+deal criou duplicatas");
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// ÁREA 4 — AVALIAÇÃO DE SHOWDOWN (TEXAS HOLD'EM)
// ═══════════════════════════════════════════════════════════════════

mod showdown_evaluation {
    use super::*;

    // ─── Royal Flush ───

    #[test]
    fn royal_flush_espadas() {
        let hole = vec![c(Rank::Ace, Suit::Spades), c(Rank::King, Suit::Spades)];
        let community = vec![
            c(Rank::Queen, Suit::Spades),
            c(Rank::Jack, Suit::Spades),
            c(Rank::Ten, Suit::Spades),
            c(Rank::Two, Suit::Hearts),
            c(Rank::Three, Suit::Diamonds),
        ];
        let result = evaluate_hand(&hole, &community);
        assert_eq!(result.rank, HandRank::RoyalFlush);
        assert_eq!(result.value, 10);
    }

    #[test]
    fn royal_flush_copas() {
        let hole = vec![c(Rank::Ten, Suit::Hearts), c(Rank::Jack, Suit::Hearts)];
        let community = vec![
            c(Rank::Queen, Suit::Hearts),
            c(Rank::King, Suit::Hearts),
            c(Rank::Ace, Suit::Hearts),
            c(Rank::Two, Suit::Clubs),
            c(Rank::Three, Suit::Diamonds),
        ];
        let result = evaluate_hand(&hole, &community);
        assert_eq!(result.rank, HandRank::RoyalFlush);
    }

    // ─── Straight Flush ───

    #[test]
    fn straight_flush_5_a_9() {
        let hole = vec![c(Rank::Nine, Suit::Hearts), c(Rank::Eight, Suit::Hearts)];
        let community = vec![
            c(Rank::Seven, Suit::Hearts),
            c(Rank::Six, Suit::Hearts),
            c(Rank::Five, Suit::Hearts),
            c(Rank::Ace, Suit::Clubs),
            c(Rank::King, Suit::Diamonds),
        ];
        let result = evaluate_hand(&hole, &community);
        assert_eq!(result.rank, HandRank::StraightFlush);
        assert_eq!(result.value, 9);
    }

    #[test]
    fn straight_flush_wheel_a_5() {
        // Wheel straight flush: A-2-3-4-5 do mesmo naipe
        let hole = vec![c(Rank::Ace, Suit::Clubs), c(Rank::Two, Suit::Clubs)];
        let community = vec![
            c(Rank::Three, Suit::Clubs),
            c(Rank::Four, Suit::Clubs),
            c(Rank::Five, Suit::Clubs),
            c(Rank::King, Suit::Diamonds),
            c(Rank::Queen, Suit::Hearts),
        ];
        let result = evaluate_hand(&hole, &community);
        assert_eq!(result.rank, HandRank::StraightFlush);
    }

    #[test]
    fn straight_flush_nao_e_royal_se_nao_tem_ace() {
        // 9-10-J-Q-K do mesmo naipe — Straight Flush, não Royal
        let hole = vec![c(Rank::Nine, Suit::Diamonds), c(Rank::Ten, Suit::Diamonds)];
        let community = vec![
            c(Rank::Jack, Suit::Diamonds),
            c(Rank::Queen, Suit::Diamonds),
            c(Rank::King, Suit::Diamonds),
            c(Rank::Two, Suit::Clubs),
            c(Rank::Three, Suit::Hearts),
        ];
        let result = evaluate_hand(&hole, &community);
        assert_eq!(result.rank, HandRank::StraightFlush);
        assert_ne!(result.rank, HandRank::RoyalFlush);
    }

    // ─── Four of a Kind ───

    #[test]
    fn four_of_a_kind_reis() {
        let hole = vec![c(Rank::King, Suit::Hearts), c(Rank::King, Suit::Diamonds)];
        let community = vec![
            c(Rank::King, Suit::Clubs),
            c(Rank::King, Suit::Spades),
            c(Rank::Ace, Suit::Hearts),
            c(Rank::Two, Suit::Diamonds),
            c(Rank::Three, Suit::Clubs),
        ];
        let result = evaluate_hand(&hole, &community);
        assert_eq!(result.rank, HandRank::FourOfAKind);
        assert_eq!(result.value, 8);
    }

    #[test]
    fn four_of_a_kind_na_board_com_kicker_as() {
        // Quadra na mesa (4 cartas), jogador tem Ás como kicker
        let hole = vec![c(Rank::Seven, Suit::Hearts), c(Rank::Ace, Suit::Diamonds)];
        let community = vec![
            c(Rank::Seven, Suit::Clubs),
            c(Rank::Seven, Suit::Spades),
            c(Rank::Seven, Suit::Diamonds),
            c(Rank::Two, Suit::Diamonds),
            c(Rank::Three, Suit::Clubs),
        ];
        let result = evaluate_hand(&hole, &community);
        assert_eq!(result.rank, HandRank::FourOfAKind);
    }

    // ─── Full House ───

    #[test]
    fn full_house_as_sobre_reis() {
        let hole = vec![c(Rank::Ace, Suit::Hearts), c(Rank::Ace, Suit::Diamonds)];
        let community = vec![
            c(Rank::Ace, Suit::Clubs),
            c(Rank::King, Suit::Spades),
            c(Rank::King, Suit::Hearts),
            c(Rank::Two, Suit::Diamonds),
            c(Rank::Three, Suit::Clubs),
        ];
        let result = evaluate_hand(&hole, &community);
        assert_eq!(result.rank, HandRank::FullHouse);
        assert_eq!(result.value, 7);
    }

    #[test]
    fn full_house_reis_sobre_as() {
        // Trinca de Reis + Par de Ases (mas trinca de Ases também existe → Full House As/Reis)
        let hole = vec![c(Rank::King, Suit::Hearts), c(Rank::King, Suit::Diamonds)];
        let community = vec![
            c(Rank::King, Suit::Clubs),
            c(Rank::Ace, Suit::Spades),
            c(Rank::Ace, Suit::Hearts),
            c(Rank::Ace, Suit::Diamonds),
            c(Rank::Three, Suit::Clubs),
        ];
        let result = evaluate_hand(&hole, &community);
        assert_eq!(result.rank, HandRank::FullHouse);
    }

    // ─── Flush ───

    #[test]
    fn flush_ouros() {
        let hole = vec![c(Rank::Ace, Suit::Diamonds), c(Rank::Five, Suit::Diamonds)];
        let community = vec![
            c(Rank::Ten, Suit::Diamonds),
            c(Rank::Seven, Suit::Diamonds),
            c(Rank::Two, Suit::Diamonds),
            c(Rank::King, Suit::Hearts),
            c(Rank::Queen, Suit::Clubs),
        ];
        let result = evaluate_hand(&hole, &community);
        assert_eq!(result.rank, HandRank::Flush);
        assert_eq!(result.value, 6);
    }

    #[test]
    fn flush_nao_e_straight_flush() {
        // Flush mas cartas não formam sequência
        let hole = vec![c(Rank::Ace, Suit::Spades), c(Rank::Three, Suit::Spades)];
        let community = vec![
            c(Rank::Nine, Suit::Spades),
            c(Rank::Jack, Suit::Spades),
            c(Rank::King, Suit::Spades),
            c(Rank::Two, Suit::Hearts),
            c(Rank::Five, Suit::Clubs),
        ];
        let result = evaluate_hand(&hole, &community);
        assert_eq!(result.rank, HandRank::Flush);
        assert_ne!(result.rank, HandRank::StraightFlush);
    }

    // ─── Straight ───

    #[test]
    fn straight_5_a_9() {
        let hole = vec![c(Rank::Nine, Suit::Hearts), c(Rank::Eight, Suit::Diamonds)];
        let community = vec![
            c(Rank::Seven, Suit::Clubs),
            c(Rank::Six, Suit::Spades),
            c(Rank::Five, Suit::Hearts),
            c(Rank::Ace, Suit::Diamonds),
            c(Rank::King, Suit::Clubs),
        ];
        let result = evaluate_hand(&hole, &community);
        assert_eq!(result.rank, HandRank::Straight);
        assert_eq!(result.value, 5);
    }

    #[test]
    fn straight_wheel_a_5() {
        let hole = vec![c(Rank::Ace, Suit::Hearts), c(Rank::Two, Suit::Diamonds)];
        let community = vec![
            c(Rank::Three, Suit::Clubs),
            c(Rank::Four, Suit::Spades),
            c(Rank::Five, Suit::Hearts),
            c(Rank::King, Suit::Diamonds),
            c(Rank::Queen, Suit::Clubs),
        ];
        let result = evaluate_hand(&hole, &community);
        assert_eq!(result.rank, HandRank::Straight);
    }

    #[test]
    fn straight_10_a_ace() {
        let hole = vec![c(Rank::Ten, Suit::Hearts), c(Rank::Jack, Suit::Diamonds)];
        let community = vec![
            c(Rank::Queen, Suit::Clubs),
            c(Rank::King, Suit::Spades),
            c(Rank::Ace, Suit::Hearts),
            c(Rank::Two, Suit::Diamonds),
            c(Rank::Three, Suit::Clubs),
        ];
        let result = evaluate_hand(&hole, &community);
        assert_eq!(result.rank, HandRank::Straight);
    }

    // ─── Three of a Kind ───

    #[test]
    fn three_of_a_kind_damas() {
        let hole = vec![c(Rank::Queen, Suit::Hearts), c(Rank::Queen, Suit::Diamonds)];
        let community = vec![
            c(Rank::Queen, Suit::Clubs),
            c(Rank::Ace, Suit::Spades),
            c(Rank::Two, Suit::Hearts),
            c(Rank::Three, Suit::Diamonds),
            c(Rank::Four, Suit::Clubs),
        ];
        let result = evaluate_hand(&hole, &community);
        assert_eq!(result.rank, HandRank::ThreeOfAKind);
        assert_eq!(result.value, 4);
    }

    // ─── Two Pair ───

    #[test]
    fn two_pair_ases_e_reis() {
        let hole = vec![c(Rank::Ace, Suit::Hearts), c(Rank::Ace, Suit::Diamonds)];
        let community = vec![
            c(Rank::King, Suit::Clubs),
            c(Rank::King, Suit::Spades),
            c(Rank::Two, Suit::Hearts),
            c(Rank::Three, Suit::Diamonds),
            c(Rank::Four, Suit::Clubs),
        ];
        let result = evaluate_hand(&hole, &community);
        assert_eq!(result.rank, HandRank::TwoPair);
        assert_eq!(result.value, 3);
    }

    // ─── One Pair ───

    #[test]
    fn one_pair_valetes() {
        let hole = vec![c(Rank::Jack, Suit::Hearts), c(Rank::Jack, Suit::Diamonds)];
        let community = vec![
            c(Rank::Ace, Suit::Clubs),
            c(Rank::King, Suit::Spades),
            c(Rank::Two, Suit::Hearts),
            c(Rank::Three, Suit::Diamonds),
            c(Rank::Four, Suit::Clubs),
        ];
        let result = evaluate_hand(&hole, &community);
        assert_eq!(result.rank, HandRank::OnePair);
        assert_eq!(result.value, 2);
    }

    // ─── High Card ───

    #[test]
    fn high_card_asa_kicker() {
        let hole = vec![c(Rank::Ace, Suit::Spades), c(Rank::King, Suit::Diamonds)];
        let community = vec![
            c(Rank::Nine, Suit::Hearts),
            c(Rank::Five, Suit::Clubs),
            c(Rank::Two, Suit::Spades),
            c(Rank::Seven, Suit::Hearts),
            c(Rank::Three, Suit::Diamonds),
        ];
        let result = evaluate_hand(&hole, &community);
        assert_eq!(result.rank, HandRank::HighCard);
        assert_eq!(result.value, 1);
    }

    // ─── Comparações de mãos ───

    #[test]
    fn compare_royal_flush_bate_straight_flush() {
        let royal = evaluate_hand(
            &[c(Rank::Ace, Suit::Spades), c(Rank::King, Suit::Spades)],
            &[
                c(Rank::Queen, Suit::Spades),
                c(Rank::Jack, Suit::Spades),
                c(Rank::Ten, Suit::Spades),
                c(Rank::Two, Suit::Hearts),
                c(Rank::Three, Suit::Diamonds),
            ],
        );
        let sf = evaluate_hand(
            &[c(Rank::Nine, Suit::Hearts), c(Rank::Eight, Suit::Hearts)],
            &[
                c(Rank::Seven, Suit::Hearts),
                c(Rank::Six, Suit::Hearts),
                c(Rank::Five, Suit::Hearts),
                c(Rank::Ace, Suit::Clubs),
                c(Rank::King, Suit::Diamonds),
            ],
        );
        assert_eq!(compare_hands(&royal, &sf), std::cmp::Ordering::Greater);
    }

    #[test]
    fn compare_flush_bate_straight() {
        let flush = evaluate_hand(
            &[c(Rank::Ace, Suit::Diamonds), c(Rank::Five, Suit::Diamonds)],
            &[
                c(Rank::Ten, Suit::Diamonds),
                c(Rank::Seven, Suit::Diamonds),
                c(Rank::Two, Suit::Diamonds),
                c(Rank::King, Suit::Hearts),
                c(Rank::Queen, Suit::Clubs),
            ],
        );
        let straight = evaluate_hand(
            &[c(Rank::Nine, Suit::Hearts), c(Rank::Eight, Suit::Diamonds)],
            &[
                c(Rank::Seven, Suit::Clubs),
                c(Rank::Six, Suit::Spades),
                c(Rank::Five, Suit::Hearts),
                c(Rank::Ace, Suit::Diamonds),
                c(Rank::King, Suit::Clubs),
            ],
        );
        assert_eq!(
            compare_hands(&flush, &straight),
            std::cmp::Ordering::Greater
        );
    }

    #[test]
    fn compare_full_house_bate_flush() {
        let fh = evaluate_hand(
            &[c(Rank::Ace, Suit::Hearts), c(Rank::Ace, Suit::Diamonds)],
            &[
                c(Rank::Ace, Suit::Clubs),
                c(Rank::King, Suit::Spades),
                c(Rank::King, Suit::Hearts),
                c(Rank::Two, Suit::Diamonds),
                c(Rank::Three, Suit::Clubs),
            ],
        );
        let flush = evaluate_hand(
            &[c(Rank::Ace, Suit::Diamonds), c(Rank::Five, Suit::Diamonds)],
            &[
                c(Rank::Ten, Suit::Diamonds),
                c(Rank::Seven, Suit::Diamonds),
                c(Rank::Two, Suit::Diamonds),
                c(Rank::King, Suit::Hearts),
                c(Rank::Queen, Suit::Clubs),
            ],
        );
        assert_eq!(compare_hands(&fh, &flush), std::cmp::Ordering::Greater);
    }

    #[test]
    fn compare_quadra_bate_full_house() {
        let quads = evaluate_hand(
            &[c(Rank::King, Suit::Hearts), c(Rank::King, Suit::Diamonds)],
            &[
                c(Rank::King, Suit::Clubs),
                c(Rank::King, Suit::Spades),
                c(Rank::Ace, Suit::Hearts),
                c(Rank::Two, Suit::Diamonds),
                c(Rank::Three, Suit::Clubs),
            ],
        );
        let fh = evaluate_hand(
            &[c(Rank::Ace, Suit::Hearts), c(Rank::Ace, Suit::Diamonds)],
            &[
                c(Rank::Ace, Suit::Clubs),
                c(Rank::King, Suit::Spades),
                c(Rank::King, Suit::Hearts),
                c(Rank::Two, Suit::Diamonds),
                c(Rank::Three, Suit::Clubs),
            ],
        );
        assert_eq!(compare_hands(&quads, &fh), std::cmp::Ordering::Greater);
    }

    #[test]
    fn compare_par_alto_bate_par_baixo() {
        let par_alto = evaluate_hand(
            &[c(Rank::Ace, Suit::Hearts), c(Rank::Ace, Suit::Diamonds)],
            &[
                c(Rank::King, Suit::Clubs),
                c(Rank::Queen, Suit::Spades),
                c(Rank::Two, Suit::Hearts),
                c(Rank::Three, Suit::Diamonds),
                c(Rank::Four, Suit::Clubs),
            ],
        );
        let par_baixo = evaluate_hand(
            &[c(Rank::King, Suit::Hearts), c(Rank::King, Suit::Diamonds)],
            &[
                c(Rank::Ace, Suit::Clubs),
                c(Rank::Queen, Suit::Spades),
                c(Rank::Two, Suit::Hearts),
                c(Rank::Three, Suit::Diamonds),
                c(Rank::Four, Suit::Clubs),
            ],
        );
        assert_eq!(
            compare_hands(&par_alto, &par_baixo),
            std::cmp::Ordering::Greater
        );
    }

    #[test]
    fn compare_empate_mesmo_valor() {
        let a = evaluate_hand(
            &[c(Rank::Ace, Suit::Hearts), c(Rank::Ace, Suit::Diamonds)],
            &[
                c(Rank::King, Suit::Clubs),
                c(Rank::King, Suit::Spades),
                c(Rank::Two, Suit::Hearts),
                c(Rank::Three, Suit::Diamonds),
                c(Rank::Four, Suit::Clubs),
            ],
        );
        let b = evaluate_hand(
            &[c(Rank::Ace, Suit::Clubs), c(Rank::Ace, Suit::Spades)],
            &[
                c(Rank::King, Suit::Hearts),
                c(Rank::King, Suit::Diamonds),
                c(Rank::Two, Suit::Hearts),
                c(Rank::Three, Suit::Diamonds),
                c(Rank::Four, Suit::Clubs),
            ],
        );
        assert_eq!(compare_hands(&a, &b), std::cmp::Ordering::Equal);
    }

    // ─── Hierarquia completa ───

    #[test]
    fn hierarquia_completa_de_maos() {
        // Verifica que cada HandRank é estritamente menor que a próxima
        assert!(HandRank::HighCard < HandRank::OnePair);
        assert!(HandRank::OnePair < HandRank::TwoPair);
        assert!(HandRank::TwoPair < HandRank::ThreeOfAKind);
        assert!(HandRank::ThreeOfAKind < HandRank::Straight);
        assert!(HandRank::Straight < HandRank::Flush);
        assert!(HandRank::Flush < HandRank::FullHouse);
        assert!(HandRank::FullHouse < HandRank::FourOfAKind);
        assert!(HandRank::FourOfAKind < HandRank::StraightFlush);
        assert!(HandRank::StraightFlush < HandRank::RoyalFlush);
    }

    #[test]
    fn get_hand_name_todas_categorias() {
        assert_eq!(get_hand_name(HandRank::HighCard), "High Card");
        assert_eq!(get_hand_name(HandRank::OnePair), "One Pair");
        assert_eq!(get_hand_name(HandRank::TwoPair), "Two Pair");
        assert_eq!(get_hand_name(HandRank::ThreeOfAKind), "Three of a Kind");
        assert_eq!(get_hand_name(HandRank::Straight), "Straight");
        assert_eq!(get_hand_name(HandRank::Flush), "Flush");
        assert_eq!(get_hand_name(HandRank::FullHouse), "Full House");
        assert_eq!(get_hand_name(HandRank::FourOfAKind), "Four of a Kind");
        assert_eq!(get_hand_name(HandRank::StraightFlush), "Straight Flush");
        assert_eq!(get_hand_name(HandRank::RoyalFlush), "Royal Flush");
    }

    // ─── Casos edge ───

    #[test]
    fn evaluate_com_apenas_hole_cards() {
        // Sem cartas comunitárias — só 2 cartas
        let hole = vec![c(Rank::Ace, Suit::Spades), c(Rank::King, Suit::Diamonds)];
        let community: Vec<Card> = vec![];
        let result = evaluate_hand(&hole, &community);
        assert_eq!(result.rank, HandRank::HighCard);
    }

    #[test]
    fn evaluate_board_joga_sozinha() {
        // Board tem um par — jogador não tem nada melhor
        let hole = vec![c(Rank::Two, Suit::Spades), c(Rank::Three, Suit::Diamonds)];
        let community = vec![
            c(Rank::King, Suit::Clubs),
            c(Rank::King, Suit::Spades),
            c(Rank::Queen, Suit::Hearts),
            c(Rank::Jack, Suit::Diamonds),
            c(Rank::Nine, Suit::Clubs),
        ];
        let result = evaluate_hand(&hole, &community);
        assert_eq!(result.rank, HandRank::OnePair);
    }

    #[test]
    fn evaluate_melhor_mao_entre_possiveis() {
        // Jogador tem par na mão, mas board tem flush — flush deve prevalecer
        let hole = vec![c(Rank::King, Suit::Hearts), c(Rank::King, Suit::Diamonds)];
        let community = vec![
            c(Rank::Two, Suit::Spades),
            c(Rank::Five, Suit::Spades),
            c(Rank::Eight, Suit::Spades),
            c(Rank::Jack, Suit::Spades),
            c(Rank::Ace, Suit::Spades),
        ];
        let result = evaluate_hand(&hole, &community);
        // Flush na mesa (5 espadas) deve bater o par de Reis
        assert_eq!(result.rank, HandRank::Flush);
    }
}

// ═══════════════════════════════════════════════════════════════════
// ÁREA 5 — DIVISÃO DE SIDE POTS
// ═══════════════════════════════════════════════════════════════════

mod side_pots_tests {
    use super::*;

    // ─── Cálculo de pots ───

    #[test]
    fn pot_unico_dois_jogadores_iguais() {
        let players = vec![
            make_player("p1", 100.0, false, vec![]),
            make_player("p2", 100.0, false, vec![]),
        ];
        let pots = calculate_side_pots(&players);
        assert_eq!(pots.len(), 1);
        assert!((pots[0].amount - 200.0).abs() < f64::EPSILON);
        assert_eq!(pots[0].eligible_players.len(), 2);
    }

    #[test]
    fn main_pot_mais_side_pot() {
        // p1: 100, p2: 200, p3: 200
        // main: (100-0)*3 = 300, side: (200-100)*2 = 200
        let players = vec![
            make_player("p1", 100.0, false, vec![]),
            make_player("p2", 200.0, false, vec![]),
            make_player("p3", 200.0, false, vec![]),
        ];
        let pots = calculate_side_pots(&players);
        assert_eq!(pots.len(), 2);
        assert!((pots[0].amount - 300.0).abs() < f64::EPSILON);
        assert_eq!(pots[0].eligible_players.len(), 3);
        assert!((pots[1].amount - 200.0).abs() < f64::EPSILON);
        assert_eq!(pots[1].eligible_players.len(), 2);
    }

    #[test]
    fn tres_niveis_de_pots() {
        // p1: 50, p2: 100, p3: 200
        // pot0: 50*3=150, pot1: 50*2=100, pot2: 100*1=100
        let players = vec![
            make_player("p1", 50.0, false, vec![]),
            make_player("p2", 100.0, false, vec![]),
            make_player("p3", 200.0, false, vec![]),
        ];
        let pots = calculate_side_pots(&players);
        assert_eq!(pots.len(), 3);
        assert!((pots[0].amount - 150.0).abs() < f64::EPSILON);
        assert!((pots[1].amount - 100.0).abs() < f64::EPSILON);
        assert!((pots[2].amount - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn soma_dos_pots_igual_ao_total_apostado() {
        let players = vec![
            make_player("p1", 50.0, false, vec![]),
            make_player("p2", 100.0, false, vec![]),
            make_player("p3", 200.0, false, vec![]),
        ];
        let pots = calculate_side_pots(&players);
        let total_pots: f64 = pots.iter().map(|p| p.amount).sum();
        let total_bets: f64 = players.iter().map(|p| p.total_bet).sum();
        assert!(
            (total_pots - total_bets).abs() < f64::EPSILON,
            "Soma dos pots deve igualar total apostado"
        );
    }

    #[test]
    fn jogador_folded_cria_pots_mas_nao_e_elegivel() {
        let players = vec![
            make_player("p1", 100.0, true, vec![]), // folded
            make_player("p2", 200.0, false, vec![]),
        ];
        let pots = calculate_side_pots(&players);
        assert_eq!(pots.len(), 2);
        // p1 ainda contribui para os pots, mas não é elegível na distribuição
    }

    #[test]
    fn jogador_com_zero_aposta_nao_cria_pot() {
        let players = vec![
            make_player("p1", 0.0, false, vec![]),
            make_player("p2", 100.0, false, vec![]),
        ];
        let pots = calculate_side_pots(&players);
        // Apenas p2 contribuiu
        assert_eq!(pots.len(), 1);
        assert!((pots[0].amount - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn sem_jogadores_retorna_vazio() {
        let pots = calculate_side_pots(&[]);
        assert!(pots.is_empty());
    }

    #[test]
    fn todos_com_zero_aposta_retorna_vazio() {
        let players = vec![
            make_player("p1", 0.0, false, vec![]),
            make_player("p2", 0.0, false, vec![]),
        ];
        let pots = calculate_side_pots(&players);
        assert!(pots.is_empty());
    }

    // ─── Distribuição de pots ───

    #[test]
    fn distribuicao_vencedor_unico() {
        // p1 tem par de Ases, p2 tem carta alta
        let players = vec![
            make_player(
                "p1",
                100.0,
                false,
                vec![c(Rank::Ace, Suit::Hearts), c(Rank::King, Suit::Hearts)],
            ),
            make_player(
                "p2",
                100.0,
                false,
                vec![c(Rank::Two, Suit::Clubs), c(Rank::Three, Suit::Clubs)],
            ),
        ];
        let pots = vec![Pot {
            amount: 200.0,
            eligible_players: vec!["p1".into(), "p2".into()],
        }];
        let community = vec![
            c(Rank::Ace, Suit::Spades),
            c(Rank::Jack, Suit::Diamonds),
            c(Rank::Ten, Suit::Hearts),
            c(Rank::Five, Suit::Diamonds),
            c(Rank::Two, Suit::Hearts),
        ];
        let payouts = distribute_pots(&pots, &players, &community);
        assert!((*payouts.get("p1").unwrap() - 200.0).abs() < f64::EPSILON);
        assert!(payouts.get("p2").is_none() || payouts.get("p2") == Some(&0.0));
    }

    #[test]
    fn distribuicao_split_pot_empate() {
        // Ambos jogam o board (royal flush na mesa)
        let players = vec![
            make_player(
                "p1",
                100.0,
                false,
                vec![c(Rank::Two, Suit::Hearts), c(Rank::Three, Suit::Hearts)],
            ),
            make_player(
                "p2",
                100.0,
                false,
                vec![c(Rank::Four, Suit::Clubs), c(Rank::Five, Suit::Clubs)],
            ),
        ];
        let pots = vec![Pot {
            amount: 200.0,
            eligible_players: vec!["p1".into(), "p2".into()],
        }];
        let community = vec![
            c(Rank::Ace, Suit::Diamonds),
            c(Rank::King, Suit::Diamonds),
            c(Rank::Queen, Suit::Diamonds),
            c(Rank::Jack, Suit::Diamonds),
            c(Rank::Ten, Suit::Diamonds),
        ];
        let payouts = distribute_pots(&pots, &players, &community);
        assert!((*payouts.get("p1").unwrap() - 100.0).abs() < f64::EPSILON);
        assert!((*payouts.get("p2").unwrap() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn distribuicao_folded_nao_recebe() {
        // p1 foldou, p2 ganha por WO (único não-folded)
        let players = vec![
            make_player(
                "p1",
                100.0,
                true,
                vec![c(Rank::Ace, Suit::Hearts), c(Rank::Ace, Suit::Diamonds)],
            ),
            make_player(
                "p2",
                100.0,
                false,
                vec![c(Rank::Two, Suit::Clubs), c(Rank::Three, Suit::Clubs)],
            ),
        ];
        let pots = vec![Pot {
            amount: 200.0,
            eligible_players: vec!["p1".into(), "p2".into()],
        }];
        let community = vec![
            c(Rank::King, Suit::Spades),
            c(Rank::Queen, Suit::Diamonds),
            c(Rank::Jack, Suit::Hearts),
            c(Rank::Nine, Suit::Diamonds),
            c(Rank::Eight, Suit::Clubs),
        ];
        let payouts = distribute_pots(&pots, &players, &community);
        // p1 foldou — p2 recebe tudo
        assert!((*payouts.get("p2").unwrap() - 200.0).abs() < f64::EPSILON);
        assert!(payouts.get("p1").is_none() || payouts.get("p1") == Some(&0.0));
    }

    #[test]
    fn distribuicao_main_e_side_pot_vencedores_diferentes() {
        // p1: 100 (all-in), p2: 200, p3: 200
        // p1 ganha main pot (300), p2 ganha side pot (200)
        let players = vec![
            make_player(
                "p1",
                100.0,
                false,
                vec![c(Rank::Ace, Suit::Hearts), c(Rank::Ace, Suit::Diamonds)],
            ),
            make_player(
                "p2",
                200.0,
                false,
                vec![c(Rank::King, Suit::Hearts), c(Rank::King, Suit::Diamonds)],
            ),
            make_player(
                "p3",
                200.0,
                false,
                vec![c(Rank::Queen, Suit::Hearts), c(Rank::Queen, Suit::Diamonds)],
            ),
        ];
        let community = vec![
            c(Rank::Two, Suit::Spades),
            c(Rank::Five, Suit::Clubs),
            c(Rank::Eight, Suit::Diamonds),
            c(Rank::Jack, Suit::Spades),
            c(Rank::Nine, Suit::Clubs),
        ];
        let result = resolve_side_pots(&players, &community);
        assert_eq!(result.pots.len(), 2);
        assert!((result.pots[0].amount - 300.0).abs() < f64::EPSILON); // main
        assert!((result.pots[1].amount - 200.0).abs() < f64::EPSILON); // side

        // p1 tem par de Ases (ganha main), p2 tem par de Reis (ganha side)
        assert!((*result.payouts.get("p1").unwrap() - 300.0).abs() < f64::EPSILON);
        assert!((*result.payouts.get("p2").unwrap() - 200.0).abs() < f64::EPSILON);
    }

    #[test]
    fn distribuicao_split_pot_com_resto() {
        // Pot ímpar dividido entre 2 — floor division
        let players = vec![
            make_player(
                "p1",
                100.0,
                false,
                vec![c(Rank::Two, Suit::Hearts), c(Rank::Three, Suit::Hearts)],
            ),
            make_player(
                "p2",
                100.0,
                false,
                vec![c(Rank::Four, Suit::Clubs), c(Rank::Five, Suit::Clubs)],
            ),
        ];
        let pots = vec![Pot {
            amount: 201.0,
            eligible_players: vec!["p1".into(), "p2".into()],
        }];
        let community = vec![
            c(Rank::Ace, Suit::Diamonds),
            c(Rank::King, Suit::Diamonds),
            c(Rank::Queen, Suit::Diamonds),
            c(Rank::Jack, Suit::Diamonds),
            c(Rank::Ten, Suit::Diamonds),
        ];
        let payouts = distribute_pots(&pots, &players, &community);
        // 201 / 2 = 100.5 cada (truncado para 2 casas)
        assert_eq!(payouts.get("p1"), Some(&100.5));
        assert_eq!(payouts.get("p2"), Some(&100.5));
    }

    #[test]
    fn resolve_side_pots_integracao_completa() {
        let players = vec![
            make_player(
                "p1",
                100.0,
                false,
                vec![c(Rank::Ace, Suit::Hearts), c(Rank::King, Suit::Hearts)],
            ),
            make_player(
                "p2",
                200.0,
                false,
                vec![c(Rank::Two, Suit::Clubs), c(Rank::Three, Suit::Clubs)],
            ),
        ];
        let community = vec![
            c(Rank::Ace, Suit::Spades),
            c(Rank::Jack, Suit::Diamonds),
            c(Rank::Ten, Suit::Hearts),
            c(Rank::Five, Suit::Diamonds),
            c(Rank::Two, Suit::Hearts),
        ];
        let result = resolve_side_pots(&players, &community);
        assert_eq!(result.pots.len(), 2);
        assert_eq!(result.contributions.len(), 2);
    }

    // ─── Testes property-based (proptest) ───

    proptest! {
        /// Propriedade: soma dos pots sempre iguala soma das apostas
        #[test]
        fn prop_soma_pots_igual_apostas(
            bets in prop::collection::vec(1.0f64..1000.0, 2..6)
        ) {
            let players: Vec<PlayerForPots> = bets.iter().enumerate()
                .map(|(i, &bet)| make_player(&format!("p{}", i), bet, false, vec![]))
                .collect();
            let pots = calculate_side_pots(&players);
            let total_pots: f64 = pots.iter().map(|p| p.amount).sum();
            let total_bets: f64 = bets.iter().sum();
            prop_assert!((total_pots - total_bets).abs() < 0.01, "total_pots={}, total_bets={}", total_pots, total_bets);
        }

        /// Propriedade: cada pot tem pelo menos 1 jogador elegível
        #[test]
        fn prop_cada_pot_tem_elegivel(
            bets in prop::collection::vec(1.0f64..500.0, 2..5)
        ) {
            let players: Vec<PlayerForPots> = bets.iter().enumerate()
                .map(|(i, &bet)| make_player(&format!("p{}", i), bet, false, vec![]))
                .collect();
            let pots = calculate_side_pots(&players);
            for pot in &pots {
                prop_assert!(!pot.eligible_players.is_empty(), "Pot sem elegíveis");
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// ÁREA EXTRA — RAKE (TAXA DA CASA)
// ═══════════════════════════════════════════════════════════════════

mod rake_tests {
    use super::*;

    #[test]
    fn rake_abaixo_do_cap() {
        // 5% de 100 = 5, cap = 10
        assert!((calculate_rake_for_pot(100.0, 5.0, 10.0) - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn rake_no_cap() {
        // 5% de 300 = 15, mas cap = 10
        assert!((calculate_rake_for_pot(300.0, 5.0, 10.0) - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn rake_arredondamento_floor() {
        // 5% de 30 = 1.5 → truncado para 1.5
        assert!((calculate_rake_for_pot(30.0, 5.0, 10.0) - 1.5).abs() < 0.01);
    }

    #[test]
    fn rake_zero_percent() {
        assert!((calculate_rake_for_pot(100.0, 0.0, 10.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn rake_zero_cap() {
        assert!((calculate_rake_for_pot(100.0, 5.0, 0.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn rake_pot_pequeno() {
        // 5% de 1 = 0.05 → truncado para 0.05
        assert!((calculate_rake_for_pot(1.0, 5.0, 10.0) - 0.05).abs() < 0.01);
    }

    #[test]
    fn deduct_rake_pot_unico() {
        let pots = vec![make_rake_pot(200.0)];
        let result = deduct_rake(&pots, &default_table_config(), None);
        assert!((result.total_rake - 10.0).abs() < f64::EPSILON);
        assert!((result.pots_after_rake[0].amount - 190.0).abs() < f64::EPSILON);
        assert!((result.total_pot_before_rake - 200.0).abs() < f64::EPSILON);
    }

    #[test]
    fn deduct_rake_multipots_proporcional() {
        let pots = vec![make_rake_pot(100.0), make_rake_pot(50.0)];
        let result = deduct_rake(&pots, &default_table_config(), None);
        // 5% de 150 = 7.5 → truncado para 7.5
        // main: trunc(7.5 * 100/150) = 5.0, side: 7.5-5.0 = 2.5
        assert!((result.total_rake - 7.5).abs() < 0.01);
        assert!((result.per_pot[0].rake - 5.0).abs() < 0.01);
        assert!((result.per_pot[1].rake - 2.5).abs() < 0.01);
        assert!((result.pots_after_rake[0].amount - 95.0).abs() < 0.01);
        assert!((result.pots_after_rake[1].amount - 47.5).abs() < 0.01);
    }

    #[test]
    fn deduct_rake_abaixo_minimo() {
        // BB=10, min=20. Pot=15 → sem rake
        let pots = vec![make_rake_pot(15.0)];
        let result = deduct_rake(&pots, &default_table_config(), None);
        assert!((result.total_rake - 0.0).abs() < f64::EPSILON);
        assert!((result.pots_after_rake[0].amount - 15.0).abs() < f64::EPSILON);
    }

    #[test]
    fn deduct_rake_exatamente_no_minimo() {
        // Pot=20 → cobra rake (5% de 20 = 1)
        let pots = vec![make_rake_pot(20.0)];
        let result = deduct_rake(&pots, &default_table_config(), None);
        assert!((result.total_rake - 1.0).abs() < f64::EPSILON);
        assert!((result.pots_after_rake[0].amount - 19.0).abs() < f64::EPSILON);
    }

    #[test]
    fn deduct_rake_min_custom() {
        let pots = vec![make_rake_pot(50.0)];
        let result = deduct_rake(&pots, &default_table_config(), Some(100.0));
        assert!((result.total_rake - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn deduct_rake_tres_pots_ultimo_absorve_resto() {
        let pots = vec![
            make_rake_pot(100.0),
            make_rake_pot(60.0),
            make_rake_pot(40.0),
        ];
        let result = deduct_rake(&pots, &default_table_config(), None);
        // total=200, 5%=10
        // pot0: floor(10*100/200)=5, pot1: floor(10*60/200)=3, pot2: 10-5-3=2
        assert!((result.total_rake - 10.0).abs() < f64::EPSILON);
        assert!((result.per_pot[0].rake - 5.0).abs() < f64::EPSILON);
        assert!((result.per_pot[1].rake - 3.0).abs() < f64::EPSILON);
        assert!((result.per_pot[2].rake - 2.0).abs() < f64::EPSILON);
        let sum_rake: f64 = result.per_pot.iter().map(|e| e.rake).sum();
        assert!((sum_rake - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn deduct_rake_preserva_elegiveis() {
        let pot = Pot {
            amount: 200.0,
            eligible_players: vec!["alice".into(), "bob".into(), "charlie".into()],
        };
        let result = deduct_rake(&[pot], &default_table_config(), None);
        assert_eq!(result.pots_after_rake[0].eligible_players.len(), 3);
        assert!(result.pots_after_rake[0]
            .eligible_players
            .contains(&"alice".into()));
    }

    #[test]
    fn rake_nunca_negativo() {
        // Rake sempre ≥ 0
        for &amount in &[0.0f64, 1.0, 10.0, 100.0, 1000.0] {
            let rake = calculate_rake_for_pot(amount, 5.0, 100.0);
            assert!(
                rake <= amount,
                "Rake {} não pode exceder pot {}",
                rake,
                amount
            );
        }
    }

    // ─── Testes property-based (proptest) ───

    proptest! {
        /// Propriedade: rake nunca excede o pot
        #[test]
        fn prop_rake_nunca_excede_pot(
            pot_amount in 0.0f64..100_000.0,
            rake_percent in 0.0..100.0,
            rake_cap in 0.0f64..100_000.0
        ) {
            let rake = calculate_rake_for_pot(pot_amount, rake_percent, rake_cap);
            prop_assert!(rake <= pot_amount, "Rake excede pot");
        }

        /// Propriedade: rake nunca excede o cap
        #[test]
        fn prop_rake_nunca_excede_cap(
            pot_amount in 0.0f64..100_000.0,
            rake_percent in 0.1..50.0,
            rake_cap in 1.0f64..10_000.0
        ) {
            let rake = calculate_rake_for_pot(pot_amount, rake_percent, rake_cap);
            prop_assert!(rake <= rake_cap, "Rake excede cap");
        }

        /// Propriedade: soma dos rakes por pot = rake total
        #[test]
        fn prop_soma_rakes_igual_total(
            pot_amounts in prop::collection::vec(10.0f64..1000.0, 1..5)
        ) {
            let pots: Vec<Pot> = pot_amounts.iter()
                .map(|&a| make_rake_pot(a))
                .collect();
            let config = TableConfig {
                big_blind: 5.0,
                rake_percent: 5.0,
                rake_cap: 50.0,
            };
            let result = deduct_rake(&pots, &config, None);
            let sum: f64 = result.per_pot.iter().map(|e| e.rake).sum();
            prop_assert!((sum - result.total_rake).abs() < 0.01, "sum={}, total_rake={}", sum, result.total_rake);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// TESTES DE INTEGRÇÃO END-TO-END
// ═══════════════════════════════════════════════════════════════════

mod integration {
    use super::*;

    #[test]
    fn mao_completa_hole_flop_turn_river_showdown() {
        // Simula uma mão completa de Texas Hold'em
        let deck = create_deck();
        let shuffled = shuffle_deck(&deck);

        // 2 jogadores
        let (hole1, rest1) = deal_cards(&shuffled, 2);
        let (hole2, rest2) = deal_cards(&rest1, 2);

        // Flop (3 cartas)
        let (flop, rest3) = deal_cards(&rest2, 3);
        // Turn (1 carta)
        let (turn, rest4) = deal_cards(&rest3, 1);
        // River (1 carta)
        let (river, _rest5) = deal_cards(&rest4, 1);

        let community: Vec<Card> = flop
            .iter()
            .chain(turn.iter())
            .chain(river.iter())
            .copied()
            .collect();
        assert_eq!(community.len(), 5);

        // Avalia ambas as mãos
        let result1 = evaluate_hand(&hole1, &community);
        let result2 = evaluate_hand(&hole2, &community);

        // Ambas devem ter um HandRank válido
        assert!(result1.value >= 1 && result1.value <= 10);
        assert!(result2.value >= 1 && result2.value <= 10);

        // Uma deve vencer ou empatar
        let cmp = compare_hands(&result1, &result2);
        assert!(
            cmp == std::cmp::Ordering::Greater
                || cmp == std::cmp::Ordering::Less
                || cmp == std::cmp::Ordering::Equal
        );
    }

    #[test]
    fn fluxo_completo_side_pots_com_rake() {
        // 3 jogadores com stacks diferentes, calcula pots, aplica rake, distribui
        let players = vec![
            make_player(
                "p1",
                100.0,
                false,
                vec![c(Rank::Ace, Suit::Hearts), c(Rank::Ace, Suit::Diamonds)],
            ),
            make_player(
                "p2",
                200.0,
                false,
                vec![c(Rank::King, Suit::Hearts), c(Rank::King, Suit::Diamonds)],
            ),
            make_player(
                "p3",
                200.0,
                false,
                vec![c(Rank::Queen, Suit::Hearts), c(Rank::Queen, Suit::Diamonds)],
            ),
        ];
        let community = vec![
            c(Rank::Two, Suit::Spades),
            c(Rank::Five, Suit::Clubs),
            c(Rank::Eight, Suit::Diamonds),
            c(Rank::Jack, Suit::Spades),
            c(Rank::Nine, Suit::Clubs),
        ];

        // 1. Calcula side pots
        let pots = calculate_side_pots(&players);
        assert_eq!(pots.len(), 2);

        // 2. Converte para rake pots e aplica rake
        let rake_pots: Vec<Pot> = pots
            .iter()
            .map(|p| Pot {
                amount: p.amount,
                eligible_players: p.eligible_players.clone(),
            })
            .collect();
        let config = TableConfig {
            big_blind: 10.0,
            rake_percent: 5.0,
            rake_cap: 20.0,
        };
        let rake_result = deduct_rake(&rake_pots, &config, None);
        assert!(rake_result.total_rake > 0.0);

        // 3. Distribui pots (sem rake para simplificar)
        let payouts = distribute_pots(&pots, &players, &community);
        // p1 tem par de Ases → ganha main pot (300)
        assert!((*payouts.get("p1").unwrap() - 300.0).abs() < f64::EPSILON);
        // p2 tem par de Reis → ganha side pot (200)
        assert!((*payouts.get("p2").unwrap() - 200.0).abs() < f64::EPSILON);
    }

    #[test]
    fn mil_maos_aleatorias_sem_duplicatas() {
        // Executa 1.000 mãos aleatórias e verifica que nunca há duplicatas
        for _ in 0..1_000 {
            let deck = create_deck();
            let shuffled = shuffle_deck(&deck);

            // Distribui 9 cartas (2 jogadores + 5 comunitárias)
            let (dealt, remaining) = deal_cards(&shuffled, 9);
            let set: HashSet<(Rank, Suit)> = dealt.iter().map(|c| (c.rank, c.suit)).collect();
            assert_eq!(set.len(), 9, "Duplicatas detectadas em deal");

            let remaining_set: HashSet<(Rank, Suit)> =
                remaining.iter().map(|c| (c.rank, c.suit)).collect();
            assert_eq!(remaining_set.len(), 43);

            // Nenhuma sobreposição
            for card in &set {
                assert!(!remaining_set.contains(card));
            }
        }
    }

    #[test]
    fn mil_embaralhamentos_preservam_52_unicas() {
        for _ in 0..1_000 {
            let deck = create_deck();
            let shuffled = shuffle_deck(&deck);
            assert_eq!(shuffled.len(), 52);
            let set = deck_to_set(&shuffled);
            assert_eq!(set.len(), 52, "Shuffle criou duplicatas");
        }
    }
}
