# Guia de Exemplos Práticos — Upgrades de Segurança e Arquitetura Enterprise

**Atualizado:** 2026-08-01 | **Status:** Guia técnico; validação contínua, sem alegação de certificação de produção.


Este documento é o guia prático e educacional para a utilização dos novos recursos de alta segurança adicionados ao projeto **Poker_Project**:
1. **Sistema Provably Fair** (Baralho auditável por HMAC-SHA256)
2. **Autenticação JWT HMAC-SHA256** (tokens curtos, refresh e MFA/TOTP)
3. **Protocolo Binário WebSocket** (Codec de baixa latência)
4. **Outbox Pattern & Audit Logs** (Consistência financeira e antifraude SQL)
5. **Telemetria Estruturada** (Spans de auditoria com OpenTelemetry)

---

## 1. 🃏 Sistema Provably Fair (Baralho Auditável)

### 📐 Conceito:
No início de cada mão, o servidor gera uma semente secreta (`Server Seed`) e publica apenas o seu hash SHA-256 (`Server Hash`). O jogador envia sua própria semente (`Client Seed`). O embaralhamento é 100% determinístico e auditável.

### 💻 Exemplo em Rust (`Motor-Rust`):

```rust
use poker_engine::provably_fair::{
    ProvablyFairState, provably_fair_shuffle, verify_shuffle, hash_server_seed_bytes
};

fn main() {
    // 1. O Servidor inicializa o estado no início da mão
    let client_seed = "minha_semente_aleatoria_player_123";
    let nonce = 1; // 1ª mão da mesa
    let mut pf_state = ProvablyFairState::new(client_seed, nonce);

    println!("📢 Server Hash Público publicado na mesa: {}", pf_state.server_hash);

    // 2. Embaralhamento do Baralho (52 cartas)
    let original_deck: Vec<u8> = (0..52).collect();
    let mut deck = original_deck.clone();
    
    let server_seed_bytes = hex::decode(&pf_state.server_seed_hex).unwrap();
    provably_fair_shuffle(&mut deck, &server_seed_bytes, client_seed, nonce).unwrap();

    println!("🎴 Baralho embaralhado (primeira carta): {}", deck[0]);

    // 3. Pós-Mão: O servidor revela a Server Seed secreta em texto claro
    println!("🔓 Server Seed Revelada pós-jogo: {}", pf_state.server_seed_hex);

    // 4. Auditoria pelo Cliente/Jogador
    let eh_valido = verify_shuffle(
        &original_deck,
        &deck,
        &pf_state.server_seed_hex,
        client_seed,
        nonce,
    ).unwrap();

    assert!(eh_valido);
    println!("✅ Baralho verificado pelo jogador: 100% Autêntico e Sem Manipulação!");
}
```

### 🦀 Exemplo de Auditoria no Cliente em 100% Rust (`Frontend-Dioxus` / WASM):

No frontend da plataforma, a auditoria é feita inteiramente em Rust nativo compondo um componente de interface Dioxus:

```rust
use dioxus::prelude::*;
use poker_engine::provably_fair::verify_shuffle;

#[component]
pub fn ProvablyFairAuditModal(
    server_seed: String,
    client_seed: String,
    nonce: u64,
    final_deck: Vec<u8>,
) -> Element {
    let mut audit_result = use_signal(|| None::<bool>);

    let handle_audit = move |_| {
        let original_deck: Vec<u8> = (0..52).collect();
        let is_valid = verify_shuffle(
            &original_deck,
            &final_deck,
            &server_seed,
            &client_seed,
            nonce,
        )
        .unwrap_or(false);

        audit_result.set(Some(is_valid));
    };

    rsx! {
        div { class: "audit-modal",
            h3 { "🔍 Auditoria Criptográfica de Mão (Provably Fair)" }
            p { "Server Seed: {server_seed}" }
            p { "Client Seed: {client_seed}" }
            p { "Nonce: {nonce}" }

            button { onclick: handle_audit, "Verificar Integridade da Mão" }

            if let Some(valid) = *audit_result.read() {
                if valid {
                    div { class: "alert-success", "✅ Baralho 100% Autêntico e Auditado em Rust WASM!" }
                } else {
                    div { class: "alert-danger", "🚨 Alerta: Semente ou Baralho Incompatível!" }
                }
            }
        }
    }
}
```

