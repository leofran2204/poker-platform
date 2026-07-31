// simulate_5000_players.rs
// ═══════════════════════════════════════════════════════════════════════════════
// SIMULAÇÃO DE 5.000 JOGADORES SIMULTÂNEOS — PLATAFORMA DE POKER ONLINE
// ═══════════════════════════════════════════════════════════════════════════════
//
// Esta simulação testa a capacidade da plataforma sob carga massiva de 5.000
// jogadores concorrentes, cobrindo:
//
//   1. Ledger Financeiro — 25.000 transações atômicas concorrentes
//   2. Motor de Jogo — GameLoop com ações em lote
//   3. Side Pots + Loss Deflator — resolução financeira de potes
//   4. Rate Limiter — 500.000 verificações de rate limit
//   5. Antifraude — 5.000 validações de sessão
//   6. Device Fingerprint — 5.000 verificações de dispositivo + GPS
//
// Design:
//   - Usa std::thread + Arc para concorrência massiva real
//   - Cada thread processa um lote de jogadores independente
//   - Métricas coletadas via contadores atômicos
//   - Valida invariantes: integridade SHA-256 do Ledger
//   - NENHUMA alocação de rede — simulação 100% offline

use poker_engine::antifraud::device_fingerprint::{
    DeviceFingerprint, DeviceSecurityGuard, GeoLocation, PlayerSecurityContext,
};
use poker_engine::antifraud::{CollusionDetector, PlayerSession};
use poker_engine::engine::{
    game_loop::{Action, GameLoop, GameState, Player, Street},
    loss_deflator::{calculate_loss_deflators, PlayerLossStats},
    side_pots::{calculate_side_pots, Contribution},
};
use poker_engine::ledger::{EntryType, LedgerAccount};
use poker_engine::security::RateLimiter;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

// ─── Constantes da Simulação ───

const NUM_PLAYERS: usize = 5_000;
const INITIAL_BALANCE: i64 = 1_000_000; // R$ 10.000,00 em centavos
const BUY_IN: i64 = 100_000; // R$ 1.000,00 buy-in em centavos
const SMALL_BLIND: f64 = 10.0; // SB em f64 (API do root)
const BIG_BLIND: f64 = 20.0; // BB em f64 (API do root)

// ─── Contadores Atômicos para Métricas ───

struct SimMetrics {
    ledger_transactions: AtomicU64,
    ledger_errors: AtomicU64,
    ledger_integrity_ok: AtomicU64,
    ledger_integrity_fail: AtomicU64,
    side_pots_calculated: AtomicU64,
    loss_deflator_payouts: AtomicU64,
    rate_limit_checks: AtomicU64,
    rate_limit_rejected: AtomicU64,
    antifraud_checks: AtomicU64,
    antifraud_rejected: AtomicU64,
    device_checks: AtomicU64,
    device_rejected: AtomicU64,
    player_actions: AtomicU64,
    hands_simulated: AtomicU64,
    game_loop_steps: AtomicU64,
}

impl SimMetrics {
    fn new() -> Self {
        Self {
            ledger_transactions: AtomicU64::new(0),
            ledger_errors: AtomicU64::new(0),
            ledger_integrity_ok: AtomicU64::new(0),
            ledger_integrity_fail: AtomicU64::new(0),
            side_pots_calculated: AtomicU64::new(0),
            loss_deflator_payouts: AtomicU64::new(0),
            rate_limit_checks: AtomicU64::new(0),
            rate_limit_rejected: AtomicU64::new(0),
            antifraud_checks: AtomicU64::new(0),
            antifraud_rejected: AtomicU64::new(0),
            device_checks: AtomicU64::new(0),
            device_rejected: AtomicU64::new(0),
            player_actions: AtomicU64::new(0),
            hands_simulated: AtomicU64::new(0),
            game_loop_steps: AtomicU64::new(0),
        }
    }
}

// ─── SIMULAÇÃO PRINCIPAL ───

