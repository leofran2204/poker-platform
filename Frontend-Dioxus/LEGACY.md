# Frontend-Dioxus — LEGADO (não usar em deploy)

Este frontend **WASM/Dioxus** **não** é o deploy canônico e **não** recebe features novas (presence, settlements UI, etc.).

| | |
|--|--|
| **Canônico** | `Frontend-Web/` (TypeScript + React + Vite + Tailwind) |
| **Desde** | 2026-08-04 (stack v4.0) |
| **Compose** | `poker_frontend` → `Frontend-Web/Dockerfile` apenas |
| **CI** | Ainda há job de check WASM histórico em `rust-ci.yml` — não bloqueia o produto demo |
| **Scripts legados** | `scripts/cargo-dioxus.ps1`, `build-frontend-dist.sh`, `rebuild-frontend-dist.sh`, `install-wasm-bindgen-and-dist.sh` |

Mantido no repositório como **arquivo histórico**. Preferir deletar/arquivar fora do monorepo em ciclo futuro se o CI WASM for aposentado.
