# Relatório para parceiros — Zero Tilt Poker

**Como usar este texto:** leia o sumário em voz alta (3 minutos). As seções 2–8 são a prova. A seção 9 é o que ainda não somos — deixe visível; esconder gap é o que queima due diligence. A seção 10 é o pedido. A seção 11 é o mapa de melhorias (tecnologia e experiência) para o próximo ciclo de investimento.

| | |
|--|--|
| **Produto** | Zero Tilt Poker |
| **Demo** | [https://zerotiltpoker.net](https://zerotiltpoker.net) |
| **Ciclo** | S20c (2026-09-01) — staging/demo, **sem** certificação de produção |
| **Stack** | Motor e API em **Rust**; UI em **TypeScript** (React + Vite + Tailwind); PostgreSQL 15; Redis 7; Caddy + HTTPS Let's Encrypt |
| **Repositório** | https://github.com/leofran2204/poker-platform |
| **Fonte dos números** | `Documentacao/STATUS_OPERACIONAL.json` |

---

## 0. Sumário executivo (o que dizer na primeira reunião)

Zero Tilt não é um clone de sala gigante. É uma plataforma de pôquer **recreacional**, com motor próprio, sentada em três ideias que o mercado mainstream abandonou:

1. **A mesa tem que ser honesta e rápida** — regras em Rust, dinheiro em centavos inteiros, baralho auditável, liquidação de mão assinada.
2. **O jogador não pode sair destroçado da sessão** — Loss Deflator (cashback de bad beat por matemática, não por “bônus de cassino”), frentes fixas, Play Money que renova, ensino no próprio lobby.
3. **O crescimento é de clube, não de anúncio** — rede de **dois níveis** (clube → agente), receita de **rake**, hoje ensaiada em Play Money; dinheiro real só com licença.

O que já está no ar: demo HTTPS, e-mail verificado, MFA, mesas Play Money e Jogo Real **isoladas**, três variantes (Hold’em, Short Deck, Omaha Short Deck), torneios com Big Blind Ante em 26 níveis, admin de clubes, stack Docker **4/4 healthy** na VPS.

O que **não** está: certificação de produção, PIX automático em produção, saque automático, autoexclusão de produto, multi-servidor de mesas. Isso não se esconde. O parceiro que entra agora compra **produto + liquidez Play Money + o trilho até 2027**, não um cassino “já legalizado”.

**Pedido:** capital e liquidez de um **clube âncora**. Em troca: até **85% do rake** da rede que esse clube trouxer, no mesmo grafo que já existe no banco. Enquanto a papelada não fecha, a operação pública da rede é **Play Money**.

---

## 1. Posicionamento — pôquer renovado, não cassino disfarçado

O jogador brasileiro de mesa cansou de duas coisas: sala fria (soft, bot, delay) e sala que trata pôquer como caça-níquel (bônus opaco, rake invisível, tilt como feature).

Zero Tilt escolhe o lado difícil:

| Escolha de produto | O que o jogador sente | O que o parceiro ganha |
|--------------------|----------------------|------------------------|
| Catálogo **curto** e frentes **fixas** | Não precisa “escolher stake até quebrar” | Liquidez concentrada (o único problema do pôquer) |
| **Short Deck** e **Omaha Short Deck** nativos no motor | Jogo mais rápido, mais showdown, menos fold-fest | Diferenciação vs. “só NL Hold’em 100bb” |
| Big Blind Ante (26 níveis) nos MTT | Torneio moderno, sem ante morto bagunçando pote | Formato que o jogador de 2020+ já espera |
| Skin Full Tilt, PT-BR de verdade | Nostalgia + clareza (não Google Translate) | Identidade; não é mais um skin cinza |
| Dica do Pró + história do pôquer (8 capítulos mundo + 7 Brasil) | A sala ensina em vez de só extrair | Recreacional fica; regular não se sente otário |
| Play Money ≠ Jogo Real | Treina sem misturar salário | Risco reputacional menor até a licença |

Catálogo cash vigente (Play Money e Jogo Real, mesas espelhadas, **não** misturáveis):

| Mesa | Variante | Blinds | Cap | Frente |
|------|----------|--------|-----|--------|
| Hold’em | 52 cartas, 2 hole | 0,25 / 0,25 | 9 | R$ 25 |
| Hold’em Short Deck | 36 cartas (sem 2–5); flush > full house; wheel A-6-7-8-9 | 0,25 / 0,50 | 6 | R$ 75 |
| Omaha Short Deck | 4 hole; showdown = 2 hole + 3 board | 0,50 / 0,50 | 4 | R$ 100 |

Short Deck no Zero Tilt **não é um rótulo no lobby**: o motor troca baralho e avaliador (`create_short_deck`, `evaluate_hand_short_deck`, `evaluate_hand_short_deck_omaha`). Isso é o tipo de detalhe que um parceiro técnico testa em cinco minutos — e que uma operação “de fachada” não tem.

---

## 2. Segurança — por que a casa não é um site de WordPress com fichas

Pôquer online quebra de três jeitos: **conta roubada**, **centavo errado**, **baralho suspeito**. A stack foi escolhida contra esses três.

### 2.1 Infraestrutura e acesso

- Tráfego público **só HTTPS** (Caddy + Let's Encrypt). API e SPA no **mesmo domínio** (menos ataque de cookie cruzado).
- WebSocket de mesa **não aceita o JWT cru**: o browser pede um **ticket de 60 segundos, uso único**, amarrado à mesa (Redis em produção).
- Autenticação: senha + **e-mail verificado** (Resend, domínio `zerotiltpoker.net` verified) + **MFA TOTP**. Mudança de senha, papel ou MFA **invalida tokens já emitidos** (`token_version`).
- Rate limit de login por IP (Redis atômico em produção).
- Admin B2B e carteira **não são indexados** (`robots.txt`, `noindex`).

### 2.2 Dinheiro como banco, não como jogo

- Tudo que é saldo, pote, rake, buy-in e blinds é **`u64` centavos inteiros** no backend. O frontend só formata `R$ x,xx` na tela. Não existe “R$ 10,50 + 0,1 virar 10,59999”.
- Join de mesa: débito, escrow, ledger e assento na **mesma transação**. Leave só **entre mãos**.
- Liquidação de mão **assinada (HMAC)** — o e2e ao vivo já conferiu assinatura + vencedor + (payouts + rake = pote).
- Carteira Real **não** se alimenta de Play Money. O modo da mesa tem que bater com o modo do cliente (`play` \| `real`).
- PIX automático em **production está rejeitado no código**. DePix existe só em **sandbox** (chave de teste, allowlist, idempotência, HMAC com janela, crédito **somente** em `checkout.completed`). Não há saque automático. CPF/CNPJ vai ao provedor **sem ficar gravado localmente**.

Isso é o contrário de “ liberamos PIX no grupo até legalizar”. O produto **recusa** o atalho que destruiria o parceiro.

### 2.3 Integridade da mesa

- Embaralhamento **Provably Fair**: o servidor publica o hash da semente, embaralha com ChaCha8 + HMAC-SHA256 (semente do servidor + semente do cliente + nonce), revela a semente no histórico. O jogador pode reconstruir o baralho.
- Antifraude em Rust, na mesa: **bots**, **chip dumping**, **colusão** (soft play / pares), **multi-conta**.
- **Guarda de sub-rede /24:** dois jogadores no mesmo IP ou na mesma /24 **não sentam na mesma mesa**.
- VPIP/PFR anômalos disparam revisão (amostra mínima; não é ban automático por “jogar loose”).
- Admin pode banir e **congelar** saldo no ledger.

### 2.4 O que segurança **ainda não** é

Não há pentest externo publicado, não há certificação PCI, não há auditoria de laboratório de jogo. Há engenharia sólida de **staging**. O parceiro que exigir selo de produção está certo — isso é exatamente o uso do capital da seção 10.

---

## 3. Confiabilidade — a mão precisa acabar igual para todo mundo

| Prova | Resultado (S20c) |
|-------|------------------|
| Stack Docker na VPS | **4/4 healthy** (`poker_api`, `poker_frontend`, `poker_postgres`, `poker_redis`) |
| Health público | `https://zerotiltpoker.net/api/health` **OK** (só responde após Postgres e Redis) |
| Motor | Gate com **Clippy estrito** + suíte grande do Motor-Rust (ordem de **1.828** testes no ciclo documentado) |
| API | Contratos PostgreSQL + dezenas de testes de API (login, MFA, lobby, DePix sandbox) |
| Frontend | `tsc -b` + Vite, **60 módulos / 324 KB** |
| Stress de catálogo | `cash_catalog_10k_hands` — 10 mil mãos por configuração |
| E2e ao vivo | `live-e2e-ten-users.mjs` (10 usuários × 100 mãos) com settlement verificado |
| Migrations | **001–032** aplicadas (inclui BBA) |
| Backup | Dump verificável na operação S20 |
| Rebuild | API ~**4 min 13 s**; frontend ~**18 s** |

Desconexões: a mesa **não** deixa pote órfão por “jogador caiu”. Há settle após disconnect e histórico com número sequencial atômico por mesa.

Limite honesto de arquitetura: **uma mesa tem um dono por processo**. Não vendemos “mil mesas em Kubernetes multi-pod” enquanto isso for verdade. Redis **não** transforma o ator em cluster. Quem promete isso sem ownership distribuído está mentindo — nós anotamos na seção 9.

---

## 4. Agilidade — a sessão não pode parecer 2006 com ping de 2010

- Motor e API em **Rust + Tokio + Axum**: o caminho quente (ação da vez → validação → broadcast) não passa por interpretador nem por ORM preguiçoso no flop.
- Timeouts de ação: **30 s / 15 s / 8 s** (normal / turbo / hyper). Estouro = fold. A mesa não espera o jogador “pensar no WhatsApp”.
- Ticket WS de **60 s** evita fila zumbi de conexões.
- Codec binário já existe na API (`ProvablyFairHandStart/End` e opcodes de jogo) — base para o próximo salto de latência percebida (seção 11).
- Frontend leve (324 KB de módulos no build do gate): o celular médio brasileiro abre o lobby sem baixar um jogo da Steam.

Agilidade de **produto** também é catálogo curto: três mesas oficiais, frentes fixas. O jogador escolhe variante, não um Excel de stakes. Liquidez aparece mais rápido — e liquidez **é** velocidade da casa.

---

## 5. Experiência ZeroTilt — o nome não é slogan

“Tilt” é o momento em que o jogador deixa de tomar decisão e passa a **punir a si mesmo**. Plataformas clássicas lucram com isso. Zero Tilt foi desenhada para **cortar o combustível**.

### 5.1 Na mão

**Loss Deflator** (já no motor, ordem financeira obrigatória: potes → rake → deflator → pagamentos):

| Equity do perdedor no instante do all-in pago | Devolução sobre o pote líquido em que ele estava |
|-----------------------------------------------|--------------------------------------------------|
| abaixo de 56% | 0% |
| 56% a 65,9% | 7% |
| 66% a 75,9% | 15% |
| 76% a 85,9% | 25% |
| 86% ou mais | 35% |

Não é “bônus da casa”. Não é “rakeback escondido”. É matemática de **bad beat**: quem estava ganhando no all-in e perdeu no milagre leva um colchão. A fase da mão **não** escolhe o percentual — só reconstrói o board conhecido. Nos Termos, o benefício no estágio atual opera em **play money**. O mecanismo está pronto para a política do real, com licença.

### 5.2 Na sessão

- Play Money **renova todo dia** (R$ 1.000 cash / R$ 15.000 MTT). Perdeu a noite, não perdeu o mês.
- Frentes **fixas** (min = max). Não existe “entrar com R$ 25 e rebuy até o aluguel”.
- Isolamento Real / PM: treinar não vira depósito por acidente.
- Precisa de **≥ 2 pessoas** na mesa para iniciar mão — a casa não simula oponente fantasma para o jogador se iludir.

### 5.3 Na cabeça (ensino, não extração)

- **Dica do Pró:** estratégia em PT-BR (feeds filtrados + conteúdo local; notícia de resultado **não** entra como “dica”).
- **História do pôquer** no próprio layout da mesa (8 capítulos mundo + 7 Brasil, fontes, PT-BR normalizado) — o vazio da tela vira cultura, não banner de roleta.
- Termos claros: 18+, anti-bot, anti-colusão, anti-multi-conta.

Isso é o “Zero” do nome: menos adrenalina suja, mais sessão que a pessoa **quer repetir amanhã**. Retenção de pôquer recreacional se compra assim, não com e-mail de “seu bônus expira em 2 horas”.

---

## 6. Saúde emocional e financeira — o argumento ético (e o de negócio)

Parceiro sério pergunta: “vocês não estão só profissionalizando o prejuízo?”. A resposta honesta:

**O que já reduz dano**

- Fichas de treino separadas do salário.
- Reset diário de PM (o ego reseta com o saldo).
- Stakes baixos e frente única por mesa.
- Cashback de bad beat visível e auditável, não “bônus sujeito a 40×”.
- Educação no produto (Dica do Pró, história, regras Short Deck/Omaha escritas).
- Sem saque automático e sem PIX de produção: **não há como a demo atual drenar conta bancária em loop**.

**O que ainda falta para merecer o discurso completo** (está no QUALITY.md como checklist, **não** como feature pronta):

- Autoexclusão (6 meses → permanente) com bloqueio no login
- Limites de depósito / perda / tempo de sessão **escolhidos pelo jogador**
- Reality check na mesa (“você está há 3 horas”)
- KYC/AML de verdade no trilho 2027

Quem apresentar Zero Tilt como “já é jogo responsável certificado” está mentindo. Quem apresentar como **arquitetura pronta para receber esses freios** — e capital para construí-los **antes** do real — está alinhado com a marca.

---

## 7. Como o parceiro entra — rede de 2 níveis, rake, não pirâmide

O código já tem B2B: clube, agentes, split **15% casa / 85% clube**, rakeback do agente **0–50% da fatia do clube**. Detalhe do plano operacional: [`PLANO_GO_TO_MARKET_REDE_2_NIVEIS.md`](PLANO_GO_TO_MARKET_REDE_2_NIVEIS.md).

```
Casa (15%)
 └── Clube parceiro (até 85%)          ← nível 1
        └── Agente (0–50% da fatia)    ← nível 2
               └── Jogadores
```

Regras que o relatório **pode** assinar em reunião:

1. Profundidade máxima **2**. Não existe “neto”.
2. Sem taxa de adesão, kit ou bônus por cadastrar gente.
3. Comissão só sobre **rake de mão jogada**.
4. Hoje a liquidação da rede é **Play Money** (pontos, tickets, ranking). BRL só com SPA + PSP + KYC.
5. Quem não joga na semana **não** leva volume de linha.

Isso é o modelo de **skins/clubes** que o pôquer mundial já entende. Não é marketing de porta em porta. O parceiro âncora não “compra uma vaga”: ele **traz liquidez** e fica com a maior fatia do rake dessa liquidez.

Saque de comissão de clube (`POST /api/admin/clubs/:id/withdraw`) existe no desenho da API — **não se usa** para a rede Play Money. É o interruptor do dia em que a papelada estiver correta.

---

## 8. Estado atual, em uma página (para anexar)

| Camada | O que está no ar | Como conferir |
|--------|------------------|---------------|
| Domínio | Demo HTTPS | Abrir zerotiltpoker.net |
| Contas | Registro + e-mail + MFA | Fluxo `/register` → `/verify-email` |
| Lobby | Filtros stake/variante, Play \| Real, online | Header + `/lobby` |
| Motor | Hold’em, Short Deck, Omaha Short Deck, BBA 26 | Sentar e jogar; docs de variante |
| Dinheiro | PM diário; Real isolado; PIX prod **off** | Carteira; código rejeita PIX production |
| B2B | Clubes, agentes, 15/85, tema | `/admin/clubs` (papel admin) |
| Qualidade | Gate S20c verde; VPS 4/4 | `STATUS_OPERACIONAL.json` |
| Conteúdo | Dica do Pró, história 8+7, PT-BR | Home / laterais da mesa |

---

## 9. O que ainda não somos (deixar na mesa)

| Gap | Impacto | O que o capital resolve |
|-----|---------|-------------------------|
| Sem certificação de produção | Não se pode vender “Launch Ready” | Auditoria, pentest, processo de release |
| PIX automático e saque **off** em produção | Real não escala | PSP com aceite **formal** de iGaming + ledger de payout |
| Mesa = um processo | Teto de escala horizontal | Ownership distribuído de mesa (Redis hoje não basta) |
| Autoexclusão / limites de depósito | Discurso ZeroTilt incompleto no real | Product + legal (RG) **antes** de abrir PIX |
| KYC/AML | Exigência SPA | Parceiro de compliance + fluxo de documentos |
| MTT ao vivo ainda em evolução | Torneio não é o carro-chefe | Acabar o ciclo de mãos MTT com a mesma disciplina do cash |
| UI de Provably Fair no cliente TS | O motor prova; o jogador leigo não clica “auditar” | Modal de auditoria no histórico (o codec 0x30/0x31 já existe) |
| Link de indicação no registro | Rede ainda é admin-manual | Campo `sponsor` no signup + dashboard do agente |
| Catálogo curto | Bom para liquidez; pouco para high roller | Só crescer stake **depois** de encher as três mesas |

Nenhum desses gaps é vergonha de staging. São a lista de compras do sócio.

---

## 10. Pedido ao parceiro

Três coisas, nesta ordem:

1. **Liquidez** — um clube âncora que se comprometa com horário (20h–24h BRT) e mesa combinada, em Play Money, 90 dias. Sem isso não há produto para regular.
2. **Capital de porteira** — jurídico SPA (ou white-label em operador autorizado), PSP, KYC, jogo responsável (autoexclusão e limites), pentest. Alvo de regulação já escrito: **janeiro de 2027**.
3. **Paciência de marca** — não pedir “liga o PIX no grupo”. O código recusa; o relatório recusa; a marca se chama Zero Tilt.

Em troca:

- Até **85% do rake** da liquidez do clube (agentes saem dessa fatia, 0–50%).
- Árvore comercial **pronta** para virar BRL no mesmo split, sem redesenhar MMN.
- Produto que o jogador recreacional consegue **explicar para a família**: treina, aprende, não mistura salário, e o bad beat tem regra.

---

## 11. Melhorias tecnológicas e de experiência (o que acrescentar)

Esta seção é o backlog que um parceiro **deveria exigir** no termo de investimento. Está agrupada por valor para o jogador, não por ticket de GitHub. O que já existe aparece como base, para ninguém pagar duas vezes.

### 11.1 Jogo responsável (prioridade 1 — casa com o nome Zero Tilt)

| Melhoria | Por que | Base já existente |
|----------|---------|-------------------|
| **Autoexclusão** (6 meses / 1 ano / permanente) com bloqueio no login e no join | Sem isso, o discurso de saúde é incompleto no real | `token_version` já derruba sessão; falta tabela e UX |
| **Limites de depósito, perda e tempo** definidos pelo jogador, com cooling-off de 24h para subir | Padrão UKGC/SPA; protege o clube também | Ledger em centavos já permite somar perda/dia |
| **Reality check** na mesa a cada 60–90 min (“você está há X mãos / Y tempo”) | Quebra o piloto automático do tilt | Timer de ação e presence TTL 90s já existem |
| **Painel “minha sessão”** (mãos, resultado, tempo, rake pago) ao cash-out | O jogador vê o filme, não só o último bad beat | Hand history sequencial por mesa |
| **Atalho de “sair da mesa no próximo intervalo”** visível | Reduz all-in de raiva | Leave já só liquida entre mãos — falta o botão claro |

### 11.2 Confiança visível (o jogador precisa *ver* a honestidade)

| Melhoria | Por que | Base já existente |
|----------|---------|-------------------|
| **Botão “Auditar baralho”** no histórico da mão (mostrar hash, semente revelada, resultado ok/falha) | Provably Fair que o leigo não vê **não existe** para ele | Motor + codec binário 0x30/0x31 + `verify_provably_fair` no histórico |
| **Recibo da mão** (pote, rake, split 15/85, Loss Deflator, odd cent) em linguagem humana | Acaba com “a casa comeu 4 centavos” | Ordem financeira já é normativa; falta UI |
| **Selo de modo** enorme: Play Money vs Jogo Real, impossível de confundir no feltro | Evita o pior suporte: “pensei que era treino” | Header já troca modo; a mesa ainda pode gritar mais |
| **Exportar histórico** estilo PokerStars (já citado na arquitetura) com 1 clique | HUD externo e conferência | Hand history JSON já nasce para auditoria |

### 11.3 Mesa: sensação de sala profissional

| Melhoria | Por que | Base já existente |
|----------|---------|-------------------|
| **Time bank** (pacote de segundos extras, 1–2 por órbita) | 30/15/8s são justos; o recreacional ainda precisa de um “fôlego” sem travar a mesa | Timeouts já no `TableConfig` |
| **Sit-out / sentar no big blind** explícito | Padrão mundial; reduz briga de “me cobrou blind afk” | Assento ACTIVE no Postgres |
| **Fila / waitlist** da mesa cheia, com ping quando abrir cadeira | Liquidez não pode perder jogador no “tá lotado” | Cap e contador por mesa já no banco |
| **Replay da mão** (2×, mostrar muck só no showdown) | Ensino + prova social no grupo | Histórico da mão; Dica do Pró ao lado |
| **Som e haptic** (check, call, raise, seu turno, all-in) com mute por tipo | Celular brasileiro joga no ônibus; o “é sua vez” precisa ser óbvio | SPA leve; falta camada de áudio |
| **Gestos de 1 toque** (fold / call / min-raise) com confirmação só acima de X bb | Menos misclick = menos tilt | Ações legais já vêm do servidor |
| **Chat da mesa com mute e denúncia** (e filtro de PIX/contato) | Comunidade sem mesa virar balcão ilegal | Presence e JWT; falta canal |
| **Avatar e cor de cadeira estáveis** (não “Player4” genérico) | Identidade recreacional | Username 3–30 já no registro |

### 11.4 Lobby e crescimento da rede (o MMN Play Money precisa de chão de fábrica)

| Melhoria | Por que | Base já existente |
|----------|---------|-------------------|
| **Link de indicação no registro** (`?ref=`) gravado uma vez | Fase B do plano de mercado sem planilha | `club_agents.total_players_referred` |
| **Dashboard do agente** (VP, VL, mãos da linha, tickets da sexta) | Ninguém evangeliza o que não vê | Admin de agentes já lista % e comissão |
| **Ranking semanal no lobby** (não “ganhos em R$” — mãos, VP, seats) | Status no lugar de renda | Badge de presença já no header |
| **“Onde estão jogando agora”** — uma mesa destacada no horário âncora | Resolve o problema clássico do recreacional: 8 mesas vazias | Presence + OPEN tables |
| **Convite in-app** (“chamar para esta mesa”) com deep link | Tira o combo WhatsApp + “qual mesa?” | Ticket WS por mesa |
| **Tema de clube** aplicado de verdade no lobby do membro | White-label que se vê | `custom_theme_json` no admin já existe |

### 11.5 Motor, escala e operações

| Melhoria | Por que | Base já existente |
|----------|---------|-------------------|
| **Ownership distribuído de mesa** (a mesa sobrevive a dois pods) | Teto atual: 1 processo | Snapshot Redis já documentado; falta dono único eleito |
| **Ligar o codec binário no cliente TS** no caminho quente | Menos JSON, menos jitter no celular | Opcodes na API |
| **Zoom / fast-fold** (ao foldar, pula para nova mão) | Recreacional moderno; mais mãos/hora sem subir stake | Motor de mão já é rápido; falta o matcher |
| **Omaha full ring 52 cartas** (além do Short Deck Omaha 4-max) | Pedido natural depois que o 4-max encher | Avaliador 2+3 já existe no SD Omaha |
| **MTT: mãos ao vivo no mesmo nível do cash** | O lobby de torneio já existe; a mesa MTT ainda “em evolução” | BBA 26 níveis **já** no motor; balanceamento de mesas nas regras |
| **Observabilidade para o clube** (mãos/hora, drop, rake, deflator pago) | Parceiro não opera no escuro | `/api/metrics` admin; financials do clube |
| **CI de e2e catalog no PR** (seeded PM + Real) | Evita regressão de variante | Scripts `live-e2e-seeded-catalog.mjs` e 10k hands |

### 11.6 Conteúdo e acessibilidade (marca ZeroTilt)

| Melhoria | Por que | Base já existente |
|----------|---------|-------------------|
| **Dica contextual na mesa** (“no Short Deck flush ganha de full”) só na 1ª órbita | O recreacional erra regra, não só spot | Dica do Pró + `poker_variant` na mesa |
| **Modo daltônico / 4 naipes com símbolo + letra** | Padrão de sala séria | Cartas já são assets; falta paleta |
| **Layout mobile-first da mesa** (botões de ação no polegar, board no centro) | A demo será jogada no telefone | Tailwind; laterais 360px no desktop S20c |
| **Reduzir movimento** (`prefers-reduced-motion`) | Tilt também é sensorial | CSS atual |
| **PT-BR em 100% dos erros de API** (“saldo insuficiente para a frente de R$ 25”) | Suporte cai pela metade | `correctPtOrthography` no conteúdo; erros de API ainda podem ser cru |

### 11.7 Ordem sugerida (90 dias de produto, ainda em Play Money)

Não fazer tudo. Fazer nesta ordem, porque cada item **alimenta o próximo** e nenhum exige PIX:

1. Selo Play/Real inconfundível na mesa + sit-out + “sair no próximo intervalo”
2. Waitlist + “mesa âncora agora” no lobby
3. Link `?ref=` + dashboard cru do agente (mesmo que seja admin filtrado)
4. Recibo da mão + botão auditar baralho
5. Reality check de tempo + painel da sessão
6. Time bank + sons de “sua vez”
7. Replay + dica contextual de variante
8. Autoexclusão e limites — **mesmo em Play Money**, para o músculo existir antes do real

Os itens 1–4 enchem mesa e constroem confiança. Os itens 5–8 são o que autoriza o nome **Zero Tilt** quando a porteira de dinheiro real abrir. Escala de pods, Zoom e Omaha 52 ficam no ciclo seguinte, com a liquidez já provada.

---

## 12. Como conferir em 20 minutos (roteiro do parceiro cético)

1. Abrir `https://zerotiltpoker.net` — cadeado HTTPS, contador online.
2. Registrar, confirmar e-mail, ligar MFA.
3. Header: **Play Money**. Lobby: sentar em Hold’em 0,25/0,25 com uma segunda conta (precisa de 2).
4. Jogar 3 mãos. Olhar se o turno é óbvio, se o fold de timeout acontece, se o stack bate com o pote.
5. Trocar o header para **Jogo Real** e confirmar que a mesa PM **não** aceita esse modo (isolamento).
6. Pedir ao fundador o `STATUS_OPERACIONAL.json` do dia e o health `4/4` da VPS.
7. Ler este relatório **seção 9** em voz alta. Se o fundador quiser pular, não é o sócio certo — e não somos a casa certa para esse sócio.

---

*Números e limites deste relatório seguem `STATUS_OPERACIONAL.json` (S20c, 2026-09-01). Não alegar certificação de produção. Não alegar PIX de produção. Não alegar autoexclusão pronta. O restante — motor, isolamento de carteiras, Loss Deflator, B2B 15/85, demo no ar — está no código e na VPS para ser demonstrado, não para ser prometido.*

<!-- DOCUMENTATION_SYNC:START -->
> **Estado operacional sincronizado (2026-09-02):** S20e — Ultimate Pineapple cash 6-max (3 hole, usa 2+3, sem descarte, ranking Short Deck); catálogo cash canônico NLHE 0,25/0,25, Hold'em Short Deck 0,25/0,50, Omaha Short Deck 0,50/0,50 e Ultimate Pineapple 0,50/0,50 (Play Money e Jogo Real). **Sem certificação de produção; o código rejeita PIX em modo production. Deploy público: VPS Hostinger (demo/staging) com domínio zerotiltpoker.net. Staging/demo apenas; não alegar Launch Ready de produção.** Stack Docker local 4/4 healthy e VPS Hostinger 4/4 healthy. Migrations 001–033 aplicadas na VPS (033 VARCHAR(32) + mesas Ultimate Pineapple PM/Real 0,50/0,50 6-max). Gate S20e: 4 testes evaluate_hand_ultimate_pineapple PASS na VPS; rebuild API 4m13s; health público OK. Frontend: filtro Pineapple 0,50/0,50 + labels 3 hole. A VPS permanece no padrão seguro PIX mock. DePix existe somente em Sandbox não produtivo, com chave sk_test_, allowlist de depositante, idempotência, HMAC com janela temporal, deduplicação de eventos e crédito apenas em checkout.completed. O CPF/CNPJ é encaminhado ao provedor sem persistência local. Depósito manual continua como fallback; não há saque automático. Mesas com dono único por processo; settlement assinado (HMAC) na liquidação.
>
> Fonte canônica: [`STATUS_OPERACIONAL.json`](STATUS_OPERACIONAL.json). Verificação: `cargo run --bin documentation-sync -- --check`.
<!-- DOCUMENTATION_SYNC:END -->