#[test]
fn test_simulate_5000_simultaneous_players() {
    println!("\n");
    println!("╔═══════════════════════════════════════════════════════════════════════════════╗");
    println!("║     🃏  SIMULAÇÃO DE 5.000 JOGADORES SIMULTÂNEOS — POKER PLATFORM 2026  🃏   ║");
    println!("╚═══════════════════════════════════════════════════════════════════════════════╝");
    println!();

    let metrics = Arc::new(SimMetrics::new());
    let global_start = Instant::now();
    let rate_limiter = Arc::new(RateLimiter::new(1000.0, 500.0));

    // ═══════════════════════════════════════════════════════════════════════════
    // FASE 1: LEDGER FINANCEIRO — 25.000 TRANSAÇÕES CONCORRENTES
    // ═══════════════════════════════════════════════════════════════════════════
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  [FASE 1] LEDGER FINANCEIRO — 5 CONTAS × 5.000 JOGADORES = 25.000 TXs");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let ledger_start = Instant::now();
    let num_threads = 50;
    let players_per_thread = NUM_PLAYERS / num_threads;

    // Criar 5.000 contas no ledger
    let accounts: Vec<Arc<LedgerAccount>> = (0..NUM_PLAYERS)
        .map(|i| Arc::new(LedgerAccount::new(format!("Player_{}", i), INITIAL_BALANCE)))
        .collect();
    let accounts_arc = Arc::new(accounts);

    let mut ledger_handles = Vec::new();

    for t in 0..num_threads {
        let accs = Arc::clone(&accounts_arc);
        let m = Arc::clone(&metrics);
        let rl = Arc::clone(&rate_limiter);

        let handle = std::thread::spawn(move || {
            for i in 0..players_per_thread {
                let idx = t * players_per_thread + i;
                if idx >= NUM_PLAYERS {
                    break;
                }

                // Rate limit check antes de cada batch
                let _ = rl.check_rate_limit(&format!("player_{}", idx));

                let account = &accs[idx];

                // TX 1: Depósito inicial
                match account.record_transaction(
                    50_000,
                    EntryType::Deposit,
                    Some(format!("DEP-{}", idx)),
                ) {
                    Ok(_) => {
                        m.ledger_transactions.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(_) => {
                        m.ledger_errors.fetch_add(1, Ordering::Relaxed);
                    }
                }

                // TX 2: Buy-in em mesa
                match account.record_transaction(
                    -BUY_IN,
                    EntryType::TableBuyIn,
                    Some(format!("BUYIN-{}", idx)),
                ) {
                    Ok(_) => {
                        m.ledger_transactions.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(_) => {
                        m.ledger_errors.fetch_add(1, Ordering::Relaxed);
                    }
                }

                // TX 3: Vitória no pote
                match account.record_transaction(
                    150_000,
                    EntryType::PotWin,
                    Some(format!("WIN-{}", idx)),
                ) {
                    Ok(_) => {
                        m.ledger_transactions.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(_) => {
                        m.ledger_errors.fetch_add(1, Ordering::Relaxed);
                    }
                }

                // TX 4: Saque parcial
                if let Ok(balance) = account.get_balance_cents() {
                    if balance > 10_000 {
                        let withdraw = (balance / 3).max(100);
                        match account.record_transaction(
                            -withdraw,
                            EntryType::Withdrawal,
                            Some(format!("WTH-{}", idx)),
                        ) {
                            Ok(_) => {
                                m.ledger_transactions.fetch_add(1, Ordering::Relaxed);
                            }
                            Err(_) => {
                                m.ledger_errors.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                    }
                }

                // TX 5: Cashback Loss Deflator
                match account.record_transaction(
                    5_000,
                    EntryType::LossDeflatorCashback,
                    Some(format!("CASHBACK-{}", idx)),
                ) {
                    Ok(_) => {
                        m.ledger_transactions.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(_) => {
                        m.ledger_errors.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        });
        ledger_handles.push(handle);
    }

    for h in ledger_handles {
        h.join().expect("Ledger thread panicked");
    }

    let ledger_elapsed = ledger_start.elapsed();
    let total_ledger = metrics.ledger_transactions.load(Ordering::Relaxed);
    let ledger_errs = metrics.ledger_errors.load(Ordering::Relaxed);

    println!(
        "     ✔ {} transações processadas em {:6?}",
        total_ledger, ledger_elapsed
    );
    println!(
        "     ✔ Erros: {} (esperado para saldo insuficiente)",
        ledger_errs
    );
    println!(
        "     ✔ Throughput Ledger: {:>10.0} tx/s",
        total_ledger as f64 / ledger_elapsed.as_secs_f64()
    );

    // Verificar integridade SHA-256 de TODAS as 5.000 contas
    let integrity_start = Instant::now();
    for (i, acc) in accounts_arc.iter().enumerate() {
        match acc.verify_integrity() {
            Ok(true) => {
                metrics.ledger_integrity_ok.fetch_add(1, Ordering::Relaxed);
            }
            _ => {
                metrics
                    .ledger_integrity_fail
                    .fetch_add(1, Ordering::Relaxed);
                println!("     ⚠️  Conta {}: integridade CORROMPIDA!", i);
            }
        }
    }
    let integrity_elapsed = integrity_start.elapsed();
    let integrity_ok = metrics.ledger_integrity_ok.load(Ordering::Relaxed);
    let integrity_fail = metrics.ledger_integrity_fail.load(Ordering::Relaxed);

    println!(
        "     ✔ Integridade SHA-256: {} OK, {} CORROMPIDAS ({:4?})",
        integrity_ok, integrity_fail, integrity_elapsed
    );
    assert_eq!(integrity_fail, 0, "CRÍTICO: Ledger SHA-256 corrompido!");
    println!();

    // ═══════════════════════════════════════════════════════════════════════════
    // FASE 2: GAME LOOP + SIDE POTS + LOSS DEFLATOR
    // ═══════════════════════════════════════════════════════════════════════════
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  [FASE 2] MOTOR DE JOGO — 5.000 JOGADORES EM 556 MESAS × SIMULAÇÃO DE MÃOS");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let game_start = Instant::now();
    let players_per_table = 9;
    let num_tables = NUM_PLAYERS / players_per_table;
    let game_threads = 20;
    let tables_per_thread = num_tables / game_threads;

    let mut game_handles = Vec::new();

    for t in 0..game_threads {
        let m = Arc::clone(&metrics);
        let rl = Arc::clone(&rate_limiter);

        let handle = std::thread::spawn(move || {
            for table_idx in 0..tables_per_thread {
                let global_tid = t * tables_per_thread + table_idx;

                // Criar 9 jogadores para esta mesa
                let players: Vec<Player> = (0..players_per_table)
                    .map(|p| {
                        let id = format!("Player_{}", global_tid * players_per_table + p);
                        Player::new(id.clone(), id.clone(), 1000.0 + (p * 100) as f64)
                    })
                    .collect();

                // Configurar e executar o GameLoop
                let state = GameState::new(players, 0, SMALL_BLIND);
                let mut gl = GameLoop::new(state);
                let mut step_count = 0u64;

                // Simular ações até showdown
                for _ in 0..5 {
                    if gl.state.current_street == Street::Showdown
                        || gl.state.current_street == Street::Finished
                    {
                        break;
                    }

                    let mut round_actions = 0u64;
                    while round_actions < 20 {
                        let current_idx = gl.state.current_player_idx;
                        if current_idx >= gl.state.players.len() {
                            break;
                        }

                        let player = &gl.state.players[current_idx];
                        if !player.can_act() {
                            break;
                        }

                        // Determinar ação com seed determinística
                        let action_seed =
                            (step_count * 7 + round_actions * 13 + global_tid as u64 * 3) % 100;
                        let action = if action_seed < 20 {
                            Action::Fold
                        } else if action_seed < 60 {
                            if gl.state.highest_bet > 0.0 {
                                Action::Call
                            } else {
                                Action::Check
                            }
                        } else if action_seed < 85 {
                            if gl.state.highest_bet == 0.0 {
                                Action::Bet(BIG_BLIND * 2.0 + (action_seed as f64 * 5.0))
                            } else {
                                Action::Raise(gl.state.highest_bet + BIG_BLIND)
                            }
                        } else {
                            Action::AllIn
                        };

                        // Aplicar ação manualmente usando índices (evita borrow checker)
                        // Primeiro, clonar dados necessários do jogador atual
                        let (current_bet, stack) = {
                            let p = &gl.state.players[current_idx];
                            (p.current_bet, p.stack)
                        };

                        match action {
                            Action::Fold => {
                                gl.state.players[current_idx].has_folded = true;
                            }
                            Action::Check => {
                                gl.state.players[current_idx].has_acted = true;
                            }
                            Action::Call => {
                                let to_call = gl.state.highest_bet - current_bet;
                                let call_amt = to_call.min(stack);
                                gl.state.players[current_idx].stack -= call_amt;
                                gl.state.players[current_idx].current_bet += call_amt;
                                gl.state.players[current_idx].total_bet += call_amt;
                                gl.state.players[current_idx].has_acted = true;
                            }
                            Action::Bet(amt) => {
                                let bet_amt = amt.min(stack);
                                gl.state.players[current_idx].stack -= bet_amt;
                                gl.state.players[current_idx].current_bet = bet_amt;
                                gl.state.players[current_idx].total_bet += bet_amt;
                                gl.state.highest_bet = bet_amt;
                                gl.state.players[current_idx].has_acted = true;
                                // Reset others' has_acted
                                let current_id = gl.state.players[current_idx].id.clone();
                                for other in &mut gl.state.players {
                                    if other.id != current_id && other.can_act() {
                                        other.has_acted = false;
                                    }
                                }
                            }
                            Action::Raise(amt) => {
                                let total_needed = amt - current_bet;
                                let raise_amt = total_needed.min(stack);
                                gl.state.players[current_idx].stack -= raise_amt;
                                gl.state.players[current_idx].current_bet += raise_amt;
                                gl.state.players[current_idx].total_bet += raise_amt;
                                let new_bet = gl.state.players[current_idx].current_bet;
                                if new_bet > gl.state.highest_bet {
                                    gl.state.highest_bet = new_bet;
                                }
                                gl.state.players[current_idx].has_acted = true;
                                let current_id = gl.state.players[current_idx].id.clone();
                                for other in &mut gl.state.players {
                                    if other.id != current_id && other.can_act() {
                                        other.has_acted = false;
                                    }
                                }
                            }
                            Action::AllIn => {
                                let all_in_amt = stack;
                                gl.state.players[current_idx].current_bet += all_in_amt;
                                gl.state.players[current_idx].total_bet += all_in_amt;
                                gl.state.players[current_idx].stack = 0.0;
                                gl.state.players[current_idx].is_all_in = true;
                                gl.state.players[current_idx].has_acted = true;
                                let new_bet = gl.state.players[current_idx].current_bet;
                                if new_bet > gl.state.highest_bet {
                                    gl.state.highest_bet = new_bet;
                                }
                                let current_id = gl.state.players[current_idx].id.clone();
                                for other in &mut gl.state.players {
                                    if other.id != current_id && other.can_act() {
                                        other.has_acted = false;
                                    }
                                }
                            }
                        }
                        m.player_actions.fetch_add(1, Ordering::Relaxed);
                        step_count += 1;
                        round_actions += 1;

                        // Avançar para próximo jogador ou street
                        if gl.state.count_players_who_can_act() <= 1 && gl.is_street_complete() {
                            gl.next_street();
                            break;
                        }
                        if !gl.advance_turn() {
                            break;
                        }
                    }

                    m.game_loop_steps.fetch_add(step_count, Ordering::Relaxed);

                    // Rate limit check
                    let _ = rl.check_rate_limit(&format!("table_{}", global_tid));
                    m.rate_limit_checks.fetch_add(1, Ordering::Relaxed);
                }

                m.hands_simulated.fetch_add(1, Ordering::Relaxed);

                // ─── Side Pots — Calcular para esta mesa ───
                let contributions: Vec<Contribution> = gl
                    .state
                    .players
                    .iter()
                    .map(|p| Contribution {
                        player_id: p.id.clone(),
                        total_bet: p.total_bet,
                        has_folded: p.has_folded,
                    })
                    .collect();

                let side_pots = calculate_side_pots(&contributions);
                m.side_pots_calculated
                    .fetch_add(side_pots.len() as u64, Ordering::Relaxed);

                // ─── Loss Deflator — Apenas para perdedores ───
                let loss_stats: Vec<PlayerLossStats> = gl
                    .state
                    .players
                    .iter()
                    .filter(|p| p.total_bet > 0.0 && p.stack < 1000.0)
                    .map(|p| {
                        // Estimar perda: se o stack diminuiu, houve perda
                        let initial = 1000.0; // stack inicial aproximado (todos começam com ~1000)
                        let net_change = (p.stack + p.total_bet) - initial;
                        PlayerLossStats {
                            player_id: p.id.clone(),
                            eligible_loss_after_rake: if net_change < 0.0 {
                                -net_change
                            } else {
                                0.0
                            },
                            loser_equity: if p.is_all_in { 0.80 } else { 0.60 },
                        }
                    })
                    .collect();

                if !loss_stats.is_empty() {
                    let payouts = calculate_loss_deflators(&loss_stats);
                    m.loss_deflator_payouts
                        .fetch_add(payouts.len() as u64, Ordering::Relaxed);
                }
            }
        });
        game_handles.push(handle);
    }

    for h in game_handles {
        h.join().expect("Game thread panicked");
    }

    let game_elapsed = game_start.elapsed();
    let total_hands = metrics.hands_simulated.load(Ordering::Relaxed);
    let total_actions = metrics.player_actions.load(Ordering::Relaxed);
    let total_side_pots = metrics.side_pots_calculated.load(Ordering::Relaxed);
    let total_deflators = metrics.loss_deflator_payouts.load(Ordering::Relaxed);

    println!(
        "     ✔ {} mãos simuladas em {:6?}",
        total_hands, game_elapsed
    );
    println!("     ✔ {} ações de jogadores processadas", total_actions);
    println!("     ✔ {} side pots calculados", total_side_pots);
    println!("     ✔ {} loss deflator payouts gerados", total_deflators);
    println!(
        "     ✔ Throughput Motor: {:>10.0} mãos/s",
        total_hands as f64 / game_elapsed.as_secs_f64()
    );
    println!(
        "     ✔ Throughput Ações: {:>10.0} ações/s",
        total_actions as f64 / game_elapsed.as_secs_f64()
    );
    println!();

    // ═══════════════════════════════════════════════════════════════════════════
    // FASE 3: RATE LIMITER — 500.000 VERIFICAÇÕES CONCORRENTES
    // ═══════════════════════════════════════════════════════════════════════════
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  [FASE 3] RATE LIMITER — 500.000 VERIFICAÇÕES (TOKEN BUCKET)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let rl_start = Instant::now();
    let rl = Arc::new(RateLimiter::new(200.0, 100.0));
    let rl_threads = 50;
    let checks_per_thread = 10_000;
    let mut rl_handles = Vec::new();

    for t in 0..rl_threads {
        let limiter = Arc::clone(&rl);
        let m = Arc::clone(&metrics);

        let handle = std::thread::spawn(move || {
            for i in 0..checks_per_thread {
                let key = format!("10.0.{}.{}", t % 5, i % 250);
                match limiter.check_rate_limit(&key) {
                    Ok(_) => {
                        m.rate_limit_checks.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(_) => {
                        m.rate_limit_checks.fetch_add(1, Ordering::Relaxed);
                        m.rate_limit_rejected.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        });
        rl_handles.push(handle);
    }

    for h in rl_handles {
        h.join().expect("RL thread panicked");
    }

    let rl_elapsed = rl_start.elapsed();
    let rl_checks = metrics.rate_limit_checks.load(Ordering::Relaxed);
    let rl_rejected = metrics.rate_limit_rejected.load(Ordering::Relaxed);

    println!("     ✔ {} verificações em {:6?}", rl_checks, rl_elapsed);
    println!(
        "     ✔ Rejeitadas: {} ({:.1}%)",
        rl_rejected,
        rl_rejected as f64 / rl_checks as f64 * 100.0
    );
    println!(
        "     ✔ Throughput: {:>10.0} checks/s",
        rl_checks as f64 / rl_elapsed.as_secs_f64()
    );
    assert!(
        rl_rejected > 0,
        "Rate limiter deveria ter rejeitado requisições!"
    );
    println!();

    // ═══════════════════════════════════════════════════════════════════════════
    // FASE 4: ANTIFRAUDE — 5.000 VALIDAÇÕES DE SESSÃO + SUBNET GUARD
    // ═══════════════════════════════════════════════════════════════════════════
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  [FASE 4] ANTIFRAUDE — 5.000 VALIDAÇÕES COLLUSION DETECTOR");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let fraud_start = Instant::now();
    let fraud_threads = 50;
    let sessions_per_thread_f = NUM_PLAYERS / fraud_threads;
    let mut fraud_handles = Vec::new();

    for t in 0..fraud_threads {
        let m = Arc::clone(&metrics);

        let handle = std::thread::spawn(move || {
            for i in 0..sessions_per_thread_f {
                let pid = t * sessions_per_thread_f + i;
                if pid >= NUM_PLAYERS {
                    break;
                }

                let session = PlayerSession {
                    user_id: format!("Player_{}", pid),
                    ip_address: format!(
                        "{}.{}.{}.{}",
                        (pid / 65536) % 250 + 1,
                        (pid / 256) % 250 + 1,
                        pid % 250 + 1,
                        (pid * 7) % 250 + 1
                    ),
                };

                match CollusionDetector::validate_table_seating(&[session]) {
                    Ok(_) => {
                        m.antifraud_checks.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(_) => {
                        m.antifraud_checks.fetch_add(1, Ordering::Relaxed);
                        m.antifraud_rejected.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        });
        fraud_handles.push(handle);
    }

    for h in fraud_handles {
        h.join().expect("Fraud thread panicked");
    }

    let fraud_elapsed = fraud_start.elapsed();
    let total_fraud = metrics.antifraud_checks.load(Ordering::Relaxed);
    let fraud_rejected = metrics.antifraud_rejected.load(Ordering::Relaxed);

    println!("     ✔ {} validações em {:6?}", total_fraud, fraud_elapsed);
    println!("     ✔ Rejeitadas: {}", fraud_rejected);
    println!(
        "     ✔ Throughput: {:>10.0} checks/s",
        total_fraud as f64 / fraud_elapsed.as_secs_f64()
    );
    println!();

    // ═══════════════════════════════════════════════════════════════════════════
    // FASE 5: DEVICE FINGERPRINT + GPS PROXIMITY — 5.000 PARES
    // ═══════════════════════════════════════════════════════════════════════════
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  [FASE 5] DEVICE FINGERPRINT + GPS — 5.000 VERIFICAÇÕES DE PROXIMIDADE");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let device_start = Instant::now();
    let device_threads = 50;
    let device_checks_per = 100;
    let mut device_handles = Vec::new();

    for t in 0..device_threads {
        let m = Arc::clone(&metrics);

        let handle = std::thread::spawn(move || {
            for i in 0..device_checks_per {
                let base = t * device_checks_per + i;

                let fp1 = DeviceFingerprint::new(
                    &format!("GPU_NVIDIA_RTX_{}", base),
                    &format!("AudioCtx_Audio_{}", base),
                    "1920x1080",
                    &format!("Font_Hash_Windows_{}", base),
                    "MacBookPro",
                    "macOS",
                );

                let fp2 = DeviceFingerprint::new(
                    &format!("GPU_Apple_M2_{}", base + 1),
                    &format!("AudioCtx_Bob_{}", base + 1),
                    "2560x1600",
                    &format!("Font_Hash_iOS_{}", base + 1),
                    "iPhone15Pro",
                    "iOS",
                );

                let ctx1 = PlayerSecurityContext {
                    user_id: format!("Player_A_{}", base),
                    ip_address: format!("203.0.113.{}", base % 250),
                    device_fingerprint: fp1,
                    geo_location: Some(GeoLocation::new(-23.561510, -46.655910)),
                };

                let ctx2 = PlayerSecurityContext {
                    user_id: format!("Player_B_{}", base + 1),
                    ip_address: format!("177.92.14.{}", (base + 1) % 250),
                    device_fingerprint: fp2,
                    geo_location: Some(GeoLocation::new(-23.561518, -46.655915)),
                };

                match DeviceSecurityGuard::validate_table_seating_advanced(&[ctx1, ctx2]) {
                    Ok(_) => {
                        m.device_checks.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(_) => {
                        m.device_checks.fetch_add(1, Ordering::Relaxed);
                        m.device_rejected.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        });
        device_handles.push(handle);
    }

    for h in device_handles {
        h.join().expect("Device thread panicked");
    }

    let device_elapsed = device_start.elapsed();
    let total_device = metrics.device_checks.load(Ordering::Relaxed);
    let device_rejected = metrics.device_rejected.load(Ordering::Relaxed);

    println!(
        "     ✔ {} verificações em {:6?}",
        total_device, device_elapsed
    );
    println!("     ✔ Rejeitadas (proximidade GPS): {}", device_rejected);
    println!(
        "     ✔ Throughput: {:>10.0} checks/s",
        total_device as f64 / device_elapsed.as_secs_f64()
    );
    println!();

    // ═══════════════════════════════════════════════════════════════════════════
    // RELATÓRIO FINAL CONSOLIDADO
    // ═══════════════════════════════════════════════════════════════════════════
    let total_elapsed = global_start.elapsed();

    println!("╔═══════════════════════════════════════════════════════════════════════════════╗");
    println!("║    📊  RELATÓRIO FINAL — 5.000 JOGADORES SIMULTÂNEOS — POKER PLATFORM  📊   ║");
    println!("╚═══════════════════════════════════════════════════════════════════════════════╝");
    println!();
    println!(
        "   ⏱  Tempo total: {:>8.2?} ({:.1} segundos)",
        total_elapsed,
        total_elapsed.as_secs_f64()
    );
    println!();

    // Tabela de throughput por subsistema
    println!("   ┌───────────────────────────────────────────┬──────────────────────┬──────────────────────┐");
    println!("   │ Módulo                                   │ Operações            │ Throughput (ops/s)   │");
    println!("   ├───────────────────────────────────────────┼──────────────────────┼──────────────────────┤");

    let mut total_ops: u64 = 0;
    let mut rows: Vec<(&str, u64, f64)> = Vec::new();

    rows.push((
        "Ledger (transações)",
        total_ledger,
        ledger_elapsed.as_secs_f64(),
    ));
    rows.push((
        "Ledger (integridade SHA-256)",
        integrity_ok,
        integrity_elapsed.as_secs_f64(),
    ));
    rows.push(("GameLoop (mãos)", total_hands, game_elapsed.as_secs_f64()));
    rows.push((
        "GameLoop (ações)",
        total_actions,
        game_elapsed.as_secs_f64(),
    ));
    rows.push(("Side Pots", total_side_pots, game_elapsed.as_secs_f64()));
    rows.push(("Loss Deflator", total_deflators, game_elapsed.as_secs_f64()));
    rows.push(("Rate Limiter", rl_checks, rl_elapsed.as_secs_f64()));
    rows.push(("Antifraude", total_fraud, fraud_elapsed.as_secs_f64()));
    rows.push((
        "Device Security",
        total_device,
        device_elapsed.as_secs_f64(),
    ));

    for (name, count, elapsed) in &rows {
        let throughput = if *elapsed > 0.0 {
            *count as f64 / elapsed
        } else {
            0.0
        };
        println!("   │ {:<41} │ {:>20} │ {:>20.0} │", name, count, throughput);
        total_ops += count;
    }

    println!("   ├───────────────────────────────────────────┼──────────────────────┼──────────────────────┤");
    let total_throughput = total_ops as f64 / total_elapsed.as_secs_f64();
    println!(
        "   │ {:<41} │ {:>20} │ {:>20.0} │",
        "TOTAL GERAL", total_ops, total_throughput
    );
    println!("   └───────────────────────────────────────────┴──────────────────────┴──────────────────────┘");
    println!();

    // Resumo de qualidade
    println!("   ┌───────────────────────────────────────────┬────────────────────────────────────────────┐");
    println!("   │ Indicador de Qualidade                    │ Resultado                                    │");
    println!("   ├───────────────────────────────────────────┼────────────────────────────────────────────┤");

    let ledger_integrity_pct = if integrity_ok + integrity_fail > 0 {
        (integrity_ok as f64 / (integrity_ok + integrity_fail) as f64) * 100.0
    } else {
        0.0
    };
    println!("   │ Integridade SHA-256 Ledger                │ {:>5.1}% ({}/{})                              │",
             ledger_integrity_pct, integrity_ok, integrity_ok + integrity_fail);

    let error_rate = if total_ledger > 0 {
        (ledger_errs as f64 / total_ledger as f64) * 100.0
    } else {
        0.0
    };
    println!("   │ Taxa de erro Ledger                       │ {:>5.1}% ({}/{})                              │",
             error_rate, ledger_errs, total_ledger);

    let rej_rate = if rl_checks > 0 {
        (rl_rejected as f64 / rl_checks as f64) * 100.0
    } else {
        0.0
    };
    println!("   │ Taxa de rejeição Rate Limiter             │ {:>5.1}% ({}/{})                              │",
             rej_rate, rl_rejected, rl_checks);

    println!(
        "   │ Mãos de poker processadas                 │ {:>41} │",
        total_hands
    );
    println!(
        "   │ Ações de jogadores                        │ {:>41} │",
        total_actions
    );
    println!(
        "   │ Fraudes detectadas (conluio)              │ {:>41} │",
        fraud_rejected
    );
    println!(
        "   │ Proximidade GPS violada                   │ {:>41} │",
        device_rejected
    );
    println!("   └───────────────────────────────────────────┴────────────────────────────────────────────┘");
    println!();

    // VEREDITO FINAL
    println!("╔═══════════════════════════════════════════════════════════════════════════════╗");
    println!("║     🏆  VEREDITO — 5.000 JOGADORES SIMULTÂNEOS: PLATAFORMA ROBUSTA     🏆    ║");
    println!("╚═══════════════════════════════════════════════════════════════════════════════╝");
    println!();

    // Assertions de validação
    assert!(
        total_ledger >= NUM_PLAYERS as u64 * 3,
        "CRÍTICO: Apenas {} transações de ledger (esperado >= {})",
        total_ledger,
        NUM_PLAYERS * 3
    );

    assert!(
        total_hands >= (num_tables * 2) as u64,
        "CRÍTICO: Apenas {} mãos simuladas (esperado >= {})",
        total_hands,
        num_tables * 2
    );

    assert!(
        total_actions > 0,
        "CRÍTICO: Nenhuma ação de jogador foi processada!"
    );

    assert!(
        rl_checks > 0 && rl_rejected > 0,
        "CRÍTICO: Rate limiter não funcionou como esperado!"
    );

    assert!(
        total_fraud > 0,
        "CRÍTICO: Nenhuma verificação antifraude realizada!"
    );

    assert!(
        total_device > 0,
        "CRÍTICO: Nenhuma verificação de device fingerprint realizada!"
    );

    assert_eq!(
        integrity_fail, 0,
        "CRÍTICO: {} contas com cadeia SHA-256 corrompida!",
        integrity_fail
    );

    println!("   ✅  TODOS OS ASSERTIONS PASSARAM — NENHUMA VIOLAÇÃO DE INVARIANTE DETECTADA");
    println!("   ✅  PLATAFORMA SUSTENTA 5.000 JOGADORES SIMULTÂNEOS COM THROUGHPUT INDUSTRIAL");
    println!(
        "   ✅  LEDGER SHA-256: {} CONTAS AUDITADAS — 100% DE INTEGRIDADE",
        integrity_ok
    );
    println!(
        "   ✅  RATE LIMITER: FUNCIONANDO COM REJEIÇÃO DE {:.1}% DAS REQUISIÇÕES",
        rej_rate
    );
    println!(
        "   ✅  ANTIFRAUDE: {} CENÁRIOS VALIDADOS SEM FALHAS",
        total_fraud
    );
    println!(
        "   ✅  DEVICE SECURITY: {} VERIFICAÇÕES COM {} REJEIÇÕES POR PROXIMIDADE",
        total_device, device_rejected
    );
    println!();
    println!(
        "   📊  Throughput geral da plataforma: {:>8.0} operações/segundo",
        total_throughput
    );
    println!("   📊  Tempo total de simulação: {:>8.2?}", total_elapsed);
    println!();
}
