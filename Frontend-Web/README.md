# Frontend-Web — Zero Tilt Poker

Interface do jogador e painel B2B em **TypeScript + React + Vite + Tailwind CSS**.

## Direção visual

- Inspiração **Full Tilt** clássico: feltro verde, rail dourado, lobby tabular denso
- Acabamento **moderno** (espaçamento, tipografia Segoe/Tahoma, botões sólidos)
- Evitar estética “IA genérica” (glassmorphism excessivo, gradientes neon, emojis de marketing)

## Stack

| Peça | Tecnologia |
|------|------------|
| UI | React 18 |
| Linguagem | TypeScript |
| Build | Vite 5 |
| Estilo | Tailwind 3 + CSS de componentes `.zt-*` |
| Rotas | react-router-dom 6 |
| Presença | `components/OnlinePresence.tsx` → `/api/presence/*` |

## Presença online

- **Header:** badge `N online` (todas as rotas via `Layout`)
- **Home:** faixa hero com contagem e aviso de mín. 2 na mesa
- Logado: `POST /api/presence/heartbeat` periódico
- Visitante: `GET /api/presence/online`

## Desenvolvimento local

```bash
cd Frontend-Web
npm install
npm run dev
```

Proxy Vite encaminha `/api` e `/ws` para `http://127.0.0.1:3000` (API Axum).

## Produção (Docker)

O serviço `poker_frontend` no compose usa este Dockerfile: build Node → Caddy com o mesmo `Caddyfile` (HTTPS + reverse_proxy).

## Legado

`Frontend-Dioxus/` permanece no repositório como **legado** (WASM). O deploy canônico é **Frontend-Web**.
