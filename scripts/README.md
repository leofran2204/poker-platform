# scripts/

Automação operacional do monorepo.

## Canônicos (usar)

| Script | Uso |
|--------|-----|
| `live-e2e-ten-users.mjs` | Smoke demo: 10 users / 100 hands + settlement assinado (`ALLOW_TEMP_MAIL=true`) |
| `full-validation.ps1` / `.sh` | Lote de validação autorizada (motor/API/gateway; fase frontend = no-op) |
| `deploy.ps1` / `deploy.sh` | Deploy assistido |
| `verify-public-https.sh` | Checagem HTTPS/Caddy público |
| `vps-redeploy-frontend.sh` | Redeploy rápido do frontend na VPS |
| `coverage.ps1` / `.sh` | Cobertura (quando autorizado) |

UI canônica: **`Frontend-Web/`** (`npm run build` / Docker). O antigo `Frontend-Dioxus/` e scripts WASM foram removidos do monorepo (permanecem no histórico git).
