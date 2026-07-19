/// RNG Criptográfico — CSPRNG para o motor de poker
///
/// Este módulo fornece geração de números aleatórios criptograficamente segura
/// usando o gerador do sistema operacional (OsRng).
///
/// # Por que CSPRNG?
/// - Integridade do jogo: shuffles devem ser imprevisíveis e à prova de manipulação
/// - Conformidade regulatória: jurisdições de jogos online exigem RNG auditável
/// - Prevenção de trapaça: jogadores não podem prever cartas futuras
///
/// # Implementação
/// - **Windows**: `BCryptGenRandom` (via `OsRng`)
/// - **Linux/macOS**: `getrandom` syscall (/dev/urandom)
/// - **WASM**: `web_sys::Crypto.getRandomValues` (compatível com Dioxus)
///
/// # Uso
/// ```rust
/// use poker_engine::rng_crypto::{secure_shuffle, secure_random_u32};
///
/// let mut deck = vec![1, 2, 3, 4, 5];
/// secure_shuffle(&mut deck);
/// let random_card_index = secure_random_u32(0..=51);
/// ```
use rand::seq::SliceRandom;
use rand::RngCore;

// ─── CSPRNG Source ───

/// Retorna uma referência mutável ao gerador criptográfico do SO.
/// Em testes, pode ser substituído por um gerador determinístico via feature flag.
#[inline]
pub fn csprng() -> impl RngCore {
    rand::rngs::OsRng
}

// ─── Shuffle Seguro ───

/// Embaralha um slice mutável usando Fisher-Yates com CSPRNG.
///
/// Diferente de `rand::thread_rng()`, este usa `OsRng` que é alimentado
/// pela entropia do sistema operacional — adequado para jogos de azar.
///
/// # Exemplo
/// ```rust
/// use poker_engine::rng_crypto::secure_shuffle;
///
/// let mut cartas = vec![1, 2, 3, 4, 5];
/// secure_shuffle(&mut cartas);
/// // cartas agora estão em ordem aleatória criptograficamente segura
/// ```
pub fn secure_shuffle<T>(slice: &mut [T]) {
    slice.shuffle(&mut csprng());
}

// ─── Geração de Números Aleatórios ───

/// Gera um u32 aleatório criptograficamente seguro no intervalo `[min, max]` (inclusivo).
///
/// # Panics
/// Panica se `min > max`.
///
/// # Exemplo
/// ```rust
/// use poker_engine::rng_crypto::secure_random_u32;
///
/// let dado = secure_random_u32(1..=6);  // simula um D6
/// let indice = secure_random_u32(0..=51); // índice de carta no baralho
/// ```
pub fn secure_random_u32(range: std::ops::RangeInclusive<u32>) -> u32 {
    let min = *range.start();
    let max = *range.end();
    assert!(min <= max, "secure_random_u32: min ({min}) > max ({max})");

    // Caso especial: range completo [0, u32::MAX] — sem bias possível
    if min == 0 && max == u32::MAX {
        let mut buf = [0u8; 4];
        csprng().fill_bytes(&mut buf);
        return u32::from_le_bytes(buf);
    }

    let span = max - min + 1;
    let rejection_boundary = u32::MAX - (u32::MAX % span);

    loop {
        let mut buf = [0u8; 4];
        csprng().fill_bytes(&mut buf);
        let candidate = u32::from_le_bytes(buf);

        if candidate < rejection_boundary {
            return min + (candidate % span);
        }
    }
}

/// Gera um u64 aleatório criptograficamente seguro no intervalo `[min, max]` (inclusivo).
///
/// # Panics
/// Panica se `min > max`.
pub fn secure_random_u64(range: std::ops::RangeInclusive<u64>) -> u64 {
    let min = *range.start();
    let max = *range.end();
    assert!(min <= max, "secure_random_u64: min ({min}) > max ({max})");

    // Caso especial: range completo [0, u64::MAX] — sem bias possível
    if min == 0 && max == u64::MAX {
        let mut buf = [0u8; 8];
        csprng().fill_bytes(&mut buf);
        return u64::from_le_bytes(buf);
    }

    let span = max - min + 1;
    let rejection_boundary = u64::MAX - (u64::MAX % span);

    loop {
        let mut buf = [0u8; 8];
        csprng().fill_bytes(&mut buf);
        let candidate = u64::from_le_bytes(buf);

        if candidate < rejection_boundary {
            return min + (candidate % span);
        }
    }
}

