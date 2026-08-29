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

## Notícias e dicas (home)

- Componente `NewsTips` abaixo do painel principal
- Aba **Notícias**: RSS multi-fonte; capa = thumbnail/og oficial **ou** fallback temático (sem rostos repetidos/errados)
- Aba **Jogando melhor**: tips por street (`src/data/tipsContent.json`)

## Lobby e carteira

- Header: toggle **Play Money** / **Jogo Real** (`walletMode`)
- Cash: filtros NL 0,25/0,25 · NL 0,25/0,50 · SD 0,50 · SD Omaha 0,50/1
- Badges: NLHE / Short Deck / SD Omaha
- Torneios: lista por modo; badge de variante

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

O antigo `Frontend-Dioxus/` (WASM) foi **removido** do monorepo. O deploy canônico é **Frontend-Web**.
