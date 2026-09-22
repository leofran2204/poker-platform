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

## Marca

- Apelido **ZT Poker**. Header: ás + **Zero Tilt**. Slogan: **Estude. Jogue. Sem tilt.**
- Símbolo: `public/brand/mark.jpg` (ás com ZeroTilt / Poker). Favicon e Open Graph usam o mesmo arquivo.

## Home

- Visitante: vitrine da **Academy** + `ShowcaseTable` + deflator + três jogos (Hold’em, Omaha 4, Brazilian Pineapple)
- Logado em `/` redireciona para `/curso`
- Notícias e Dica do Pró **não** ficam na home: `/noticias`, `/dicas`, rodapé e atalhos no Curso
- Header logado: Curso · Lobby · Carteira (+ Mais). Visitante: Academy · Entrar · Criar conta

## Vídeos da Academy

- Player (`CourseVideoPlayer`): MP4 + poster + transcrição da narração em texto. Sem faixa de legenda.
- Finais em `public/videos/` (`epNN-*.mp4` / `.jpg`). Piloto ep01 em 720p30; os demais ainda 480p15 até o lote.
- Voz: Hold’em = **Rôldem**, Omaha = **Omárra** (ep12, ep23–ep25). Os MP4 de ep23–ep25 ainda descrevem o jogo antigo; o texto do Módulo 5 é a regra vigente.
- Fontes Manim em `ZeroTiltCurso/epNN-*/`.

## Notícias e dicas

- Componente `NewsTips` com props `tab` (trava aba), `compact` (prévia local sem rede) e `previewLimit`
- Páginas dedicadas `/noticias` e `/dicas` (lazy no `App.tsx`, links no nav)
- Aba **Notícias**: RSS multi-fonte; capa = thumbnail/og oficial **ou** fallback temático (sem rostos repetidos/errados)
- Aba **Dica do Pró**: tips por street (`src/data/tipsContent.json`)

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