/// Gera um f64 aleatório criptograficamente seguro no intervalo `[0.0, 1.0)`.
///
/// Útil para probabilidades (ex: rake progressivo, decisões de IA).
pub fn secure_random_f64() -> f64 {
    let mut buf = [0u8; 8];
    csprng().fill_bytes(&mut buf);

    // Usa 52 bits de mantissa (IEEE 754 f64 tem 53 bits de precisão)
    let mantissa = u64::from_le_bytes(buf) & 0x000F_FFFF_FFFF_FFFF;
    mantissa as f64 / (1u64 << 52) as f64
}

/// Gera um booleano aleatório criptograficamente seguro com probabilidade `p` de true.
///
/// # Exemplo
/// ```rust
/// use poker_engine::rng_crypto::secure_random_bool;
///
/// let coin_flip = secure_random_bool(0.5);  // 50% true, 50% false
/// let rare_event = secure_random_bool(0.01); // 1% true
/// ```
pub fn secure_random_bool(probability_true: f64) -> bool {
    assert!(
        (0.0..=1.0).contains(&probability_true),
        "secure_random_bool: probability must be in [0.0, 1.0], got {probability_true}"
    );
    secure_random_f64() < probability_true
}

/// Preenche um buffer de bytes com dados aleatórios criptograficamente seguros.
///
/// Útil para gerar IDs de sessão, tokens, salts, etc.
pub fn secure_random_bytes(buf: &mut [u8]) {
    csprng().fill_bytes(buf);
}