---

## 2. 🔑 Autenticação PASETO v4 (`API-Axum`)

### 💻 Exemplo em Rust (`API-Axum`):

```rust
use poker_api::auth_paseto::{PasetoClaims, encode_paseto, decode_paseto};

fn main() {
    let secret_key = [42u8; 32]; // Chave de 256 bits

    // 1. Criar Claims para o jogador (Validade de 1 hora = 3600s)
    let claims = PasetoClaims::new("usr_99812", "jogador_pro", "player", 3600);

    // 2. Gerar Token PASETO v4.local
    let token = encode_paseto(&claims, &secret_key).unwrap();
    println!("🔐 Token PASETO Gerado: {token}");
    // Ex: v4.local.eyJzdWIiOiJ1c3JfOTk4MTIiLCJ1c2VybmFtZSI6...

    // 3. Validar e Decodificar Token
    let claims_decodificados = decode_paseto(&token, &secret_key).unwrap();
    println!("👤 Usuário Autenticado: {}", claims_decodificados.username);
}
```

### 📡 Exemplo de Extrator no Handler Axum:

```rust
use axum::{routing::get, Router};
use poker_api::auth_paseto::PasetoClaims;

async fn get_user_profile(claims: PasetoClaims) -> String {
    format!("Bem-vindo ao lobby, jogador {} (ID: {})!", claims.username, claims.sub)
}
```

---

## 3. 🚀 Protocolo Binário WebSocket (`API-Axum` / `Frontend-Dioxus`)

### 💻 Exemplo de Codificação e Decodificação de Pacote:

```rust
use poker_api::binary_codec::{BinaryPacket, BinaryOpcode};

fn main() {
    // 1. Criar ação de aposta em formato binário
    let payload = b"RAISE:5000".to_vec();
    let packet = BinaryPacket::new(BinaryOpcode::PlayerAction, payload);

    // 2. Serializar para transmissão WebSocket (Opcode + Length + Payload)
    let binary_frame: Vec<u8> = packet.encode();
    println!("📦 Pacote Binário Enviado pelo WebSocket (bytes): {:?}", binary_frame);

    // 3. Decodificar no servidor
    let decoded_packet = BinaryPacket::decode(&binary_frame).unwrap();
    assert_eq!(decoded_packet.opcode, BinaryOpcode::PlayerAction as u8);
    println!("📩 Ação Recebida: {:?}", String::from_utf8(decoded_packet.payload).unwrap());
}
```

---

## 4. 💳 Outbox Pattern & Audit Trail (Migration 003)

### 🗄️ Exemplo SQL de Transação de Saque com Registro de Auditoria:

```sql
BEGIN;

-- 1. Deduzir o saldo do jogador
UPDATE users SET balance = balance - 100.00 WHERE id = 'usr_99812';

-- 2. Registrar o evento no Outbox (Transactional Outbox Pattern)
INSERT INTO outbox_events (aggregate_type, aggregate_id, event_type, payload)
VALUES (
    'USER_BALANCE',
    'usr_99812',
    'PIX_WITHDRAW_REQUESTED',
    '{"user_id": "usr_99812", "amount": 100.00, "pix_key": "12345678900"}'::jsonb
);

-- 3. Registrar a trilha de auditoria imutável (Audit Log)
INSERT INTO audit_logs (user_id, action, ip_address, metadata)
VALUES (
    'usr_99812',
    'WITHDRAW_PIX',
    '203.0.113.195',
    '{"amount": 100.00, "device": "Mobile Dioxus"}'::jsonb
);

COMMIT;
```

---

## 5. 🔍 Telemetria Estruturada (`telemetry.rs`)

### 💻 Exemplo de Uso de Spans de Auditoria com Tracing:

```rust
use poker_api::audit_span;

fn process_table_bet(user_id: &str, table_id: &str, amount: u64) {
    let span = audit_span!(user_id, "BET_RAISE");
    let _guard = span.enter();

    tracing::info!(
        table_id = %table_id,
        amount = %amount,
        "Processando aposta na mesa com rastreamento estendido."
    );
}
```

