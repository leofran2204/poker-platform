# scripts/

Automação operacional do monorepo.

## Canônicos (usar)

| Script | Uso |
|--------|-----|
| `live-e2e-ten-users.mjs` | Smoke demo: 10 users / 100 hands + settlement assinado (`ALLOW_TEMP_MAIL=true`) |
| `full-validation.ps1` / `.sh` | Lote de validação autorizada (motor/API/frontend) |
| `deploy.ps1` / `deploy.sh` | Deploy assistido |
| `verify-public-https.sh` | Checagem HTTPS/Caddy público |
| `vps-redeploy-frontend.sh` | Redeploy rápido do frontend na VPS |
| `coverage.ps1` / `.sh` | Cobertura (quando autorizado) |

## Legado Dioxus / WASM (não usar no deploy canônico)

| Script | Nota |
|--------|------|
| `cargo-dioxus.ps1` | Wrapper cargo no `Frontend-Dioxus` (Windows) |
| `build-frontend-dist.sh` | Gera dist WASM a partir de `.wasm` |
| `rebuild-frontend-dist.sh` | Build completo WASM + wasm-bindgen |
| `install-wasm-bindgen-and-dist.sh` | Instala wasm-bindgen-cli pinado |

UI canônica: **`Frontend-Web/`** (`npm run build` / Docker).
