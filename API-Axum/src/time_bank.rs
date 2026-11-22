//! Tempo extra: os segundos que cada jogada ganha quando o tempo normal acaba.
//!
//! Regra (decisão do proprietário em 21/11/2026, todos os jogos, cash e
//! torneio): cada jogada tem 15 s de tempo normal e, acabados eles, mais 5 s
//! de tempo extra. O tempo extra renova a cada jogada: não acumula, não é
//! gasto de uma jogada para outra e não depende de quantas mãos o jogador
//! recebeu. Quem está desconectado não ganha o tempo extra.

use std::time::Duration;

/// Tempo normal de cada jogada.
pub const TURN: Duration = Duration::from_secs(15);
/// Tempo extra de cada jogada, depois do tempo normal.
pub const EXTRA: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Copy, Default)]
pub struct TimeBanks {
    disabled: bool,
}

impl TimeBanks {
    /// Mesa sem tempo extra: o turno acaba no tempo normal (testes de tempo esgotado).
    pub fn disabled() -> Self {
        Self { disabled: true }
    }

    /// Tempo extra que cada jogada recebe nesta mesa.
    pub fn extra(&self) -> Duration {
        if self.disabled {
            Duration::ZERO
        } else {
            EXTRA
        }
    }

    /// Prazo do turno: tempo normal mais o tempo extra de quem pode usá-lo.
    pub fn turn_limit(&self, base: Duration, may_use_extra: bool) -> Duration {
        if may_use_extra {
            base + self.extra()
        } else {
            base
        }
    }

    /// Tempo extra que ainda resta neste instante de um turno que já dura `elapsed`.
    pub fn live(&self, elapsed: Duration, base: Duration) -> Duration {
        self.extra().saturating_sub(elapsed.saturating_sub(base))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn secs(value: u64) -> Duration {
        Duration::from_secs(value)
    }

    #[test]
    fn every_turn_gets_fifteen_plus_five_seconds() {
        let banks = TimeBanks::default();
        assert_eq!(TURN, secs(15));
        assert_eq!(banks.extra(), secs(5));
        assert_eq!(banks.turn_limit(TURN, true), secs(20));
    }

    #[test]
    fn disconnected_player_gets_only_the_normal_time() {
        let banks = TimeBanks::default();
        assert_eq!(banks.turn_limit(TURN, false), TURN);
    }

    #[test]
    fn extra_time_starts_only_after_the_normal_time() {
        let banks = TimeBanks::default();
        assert_eq!(banks.live(secs(10), TURN), EXTRA);
        assert_eq!(banks.live(TURN, TURN), EXTRA);
        assert_eq!(banks.live(secs(17), TURN), secs(3));
        assert_eq!(banks.live(secs(90), TURN), Duration::ZERO);
    }

    #[test]
    fn extra_time_renews_and_never_accumulates() {
        // Sem estado por jogador: gastar o extra numa jogada não muda a
        // próxima, e jogar rápido não guarda segundos.
        let banks = TimeBanks::default();
        assert_eq!(banks.turn_limit(TURN, true), secs(20));
        assert_eq!(banks.live(secs(19), TURN), secs(1));
        assert_eq!(banks.turn_limit(TURN, true), secs(20));
    }

    #[test]
    fn disabled_bank_never_extends_a_turn() {
        let banks = TimeBanks::disabled();
        assert_eq!(banks.extra(), Duration::ZERO);
        assert_eq!(banks.turn_limit(TURN, true), TURN);
    }
}