<!-- DOCUMENTATION_SYNC:START -->
> **Estado operacional sincronizado (2026-09-10):** S23 — cancela inscrição com reembolso total + admin agenda e cria torneios + textos da TournamentPage **Sem certificação de produção; o código rejeita PIX em modo production. Deploy público: VPS Hostinger (demo/staging) com domínio zerotiltpoker.net. Staging/demo apenas; não alegar Launch Ready de produção.** Stack Docker local 4/4 healthy e VPS Hostinger 4/4 healthy. Migrations 001–049 na VPS (045 convite/fila, 046 ledger estrutura, 047 bots, 048 3 mesas + fee ledger, 049 total_fees). PM duas carteiras R$150 sem rebuy (ilimitado com saldo). Motor 1848 lib (fee 15%, seating 3 mesas, run-out all-in) + API 43 lib + ator MTT 2 testes integração PASS. VPS: 1º MTT fim a fim (freeroll 6 inscritos, 3 mesas, 5 mãos assinadas, campeão + payout GTD). Bots lag_v2 em MTT (12 inscritos, 43 mãos assinadas, zero erros). Lobby GET /api/lobby/tables lista mesas OPEN mesmo lotadas com X-max sempre. MTT: inscrição + 3 mesas + gameplay WS ao vivo (mesmo protocolo do cash) + rebalance/consolidação FT + payouts; gameplay_ready=true. Health público OK. Diário de mãos + replay no frontend (2026-09-10): grava suas mãos no navegador (suas cartas, board, pote, resultado), replay passo a passo, download TXT/JSON, painel de resultado sem auto-fechar. Ritmo de digestão no frontend (2026-09-10): board com stagger de 220ms por carta + painel de resultado fixo do showdown (vencedor, mão e cartas reveladas, sem auto-fechar, sobrevive à mão seguinte). Bots da casa desligados na VPS (stop oficial, reembolso) a pedido. Ritual do crupiê no frontend (2026-09-10): banner de embaralhamento + cartas distribuídas por assento a partir do dealer, versos para os oponentes, stagger no board. Migration 053 (2026-09-10): cash Texas 9-max NL 0,75/1,50 frente 15000 em PM e Real + torneios Texas R$25 em PM e Real (buy-in 2500, stack 15000, 1 reentrada 2500/25000, agenda 21:30 SP, auto-start 5). Bots externos de estratégia validados no local em 2026-09-10: bot/strategy (ranges cash 6-max/9-max, MTT ChipEV/ICM/PKO, avaliador próprio 5-7 cartas, push/fold FT, pot odds) com tsc limpo + 13/13 selftest offline; scripts/strategy-bots.mjs jogou mesa real PM NL 0,25 (3 bots, 5-6 mãos cada, 5 mãos no hand_history, decisões por ranges/odds). Tabelas ICM versionadas em bot/strategy/icm/tables (geradas de final_table.ts/bubble.ts via generate.mjs). Recebedor manual: Leofran, chave 6eefcd53-686e-42d4-a062-03751336251c (PLAY_MONEY_PIX_KEY). Saque: informar chave Pix própria, recebimento em até 24h. A VPS permanece no padrão seguro PIX mock. DePix existe somente em Sandbox não produtivo, com chave sk_test_, allowlist de depositante, idempotência, HMAC com janela temporal, deduplicação de eventos e crédito apenas em checkout.completed. O CPF/CNPJ é encaminhado ao provedor sem persistência local. Depósito manual continua como fallback; não há saque automático. Mesas com dono único por processo (cash TableActor + torneio TournamentActor, mesmo protocolo); settlement assinado (HMAC) na liquidação; halt de mesa MTT auditável (MTT_TABLE_HALTED).
>
> Fonte canônica: [`STATUS_OPERACIONAL.json`](STATUS_OPERACIONAL.json). Verificação: `cargo run --bin documentation-sync -- --check`.
<!-- DOCUMENTATION_SYNC:END -->
