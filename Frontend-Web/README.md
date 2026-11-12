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
| Estilo | Tailwind 4.3 + PostCSS e CSS de componentes `.zt-*` |
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

- Visitante: o Brazilian Pineapple é o destaque (cartas desenhadas em código, link para a aula m5l3 e botão “Assistir ao trailer”). `PineappleDrawFilm.tsx` abre o trailer e o filme completo das regras num modal com abas, legendas e capítulos; a Academy mostra o mesmo componente como cartão do filme completo. Dados em `src/data/pineappleDrawFilms.json`, gerado por `ZeroTiltCurso/editorial/motion/produce.py --publish` (não editar à mão). O filme do Brazilian Pineapple e o institucional anterior foram retirados em 09/11/2026.
- Logado em `/` redireciona para `/curso`
- Notícias e Dica do Pró **não** ficam na home: `/noticias`, `/dicas`, rodapé e atalhos no Curso
- Header logado: Curso · Lobby · Carteira (+ Mais). Visitante: Academy · Entrar · Criar conta

## Academy e mídia

- Grade: 28 aulas e 65 questões em `src/data/courseContent.json`; pré-requisitos explícitos, modalidades após fundamentos, história opcional com a difusão do Omaha. IDs/progresso anteriores preservados.
- Player: MP4, poster, VTT em português, capítulos com indicação do atual e transcrição real. Carrega sob demanda, sem autoplay; saltos funcionam antes de carregar os metadados. O modal fecha por botão/Escape, desmonta o vídeo e devolve o foco. Estado do player e quiz reinicia ao mudar de aula.
- Pronúncia: perfil `pt-BR-poker-v1` no renderizador editorial (`render.py`), com grafias aproximadas dos termos ingleses; nos vídeos em Remotion, perfil `pt-BR-poker-v2-lang-en`, em que a voz lê os termos em inglês (escolha do proprietário em 11/11/2026). `narrationHash` versiona as URLs de mídia; texto, legenda e transcrição conservam a grafia original. O hash do conteúdo da aula continua separado da identidade da voz.
- Vinte e oito aulas têm vídeo publicado; m5l3–m5l5 (Brazilian Pineapple) vêm do renderizador em Remotion (`renderer: "remotion"`). A pasta pública contém 90 arquivos (MP4/VTT/WebP das 28 aulas, do trailer e do filme de regras), e a auditoria rejeita mídia sem referência.
- O quiz de aula usa gabarito editorial local. A mesa de estudo avalia regras e contas no servidor, sem certificar estratégia ótima.
- `/curso/mesa/:moduleId`: dez mesas de módulo, 35 cenários nas quatro modalidades, bots com informação limitada, treino guiado/desafio/prática variada, replay, retorno de uma decisão e comparação. Caderno e até 50 sessões persistem apenas neste navegador; permitem reabrir e exportar JSON. Não há depósito nem vínculo com saldo.
- Cliente em `src/lib/academy.ts`; catálogo único em `src/data/courseTraining.json`, também compilado na API. Alterar regras/cenários exige revisar versão, testes e vídeos afetados. Testes de persistência/contrato em `academy.test.ts`.
- Testes de integridade em `src/lib/courseAudit.test.ts`. Conteúdo pedagógico e fontes: [documento do curso](../Documentacao/CURSO_ESTRATEGIA_POKER.md).

## Notícias e dicas

- Componente `NewsTips` com props `tab` (trava aba), `compact` (prévia local sem rede) e `previewLimit`
- Páginas dedicadas `/noticias` e `/dicas` (lazy no `App.tsx`, links no nav)
- Aba **Notícias**: RSS multi-fonte; capa = thumbnail/og oficial **ou** fallback temático (sem rostos repetidos/errados)
- Aba **Dica do Pró**: teoria por street derivada das aulas revisadas em `src/data/courseContent.json`, sem cópia editorial separada; conteúdo local visível enquanto os feeds externos carregam

## Lobby e carteira

- Header: toggle **Play Money** / **Jogo Real** (`walletMode`); no PM mostra cash e MTT
- Cash: filtros alinhados ao catálogo em [`../Documentacao/STATUS_OPERACIONAL.md`](../Documentacao/STATUS_OPERACIONAL.md)
- Badges: Tradicional / Short Deck + X-max
- Torneios: lista por modo; buy-in + taxa 15%; cancelar inscrição
- Carteira (`/wallet`): saque Real só com verificação aprovada e para chave PIX do mesmo CPF do KYC; o formulário mostra o final do CPF verificado, valida o CPF antes do envio, envia `tax_number` quando a chave não é CPF e traduz os erros da API (`src/lib/pixWithdraw.ts`)
- Admin, visão geral: passivo com jogadores no Jogo Real (carteiras, fichas em cash, prêmios de MTT abertos, saques pendentes, total devido e crédito provisório à parte), comparado só na tela com o saldo informado da conta recebedora (`src/lib/liabilities.ts`, `GET /api/admin/liabilities`)
- Home pública: hero editorial e presença no header (GET público)

## Desenvolvimento local

Tema em `src/index.css` (`@theme`), com cores, alturas de linha e camada de componentes preservadas na migração para Tailwind 4. O suporte de CSS segue [os requisitos do Tailwind 4](https://tailwindcss.com/docs/upgrade-guide#browser-requirements): Safari 16.4+, Chrome 111+ e Firefox 128+. `npm run audit:security` executa a auditoria npm integral, sem exceções. `braces` e sua cadeia antiga de build foram removidos.

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

A mesa e a Academy exibem ante nominal, valor pago e pagador. Todos os seletores seguem os limites enviados pelo motor. URLs dos players usam `mediaRevision` (MP4 + VTT + capa); reprodução continua sob demanda, sem autoplay, com transcrição, legendas e capítulos. O catálogo de treino é v3.
