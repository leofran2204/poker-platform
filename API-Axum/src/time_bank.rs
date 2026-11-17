//! Banco de tempo: reserva pessoal usada quando o tempo normal da jogada acaba.
//!
//! Regra, no padrão das salas online: cada jogador senta com 30 s de reserva.
//! Quando o tempo normal do turno termina, a reserva começa a correr sozinha e
//! o que foi gasto não volta na hora. A cada 10 mãos recebidas a reserva ganha
//! 5 s, até o teto de 60 s. Quem está desconectado não usa a reserva.
//!
//! A reserva vive na memória do ator da mesa: volta a 30 s se o jogador sair e
//! sentar de novo, se a mesa for recriada ou, no torneio, ao trocar de mesa.

use std::collections::HashMap;
use std::time::Duration;

pub const INITIAL: Duration = Duration::from_secs(30);
pub const MAX: Duration = Duration::from_secs(60);
pub const REFILL: Duration = Duration::from_secs(5);
pub const REFILL_EVERY_HANDS: u32 = 10;

#[derive(Debug, Clone, Copy)]
struct Reserve {
    remaining: Duration,
    hands_since_refill: u32,
}

impl Default for Reserve {
    fn default() -> Self {
        Self {
            remaining: INITIAL,
            hands_since_refill: 0,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TimeBanks {
    reserves: HashMap<String, Reserve>,
    disabled: bool,
}

impl TimeBanks {
    /// Mesa sem banco de tempo: o turno acaba no tempo normal (testes de tempo esgotado).
    pub fn disabled() -> Self {
        Self {
            reserves: HashMap::new(),
            disabled: true,
        }
    }

    /// Reserva guardada do jogador (sem contar o turno em andamento).
    pub fn remaining(&self, player_id: &str) -> Duration {
        if self.disabled {
            return Duration::ZERO;
        }
        self.reserves
            .get(player_id)
            .map_or(INITIAL, |reserve| reserve.remaining)
    }

    /// Prazo do turno: tempo normal mais a reserva de quem pode usá-la.
    pub fn turn_limit(&self, player_id: &str, base: Duration, may_use_reserve: bool) -> Duration {
        if may_use_reserve {
            base + self.remaining(player_id)
        } else {
            base
        }
    }

    /// Reserva que ainda resta neste instante de um turno que já dura `elapsed`.
    pub fn live(&self, player_id: &str, elapsed: Duration, base: Duration) -> Duration {
        self.remaining(player_id)
            .saturating_sub(elapsed.saturating_sub(base))
    }

    /// Desconta da reserva o que o turno passou do tempo normal.
    pub fn charge(&mut self, player_id: &str, elapsed: Duration, base: Duration) {
        let overtime = elapsed.saturating_sub(base);
        if overtime.is_zero() {
            return;
        }
        let reserve = self.reserves.entry(player_id.to_string()).or_default();
        reserve.remaining = reserve.remaining.saturating_sub(overtime);
    }

    /// Conta uma mão recebida por jogador e repõe a reserva de quem fechou o ciclo.
    pub fn hand_dealt<'a>(&mut self, player_ids: impl IntoIterator<Item = &'a str>) {
        for player_id in player_ids {
            let reserve = self.reserves.entry(player_id.to_string()).or_default();
            reserve.hands_since_refill += 1;
            if reserve.hands_since_refill >= REFILL_EVERY_HANDS {
                reserve.hands_since_refill = 0;
                reserve.remaining = (reserve.remaining + REFILL).min(MAX);
            }
        }
    }

    /// Jogador saiu da mesa: a reserva dele não fica guardada.
    pub fn forget(&mut self, player_id: &str) {
        self.reserves.remove(player_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: Duration = Duration::from_secs(30);
    const fn secs(value: u64) -> Duration {
        Duration::from_secs(value)
    }

    #[test]
    fn new_player_starts_with_the_initial_reserve() {
        let banks = TimeBanks::default();
        assert_eq!(banks.remaining("ana"), INITIAL);
        assert_eq!(banks.turn_limit("ana", BASE, true), BASE + INITIAL);
    }

    #[test]
    fn disconnected_player_gets_only_the_normal_time() {
        let banks = TimeBanks::default();
        assert_eq!(banks.turn_limit("ana", BASE, false), BASE);
    }

    #[test]
    fn acting_inside_the_normal_time_costs_nothing() {
        let mut banks = TimeBanks::default();
        banks.charge("ana", secs(29), BASE);
        banks.charge("ana", BASE, BASE);
        assert_eq!(banks.remaining("ana"), INITIAL);
    }

    #[test]
    fn overtime_is_charged_and_shortens_the_next_turn() {
        let mut banks = TimeBanks::default();
        banks.charge("ana", secs(42), BASE);
        assert_eq!(banks.remaining("ana"), secs(18));
        assert_eq!(banks.turn_limit("ana", BASE, true), secs(48));
        // A reserva dos outros não muda.
        assert_eq!(banks.remaining("beto"), INITIAL);
    }

    #[test]
    fn reserve_never_goes_below_zero() {
        let mut banks = TimeBanks::default();
        banks.charge("ana", secs(500), BASE);
        assert_eq!(banks.remaining("ana"), Duration::ZERO);
        assert_eq!(banks.turn_limit("ana", BASE, true), BASE);
    }

    #[test]
    fn live_reserve_counts_down_during_the_turn() {
        let banks = TimeBanks::default();
        assert_eq!(banks.live("ana", secs(10), BASE), INITIAL);
        assert_eq!(banks.live("ana", secs(40), BASE), secs(20));
        assert_eq!(banks.live("ana", secs(90), BASE), Duration::ZERO);
    }

    #[test]
    fn reserve_grows_every_ten_hands_up_to_the_cap() {
        let mut banks = TimeBanks::default();
        banks.charge("ana", secs(50), BASE); // sobra 10 s
        for _ in 0..REFILL_EVERY_HANDS - 1 {
            banks.hand_dealt(["ana"]);
        }
        assert_eq!(banks.remaining("ana"), secs(10));
        banks.hand_dealt(["ana"]);
        assert_eq!(banks.remaining("ana"), secs(15));

        // Quem não gasta chega ao teto e para nele.
        for _ in 0..REFILL_EVERY_HANDS * 20 {
            banks.hand_dealt(["beto"]);
        }
        assert_eq!(banks.remaining("beto"), MAX);
    }

    #[test]
    fn disabled_bank_never_extends_a_turn() {
        let mut banks = TimeBanks::disabled();
        banks.hand_dealt(["ana"; 40]);
        assert_eq!(banks.remaining("ana"), Duration::ZERO);
        assert_eq!(banks.turn_limit("ana", BASE, true), BASE);
    }

    #[test]
    fn leaving_the_table_resets_the_reserve() {
        let mut banks = TimeBanks::default();
        banks.charge("ana", secs(55), BASE);
        banks.forget("ana");
        assert_eq!(banks.remaining("ana"), INITIAL);
    }
}
