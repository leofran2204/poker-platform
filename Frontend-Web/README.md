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
| Build | Vite 8 |
| Estilo | Tailwind 3 + CSS de componentes `.zt-*` |
| Rotas | react-router-dom 6 |
| Presença | `components/OnlinePresence.tsx` → `/api/presence/*` |

## Presença online

- **Header:** badge `N online` (visitante via GET público; logado via heartbeat)
- **Home:** mesa de vitrine + faixa com contagem e aviso de mín. 2 na mesa
- Logado: `POST /api/presence/heartbeat` periódico
- Visitante: `GET /api/presence/online`

## Notícias e dicas (home)

- Componente `NewsTips` abaixo do painel principal
- Aba **Notícias**: RSS multi-fonte; capa = thumbnail/og oficial **ou** fallback temático (sem rostos repetidos/errados)
- Aba **Jogando melhor**: tips por street (`src/data/tipsContent.json`)

## Lobby e carteira

- Header: toggle **Play Money** / **Jogo Real** (`walletMode`); no PM mostra cash e MTT
- Cash: filtros alinhados ao catálogo em [`../Documentacao/STATUS_OPERACIONAL.md`](../Documentacao/STATUS_OPERACIONAL.md)
- Badges: Tradicional / Short Deck + X-max
- Torneios: lista por modo; buy-in + taxa 15%; cancelar inscrição
- Home pública: hero da mesa + presença online (GET público)

## Desenvolvimento local

```bash
cd Frontend-Web
npm install
npm run dev
npm test
npm run lint
```

Admin e `NewsTips` entram via `React.lazy`. Links de notícia/história só `http(s)`.

Proxy Vite encaminha `/api` e `/ws` para `http://127.0.0.1:3000` (API Axum).

## Produção (Docker)

O serviço `poker_frontend` no compose usa este Dockerfile: build Node → Caddy com o mesmo `Caddyfile` (HTTPS + reverse_proxy).

## Legado

O antigo `Frontend-Dioxus/` (WASM) foi **removido** do monorepo. O deploy canônico é **Frontend-Web**.
