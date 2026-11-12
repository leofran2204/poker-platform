# Guia de Exemplos Práticos — Upgrades de Segurança e Arquitetura Enterprise

**Atualizado:** 2026-11-08 (exemplos conferidos contra o código) | **Status:** Guia técnico; validação contínua, sem alegação de certificação de produção.


Este documento é o guia prático e educacional dos recursos de segurança do projeto **Poker_Project**:
1. **Provably Fair** (módulo do motor com HMAC-SHA256; **ainda não ligado à mesa**)
2. **Autenticação JWT HMAC-SHA256** (tokens curtos, refresh, `token_version` e MFA/TOTP)
3. **Protocolo Binário WebSocket** (codec aceito pela API; o Frontend-Web usa JSON em texto)
4. **Outbox Pattern & Audit Logs** (consistência financeira em centavos)
5. **Telemetria Estruturada** (spans de auditoria com `tracing`)

---

## 1. 🃏 Provably Fair (módulo do motor)

> **Estado em 08/11/2026:** `Motor-Rust/src/provably_fair.rs` existe e tem testes, mas não é chamado pelo jogo. Cada mão real embaralha com Fisher-Yates + `OsRng` (`shuffle_deck` em `Motor-Rust/src/game_loop.rs`). Nenhum texto voltado ao jogador pode prometer verificação pública até a integração; ver Termos 7.3 e [`ARQUITETURA_E_APIS.md`](ARQUITETURA_E_APIS.md).

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

### Verificação no cliente

O exemplo antigo de modal em Dioxus/WASM foi retirado junto com o frontend Dioxus. Quando o Provably Fair for ligado à mesa, a verificação do jogador ficará no histórico da mão do `Frontend-Web` (TypeScript), usando a semente revelada e os opcodes `ProvablyFairHandStart`/`ProvablyFairHandEnd` (0x30/0x31) já definidos em `API-Axum/src/binary_codec.rs`.

---

## 2. 🔑 Autenticação JWT (`Motor-Rust/src/auth.rs` + `API-Axum`)

O exemplo de PASETO v4 da versão anterior não corresponde ao código: não existe módulo PASETO no repositório. A autenticação vigente usa JWT HMAC-SHA256 emitido por `AuthManager::issue_tokens_for_user` e validado por `validate_token` (`Motor-Rust/src/auth.rs`), com refresh, MFA/TOTP e `token_version` persistente, que revoga sessões após mudança sensível (senha, papel, suspensão).

### 📡 Exemplo de Extrator no Handler Axum:

```rust
use axum::Json;
use poker_api::middleware::auth::RequireAuth;

async fn perfil(RequireAuth(user): RequireAuth) -> Json<serde_json::Value> {
    // RequireAuth valida o Bearer JWT (assinatura, expiração e token_version).
    Json(serde_json::json!({ "user_id": user.user_id, "role": user.role }))
}
```

---

## 3. 🚀 Protocolo Binário WebSocket (`API-Axum`)

A API aceita quadros binários para `Ping` e `PlayerAction`, com carga JSON dentro do pacote. O Frontend-Web atual envia somente texto JSON.

### 💻 Exemplo de Codificação e Decodificação de Pacote:

```rust
use poker_api::binary_codec::{BinaryPacket, BinaryOpcode};

fn main() {
    // 1. Criar ação de aposta em formato binário (carga JSON, valor em centavos)
    let payload = br#"{"action":"raise","amount":5000}"#.to_vec();
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

## 4. 💳 Outbox Pattern & Audit Trail

### 🗄️ Exemplo SQL de Transação de Saque com Registro de Auditoria

Forma simplificada do que `POST /api/payments/pix/withdraw` executa em uma única transação (`API-Axum/src/payments_routes.rs`), depois de conferir KYC aprovado e o mesmo CPF do KYC. Valores em **centavos inteiros**; a chave PIX nunca aparece em claro: só a impressão SHA-256 e o texto cifrado (AES-256-GCM) são gravados.

```sql
BEGIN;

-- 1. Reservar o saldo Real sem permitir saldo negativo (R$ 100,00 = 10000 centavos)
UPDATE users SET balance_real = balance_real - 10000
WHERE id = $1::uuid AND balance_real >= 10000
RETURNING balance_real;

-- 2. Registrar o saque pendente com a chave só como impressão + cifra
INSERT INTO wallet_transactions
    (user_id, amount, transaction_type, status, idempotency_key, provider,
     pix_key_fingerprint, pix_key_ciphertext, provider_status)
VALUES ($1::uuid, 10000, 'WITHDRAW', 'PENDING', $2, 'depix', $3, $4, 'QUEUED');

-- 3. Evento no Outbox (Transactional Outbox Pattern), sem a chave em claro
INSERT INTO outbox_events (aggregate_type, aggregate_id, event_type, payload)
VALUES ('wallet_transaction', $2, 'PIX_WITHDRAWAL_REQUESTED',
        jsonb_build_object('tx_id', $2, 'amount_cents', 10000, 'pix_key_fingerprint', $3));

-- 4. Trilha de auditoria na mesma transação
INSERT INTO audit_logs (user_id, action, metadata)
VALUES ($1, 'PIX_WITHDRAWAL_RESERVED', jsonb_build_object('tx_id', $2, 'amount_cents', 10000));

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
> **S27** (2026-11-09) — demo `zerotiltpoker.net` · sem certificação de produção · PIX automático ligado (DePix reconciliado).
> Fatos (catálogo, carteiras, limites): [`STATUS_OPERACIONAL.md`](STATUS_OPERACIONAL.md).
<!-- DOCUMENTATION_SYNC:END -->
