# scripts/

Automação operacional do monorepo.

## Canônicos (usar)

| Script | Uso |
|--------|-----|
| `live-e2e-ten-users.mjs` | Smoke demo: 10 users / 100 hands + settlement assinado (`ALLOW_TEMP_MAIL=true`) |
| `live-e2e-seeded-catalog.mjs` | Smoke mesa a mesa (Real ou Play): login seed → join → ≥1 mão → leave; torneios |
| `live-e2e-real-catalog.mjs` | Variante com Mail.tm + crédito admin opcional (`ADMIN_TOKEN`) |
| `full-validation.ps1` / `.sh` | Lote de validação autorizada (motor/API/gateway) |
| `deploy.ps1` / `deploy.sh` | Deploy assistido |
| `verify-public-https.sh` | Checagem HTTPS/Caddy público |
| `vps-redeploy-frontend.sh` | Redeploy na VPS (`REBUILD_API=1` para API+migration) |
| `coverage.ps1` / `.sh` | Cobertura (quando autorizado) |

## Stress do motor (não são scripts shell)

| Teste | Uso |
|-------|-----|
| `Motor-Rust/tests/cash_catalog_10k_hands.rs` | 10k mãos × NLHE / SD / SD Omaha (catálogo oficial) |
| `Motor-Rust/tests/short_deck_massive.rs` | Regras SD + 1M evals + 100k mãos 6-max |
| `cargo test --features massive-tests …` | Fuzz/fairness/stress gated |

Exemplo Docker (Windows sem toolchain GNU):

```bash
docker run --rm -v "$PWD":/app -w /app/Motor-Rust rust:1.97.0-bookworm \
  cargo test --test cash_catalog_10k_hands -- --nocapture
```

## Seeded catalog e2e

```bash
# Contas e2ecat01/02 com saldo Real (criar via SQL na VPS se necessário)
MODE=real HANDS_PER_TABLE=1 node scripts/live-e2e-seeded-catalog.mjs
MODE=play HANDS_PER_TABLE=1 node scripts/live-e2e-seeded-catalog.mjs
```

UI canônica: **`Frontend-Web/`** (`npm run build` / Docker). O antigo `Frontend-Dioxus/` foi removido do monorepo.

## DePix Sandbox local

`install-depix-local-secrets.ps1` solicita a chave `sk_test_` e o webhook secret sem ecoá-los, valida a chave em `https://api.depixapp.com/api/me` e grava somente em `Infraestrutura-Docker/.env`, ignorado pelo Git. Use `-AllowedDepositorId <UUID>` para limitar quem pode criar/simular cobranças. Não use esse instalador na VPS pública.
`install-depix-vps-live-secrets.ps1` valida uma chave `sk_live_` em `/api/me`, exige conta verificada, escopos confirmados `merchant_read`/`merchant_write`, allow-list de UUIDs e limite por depósito. Ele baixa o `.env` remoto por `scp`, cria backup datado, instala a nova versão com permissão `600` e nunca envia segredo por pipe ou argumento de shell. Use primeiro sem `-Apply` para validar; a instalação efetiva exige `-Apply`.