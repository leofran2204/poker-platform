# Frontend-Web — Zero Tilt Poker

Interface do jogador e painel B2B em **TypeScript + React + Vite + Tailwind CSS**.

## Direção visual

- Inspiração **Full Tilt** clássico: feltro verde, rail dourado, lobby tabular denso
- Acabamento **moderno** (espaçamento, tipografia Segoe/Tahoma, botões sólidos)
- Evitar estética “IA genérica” (glassmorphism excessivo, gradientes neon, emojis de marketing)

## Stack

| Peça | Tecnologia |
|------|------------|
| UI | React 19 |
| Linguagem | TypeScript |
| Build | Vite 8 |
| Estilo | Tailwind 3 + CSS de componentes `.zt-*` |
| Rotas | React Router 8 (`react-router`) |
| Presença | `components/OnlinePresence.tsx` → `/api/presence/*` |

## Presença online

- **Header:** badge `N online` (visitante via GET público; logado via heartbeat)
- **Home:** apresentação editorial, desafio de pot odds e acesso às demonstrações sob demanda
- Logado: `POST /api/presence/heartbeat` periódico
- Visitante: `GET /api/presence/online`

## Marca

- Apelido **ZT Poker**. Header: ás + **Zero Tilt**. Slogan: **Estude. Jogue. Sem tilt.**
- Símbolo: `public/brand/mark.jpg` (ás com ZeroTilt / Poker). Favicon e Open Graph usam o mesmo arquivo.

## Home

- Visitante: hero editorial, filme v3 de 81,70 s, desafio de pot odds, Academy, quatro modalidades e Agente ZT. Brazilian Pineapple aparece como modalidade exclusiva; Short Deck tem regras próprias.
- Logado em `/` redireciona para `/curso`
- Notícias e Dica do Pró **não** ficam na home: `/noticias`, `/dicas`, rodapé e atalhos no Curso
- Header logado: Curso · Lobby · Carteira (+ Mais). Visitante: Academy · Entrar · Criar conta

## Academy e mídia

- Grade local: 28 aulas e 65 questões em `src/data/courseContent.json`; pré-requisitos explícitos, modalidades após fundamentos, história opcional com a difusão do Omaha. IDs/progresso anteriores preservados.
- Player: MP4, poster, VTT em português, capítulos e transcrição real. Estado do player e quiz reinicia ao mudar de aula.
- As 28 aulas atuais têm vídeo publicado localmente. Os 25 técnicos v3 derivam da teoria, dos exercícios e dos cenários atuais; o gate de mídia confere SHA-256 do conteúdo e dos arquivos. A pasta pública contém somente os arquivos da grade ativa e do filme atual.
- Home v3, história ep11–ep13 v2 e 25 aulas técnicas v3: 1080p30, narração sintética, fontes bibliográficas vinculadas. Originais e renderizador em `ZeroTiltCurso/editorial/`; resultados em `public/videos/`.
- `homeFilm.json` é gerado com duração/transcrição/capítulos reais; não duplicar o roteiro no JSX.
- O quiz de aula usa gabarito editorial local. A mesa de estudo avalia regras e contas no servidor, sem certificar estratégia ótima.
- `/curso/mesa/:moduleId`: dez mesas de módulo, 35 cenários nas quatro modalidades, bots com informação limitada, treino guiado/desafio/prática variada, replay, retorno de uma decisão e comparação. Caderno e até 50 sessões persistem apenas neste navegador; permitem reabrir e exportar JSON. Não há depósito nem vínculo com saldo.
- Cliente em `src/lib/academy.ts`; catálogo único em `src/data/courseTraining.json`, também compilado na API. Alterar regras/cenários exige revisar versão, testes e vídeos afetados. Testes de persistência/contrato em `academy.test.ts`.
- Testes de integridade em `src/lib/courseAudit.test.ts`. Conteúdo pedagógico e fontes: [documento do curso](../Documentacao/CURSO_ESTRATEGIA_POKER.md).

## Notícias e dicas

- Componente `NewsTips` com props `tab` (trava aba), `compact` (prévia local sem rede) e `previewLimit`
- Páginas dedicadas `/noticias` e `/dicas` (lazy no `App.tsx`, links no nav)
- Aba **Notícias**: RSS multi-fonte; capa = thumbnail/og oficial **ou** fallback temático (sem rostos repetidos/errados)
- Aba **Dica do Pró**: teoria por street derivada das aulas revisadas em `src/data/courseContent.json`, sem cópia editorial separada

## Lobby e carteira

- Header: toggle **Play Money** / **Jogo Real** (`walletMode`); no PM mostra cash e MTT
- Cash: filtros alinhados ao catálogo em [`../Documentacao/STATUS_OPERACIONAL.md`](../Documentacao/STATUS_OPERACIONAL.md)
- Badges: Tradicional / Short Deck + X-max
- Torneios: lista por modo; buy-in + taxa 15%; cancelar inscrição
- Home pública: hero editorial e presença no header (GET público)

## Desenvolvimento local

```bash
cd Frontend-Web
npm ci
npm run dev
npm test
npm run lint
```

Admin, Academy, mesa, torneio e `NewsTips` entram via `React.lazy`; replays e deflator da home carregam sob demanda. Links de notícia/história só `http(s)`.

Proxy Vite encaminha `/api` e `/ws` para `http://127.0.0.1:3000` (API Axum). Para a prévia isolada de estudo, `POKER_DEV_API_URL=http://127.0.0.1:3188` troca somente o destino de `/api`; `academy_preview` e comandos estão em `scripts/README.md`. Essa prévia não fornece login, carteira, lobby ou presença.

## Produção (Docker)

O serviço `poker_frontend` no compose usa este Dockerfile: build Node → Caddy com o mesmo `Caddyfile` (HTTPS + reverse_proxy).

## Legado

O antigo `Frontend-Dioxus/` (WASM) foi **removido** do monorepo. O deploy canônico é **Frontend-Web**.