// ─── Testes ───

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    // ─── secure_shuffle ───

    #[test]
    fn test_secure_shuffle_preserves_elements() {
        let original: Vec<u32> = (0..52).collect();
        let mut shuffled = original.clone();
        secure_shuffle(&mut shuffled);

        // Todos os elementos devem estar presentes
        shuffled.sort();
        assert_eq!(shuffled, original);
    }

    #[test]
    fn test_secure_shuffle_changes_order() {
        // Probabilisticamente, um shuffle de 52 elementos deve mudar a ordem
        let original: Vec<u32> = (0..52).collect();
        let mut shuffled = original.clone();
        secure_shuffle(&mut shuffled);

        // A chance de 52 elementos ficarem na mesma ordem é 1/52! ≈ 0
        assert_ne!(
            shuffled, original,
            "Shuffle não alterou a ordem — extremamente improvável"
        );
    }

    #[test]
    fn test_secure_shuffle_empty_slice() {
        let mut empty: Vec<u32> = vec![];
        secure_shuffle(&mut empty);
        assert!(empty.is_empty());
    }

    #[test]
    fn test_secure_shuffle_single_element() {
        let mut single = vec![42u32];
        secure_shuffle(&mut single);
        assert_eq!(single, vec![42]);
    }

    // ─── secure_random_u32 ───

    #[test]
    fn test_secure_random_u32_range() {
        for _ in 0..1000 {
            let val = secure_random_u32(1..=6);
            assert!((1..=6).contains(&val), "Valor {val} fora do range 1..=6");
        }
    }

    #[test]
    fn test_secure_random_u32_min_equals_max() {
        let val = secure_random_u32(42..=42);
        assert_eq!(val, 42);
    }

    #[test]
    #[allow(clippy::absurd_extreme_comparisons)]
    fn test_secure_random_u32_full_range() {
        let val = secure_random_u32(0..=u32::MAX);
        // Não panica — apenas verificamos que retorna algo válido
        assert!(val <= u32::MAX);
    }

    #[test]
    #[should_panic(expected = "min (10) > max (5)")]
    #[allow(clippy::reversed_empty_ranges)]
    fn test_secure_random_u32_panics_on_invalid_range() {
        secure_random_u32(10..=5);
    }

    #[test]
    fn test_secure_random_u32_distribution() {
        // Teste de distribuição: 6000 lançamentos de D6 devem ter ~1000 cada face
        let mut counts = [0u32; 6];
        for _ in 0..6000 {
            let val = secure_random_u32(1..=6);
            counts[(val - 1) as usize] += 1;
        }

        for &count in &counts {
            // Cada face deve aparecer entre 800 e 1200 vezes (margem generosa)
            assert!(
                (800..=1200).contains(&count),
                "Distribuição suspeita: face com {count} ocorrências em 6000 lançamentos"
            );
        }
    }

    // ─── secure_random_u64 ───

    #[test]
    fn test_secure_random_u64_range() {
        for _ in 0..100 {
            let val = secure_random_u64(0..=99);
            assert!(val <= 99);
        }
    }

    #[test]
    fn test_secure_random_u64_large_range() {
        let val = secure_random_u64(1_000_000..=2_000_000);
        assert!((1_000_000..=2_000_000).contains(&val));
    }

    #[test]
    #[should_panic(expected = "min (100) > max (50)")]
    #[allow(clippy::reversed_empty_ranges)]
    fn test_secure_random_u64_panics_on_invalid_range() {
        secure_random_u64(100..=50);
    }

    // ─── secure_random_f64 ───

    #[test]
    fn test_secure_random_f64_range() {
        for _ in 0..1000 {
            let val = secure_random_f64();
            assert!((0.0..1.0).contains(&val), "Valor {val} fora de [0.0, 1.0)");
        }
    }

    #[test]
    fn test_secure_random_f64_not_all_same() {
        let mut seen = HashSet::new();
        for _ in 0..100 {
            let val = secure_random_f64();
            // Armazena como bits truncados para comparação
            seen.insert((val * 1_000_000.0) as u64);
        }
        // Deve haver pelo menos 50 valores distintos em 100 amostras
        assert!(
            seen.len() >= 50,
            "Pouca variação: apenas {} valores distintos em 100 amostras",
            seen.len()
        );
    }

    // ─── secure_random_bool ───

    #[test]
    fn test_secure_random_bool_always_true() {
        for _ in 0..100 {
            assert!(secure_random_bool(1.0));
        }
    }

    #[test]
    fn test_secure_random_bool_always_false() {
        for _ in 0..100 {
            assert!(!secure_random_bool(0.0));
        }
    }

    #[test]
    fn test_secure_random_bool_distribution() {
        let mut trues = 0;
        let total = 10_000;
        for _ in 0..total {
            if secure_random_bool(0.3) {
                trues += 1;
            }
        }
        let ratio = trues as f64 / total as f64;
        // 30% ± 5% (margem generosa para 10k amostras)
        assert!(
            (0.25..=0.35).contains(&ratio),
            "Distribuição suspeita: {trues}/{total} = {ratio:.3} true com p=0.3"
        );
    }

    #[test]
    #[should_panic(expected = "probability must be in [0.0, 1.0]")]
    fn test_secure_random_bool_panics_invalid_prob() {
        secure_random_bool(1.5);
    }

    // ─── secure_random_bytes ───

    #[test]
    fn test_secure_random_bytes_fills_buffer() {
        let mut buf = [0u8; 32];
        secure_random_bytes(&mut buf);
        // Extremamente improvável que 32 bytes sejam todos zero
        assert!(
            buf.iter().any(|&b| b != 0),
            "32 bytes aleatórios todos zero — extremamente improvável"
        );
    }

    #[test]
    fn test_secure_random_bytes_not_all_same() {
        let mut buf1 = [0u8; 64];
        let mut buf2 = [0u8; 64];
        secure_random_bytes(&mut buf1);
        secure_random_bytes(&mut buf2);
        assert_ne!(
            buf1, buf2,
            "Dois buffers de 64 bytes idênticos — extremamente improvável"
        );
    }

    #[test]
    fn test_secure_random_bytes_empty_buffer() {
        let mut empty: [u8; 0] = [];
        secure_random_bytes(&mut empty);
        // Não panica com buffer vazio
    }
}
