# Relatório para parceiros — Zero Tilt Poker

**Avaliação crítica atualizada em 17/09/2026:** a [seção 13](#13-avaliação-crítica-do-produto-e-da-viabilidade--17092026) examina frontend, Academy, vídeos, rede e monetização para um fundador com orçamento restrito. Leia-a antes de usar o pitch abaixo: ela aponta limitações e contradições que impedem tratar as afirmações comerciais anteriores como comprovação de qualidade, segurança ou enquadramento jurídico. O STATUS continua sendo a fonte operacional; esta avaliação não altera o produto nem o plano de compensação.

**Como usar este texto:** leia o sumário em voz alta (3 minutos). As seções 2–8 são a prova. A seção 9 é o que ainda não somos — deixe visível; esconder gap é o que queima due diligence. A seção 10 é o pedido. A seção 11 é o mapa de melhorias (tecnologia e experiência) para o próximo ciclo de investimento.

| | |
|--|--|
| **Produto** | Zero Tilt Poker |
| **Demo** | [https://zerotiltpoker.net](https://zerotiltpoker.net) |
| **Ciclo / números** | [`STATUS_OPERACIONAL.md`](STATUS_OPERACIONAL.md) — staging/demo, **sem** certificação de produção |
| **Stack** | Motor e API em **Rust**; UI em **TypeScript** (React + Vite + Tailwind); PostgreSQL 15; Redis 7; Caddy + HTTPS Let's Encrypt |
| **Repositório** | https://github.com/leofran2204/poker-platform |
| **Fonte dos números** | [`STATUS_OPERACIONAL.md`](STATUS_OPERACIONAL.md) |

---

## 0. Sumário executivo (o que dizer na primeira reunião)

Zero Tilt não é um clone de sala gigante. É uma plataforma de pôquer **recreacional**, com motor próprio, sentada em três ideias que o mercado mainstream abandonou:

1. **A mesa tem que ser honesta e rápida** — regras em Rust, dinheiro em centavos inteiros, baralho auditável, liquidação de mão assinada.
2. **O jogador não pode sair destroçado da sessão** — Loss Deflator (cashback de bad beat por matemática, não por “bônus de cassino”), frentes fixas, Play Money que renova, ensino no próprio lobby.
3. **O crescimento é de convite, não de anúncio** — rede de **dois níveis** (18% do rake individual do 1º nível, 12% do 2º, resto à casa; clube sem fatia), hoje ensaiada em Play Money; dinheiro real só com licença.

O que já está no ar: demo HTTPS, e-mail verificado, MFA, mesas Play Money e Jogo Real **isoladas**, quatro variantes (Hold’em, Short Deck, Omaha Short Deck, Ultimate Pineapple), torneios com Big Blind Ante em 26 níveis (inscrição + **mãos MTT ao vivo em 3 mesas**, S22), admin de clubes, stack Docker **4/4 healthy** na VPS.

O que **não** está: certificação de produção, autoexclusão de produto, multi-servidor de mesas. PIX automático DePix **está ligado na demo** (crédito provisório até R$ 50 no `processing`, saque travado até liquidar). Isso não se esconde. O parceiro que entra agora compra **produto + liquidez Play Money + o trilho até 2027**, não um cassino “já legalizado”.

**Pedido:** liquidez (gente na mesa) e, no futuro, capital de compliance. A rede é 2 níveis: **18%** do rake de quem você trouxe, **12%** do rake de quem eles trouxeram, resto à casa. Clube não leva dinheiro. Enquanto a papelada não fecha, a operação pública é **Play Money**.

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
| Hold’em NL 0,75/1,50 | 52 cartas, 2 hole | 0,75 / 1,50 | 9 | R$ 150 |
| Hold’em Short Deck | 36 cartas (sem 2–5); trinca > sequência; flush > full house; wheel A-6-7-8-9 | 0,25 / 0,50 | 8 | R$ 75 |
| Omaha Short Deck | 4 hole; showdown = 2 hole + 3 board | 0,50 / 0,50 | 5 | R$ 100 |
| Ultimate Pineapple | 3 hole, sem descarte; showdown 2 hole + 3 board | 0,50 / 0,50 | 6 | R$ 75 |

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
- PIX automático DePix na **demo**: HMAC, idempotência, allowlist, crédito provisório ≤ R$ 50 no `checkout.processing`, `completed` confirma o líquido, saque via payout worker (teto/`HELD`). CPF/CNPJ vai ao provedor **sem ficar gravado localmente**. Sem certificação de produção.

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

- Play Money **renova todo dia** (R$ 150 cash + R$ 150 torneio; sem rebuy). Perdeu a noite, não perdeu o mês.
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
- PIX DePix na demo tem teto de crédito provisório (R$ 50) e trava de saque até liquidar: **não há loop livre de drenagem**, mas já há depósito automático.

**O que ainda falta para merecer o discurso completo** (está no QUALITY.md como checklist, **não** como feature pronta):

- Autoexclusão (6 meses → permanente) com bloqueio no login
- Limites de depósito / perda / tempo de sessão **escolhidos pelo jogador**
- Reality check na mesa (“você está há 3 horas”)
- KYC/AML de verdade no trilho 2027

Quem apresentar Zero Tilt como “já é jogo responsável certificado” está mentindo. Quem apresentar como **arquitetura pronta para receber esses freios** — e capital para construí-los **antes** do real — está alinhado com a marca.

---

## 7. Como o parceiro entra — rede de 2 níveis, rake, não pirâmide

O motor continua com B2B **15% casa / 85% clube**. A rede de gente é outra camada: o cadastro grava o **ID do patrocinador**, e o jogador pode sentar em qualquer clube sem mudar de pai. Detalhe: [`PLANO_GO_TO_MARKET_REDE_2_NIVEIS.md`](PLANO_GO_TO_MARKET_REDE_2_NIVEIS.md).

```
Raiz (1º cadastro da plataforma)
 └── 1º nível (entrou pelo convite da raiz, ou já estava na sala)
        └── 2º nível (entrou pelo convite do 1º nível)
               └── (nível 3: a raiz não vê e não ganha)
```

Cada afiliado, no **próprio** admin, vê só os seus dois andares.

Regras que o relatório **pode** assinar em reunião:

1. Profundidade máxima **2** na tela de cada um. Não existe “neto” visível nem pago para mim.
2. Sem taxa de adesão, kit ou bônus por cadastrar gente.
3. Comissão só sobre **rake individual de mão jogada**: **18%** (1º nível) e **12%** (2º nível); o resto fica com a casa. Clube não recebe rake nesta rede.
4. Hoje a liquidação da rede é **Play Money** (pontos, tickets, ranking). BRL só com SPA + PSP + KYC.
5. Quem não joga na semana **não** leva volume de linha.

O parceiro âncora não “compra uma vaga”: ele **traz liquidez**, indica com o próprio código e acompanha só a rede dele.

Saque de comissão de clube (`POST /api/admin/clubs/:id/withdraw`) existe no desenho da API — **não se usa** para a rede Play Money. É o interruptor do dia em que a papelada estiver correta.

---

## 8. Estado atual, em uma página (para anexar)

| Camada | O que está no ar | Como conferir |
|--------|------------------|---------------|
| Domínio | Demo HTTPS | Abrir zerotiltpoker.net |
| Contas | Registro + e-mail + MFA | Fluxo `/register` → `/verify-email` |
| Lobby | Filtros stake/variante, Play \| Real, online | Header + `/lobby` |
| Motor | Hold’em, Short Deck, Omaha Short Deck, BBA 26 | Sentar e jogar; docs de variante |
| Dinheiro | PM diário; Real isolado; PIX DePix na demo (provisório ≤ R$ 50) | Carteira + STATUS |
| B2B + rede | Convite `?ref=`; 18/12 modelo; 15/85 no motor é legado | Plano de rede 2 níveis |
| Qualidade | Gate S20c verde; VPS 4/4 | `STATUS_OPERACIONAL.json` |
| Conteúdo | Dica do Pró, história 8+7, PT-BR | Home / laterais da mesa |

---

## 9. O que ainda não somos (deixar na mesa)

| Gap | Impacto | O que o capital resolve |
|-----|---------|-------------------------|
| Sem certificação de produção | Não se pode vender “Launch Ready” | Auditoria, pentest, processo de release |
| PIX automático ligado na demo, sem certificação iGaming | Real não escala como produto legal | Aceite formal de iGaming + KYC 2027 |
| Mesa = um processo | Teto de escala horizontal | Ownership distribuído de mesa (Redis hoje não basta) |
| Autoexclusão / limites de depósito | Discurso ZeroTilt incompleto no real | Product + legal (RG) **antes** de abrir PIX |
| KYC/AML | Exigência SPA | Parceiro de compliance + fluxo de documentos |
| MTT ao vivo ainda em evolução | Torneio não é o carro-chefe | Acabar o ciclo de mãos MTT com a mesma disciplina do cash |
| UI de Provably Fair no cliente TS | O motor prova; o jogador leigo não clica “auditar” | Modal de auditoria no histórico (o codec 0x30/0x31 já existe) |
| Painel **Minha Estrutura** + ledger 18/12 | Convite `?ref=` + painel + ledger mão a mão e sobre fee 15% (S22) | Admin do afiliado (2 níveis) + pontos sobre rake individual |
| Catálogo curto | Bom para liquidez; pouco para high roller | Só crescer stake **depois** de encher as três mesas |

Nenhum desses gaps é vergonha de staging. São a lista de compras do sócio.

---

## 10. Pedido ao parceiro

Três coisas, nesta ordem:

1. **Liquidez** — um clube âncora que se comprometa com horário (20h–24h BRT) e mesa combinada, em Play Money, 90 dias. Sem isso não há produto para regular.
2. **Capital de porteira** — jurídico SPA (ou white-label em operador autorizado), PSP, KYC, jogo responsável (autoexclusão e limites), pentest. Alvo de regulação já escrito: **janeiro de 2027**.
3. **Paciência de marca** — não pedir “liga o PIX no grupo”. O código recusa; o relatório recusa; a marca se chama Zero Tilt.

Em troca:

- **18%** do rake individual de quem o afiliado trouxe e **12%** do segundo nível; resto à casa. Clube não leva fatia.
- Árvore de **2 níveis** com os mesmos % em ponto e em real.
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
| **Omaha full ring 52 cartas** (além do Short Deck Omaha 5-max) | Pedido natural depois que o 5-max encher | Avaliador 2+3 já existe no SD Omaha |
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
6. Pedir ao fundador o `STATUS_OPERACIONAL.md` do dia e o health `4/4` da VPS.
7. Ler este relatório **seção 9** em voz alta. Se o fundador quiser pular, não é o sócio certo — e não somos a casa certa para esse sócio.

---

## 13. Avaliação crítica do produto e da viabilidade — 17/09/2026

> Avaliação datada. Depois desta data o STATUS passou a registrar PIX automático DePix na demo (crédito provisório ≤ R$ 50). Números vigentes: [`STATUS_OPERACIONAL.md`](STATUS_OPERACIONAL.md).

### 13.1 Parecer executivo e alcance da análise

**Modelo esclarecido pelo fundador após a análise:** a assinatura dará acesso às aulas, cursos, dicas, estratégias e coach IA. Ela não será requisito para jogar poker contra outros participantes: para isso, o jogador comprará fichas reais, que poderão ser apostadas e sacadas. O piloto com conhecidos usará valores simbólicos para apoiar o desenvolvimento e verificar o ciclo de depósito, jogo e saque. Esse esclarecimento descreve a oferta pretendida, não comprova implementação ou altera o estado operacional do STATUS.

Há duas ofertas independentes: serviço educacional por assinatura e jogo entre participantes com fichas resgatáveis. A compra dessas fichas representa saldo do jogador, não receita de venda do curso. A avaliação deve acompanhar separadamente qualidade e retenção da assinatura, funcionamento financeiro do jogo e custos da operação. A relação entre assinatura e elegibilidade a comissões não foi definida por esse esclarecimento; não inferir mudança na qualificação do plano de rede.

**A Zero Tilt tem uma base concreta de produto, mas ainda não demonstra qualidade equivalente às melhores salas e escolas de poker. Para um fundador CLT com orçamento restrito, recomendo priorizar a validação da entrega educacional enquanto prepara e verifica as condições da operação real. Isso é uma recomendação de sequência; não substitui o modelo de duas ofertas definido pelo fundador.**

Não recomendo abandonar o projeto nem reescrever sua stack. Recomendo limitar o piloto, corrigir o conteúdo e demonstrar que pessoas voltam e pagam pelo valor entregue. O apoio financeiro voluntário dos conhecidos é válido como objetivo do piloto; não comprova por si só demanda fora desse círculo. Software desenvolvido, jogadores satisfeitos e negócio sustentável são três provas diferentes. As propostas de foco educacional nas seções seguintes devem ser lidas como alternativas de execução, não como descrição da intenção do fundador.

Esta análise considera documentação, trechos de código e inspeção pública da home, índice da Academy, primeira aula e acesso ao lobby. Inclui teste da home com viewport de 390 × 844, inventário das 26 aulas e 54 perguntas do JSON, metadados dos 25 MP4 finais e nove quadros extraídos dos episódios 01, 07 e 25, aos 8, 30 e 55 segundos. Foram examinados roteiros e exemplos selecionados; não houve revisão integral de todas as aulas.

Foram consultadas fontes primárias de mercado e órgãos públicos. A comparação é de recursos documentados e critérios de qualidade; não é teste integral de contas pagas dos concorrentes. Não houve login, depósitos, saques, partida autenticada, pentest, teste de carga, avaliação em telefone físico nem acesso a métricas reais de aquisição, retenção ou receita. Contagens históricas de testes não foram reexecutadas. A análise jurídica identifica questões de enquadramento; não substitui um parecer sobre a operação concreta.

| Dimensão | Avaliação atual | Confiança e limite |
|---|---|---|
| Base de engenharia | Promissora para uma demo evoluída | Há código de rake em inteiros, assinatura de liquidação e regras de potes; isso não certifica a operação inteira. |
| Frontend público | Identidade reconhecível; acabamento e conversão ainda intermediários | Inspeção visual real e problema mobile medido. Experiência da mesa autenticada não avaliada nesta rodada. |
| Conteúdo pedagógico | Precisa de revisão antes de ser vendido como formação de referência | Erros e contradições concretos em matemática, posição e ranges. |
| Vídeos | Microaulas de introdução; acabamento técnico abaixo de uma oferta premium | Todos os arquivos medidos; avaliação visual por amostragem, sem audição integral. |
| Modelo de aquisição | Convites pessoais são plausíveis; MMN não está validado | Não foram apresentados dados de retenção, margem ou eficácia incremental do segundo nível. |
| Dinheiro real | Não há evidência suficiente para recomendar lançamento | Documentação conflitante e enquadramento, pagamentos e controles ainda por demonstrar. |
| Primeira monetização | Pequena turma com acompanhamento é uma hipótese mais compatível com o fundador | Precisa provar procura fora da obrigação afetiva de amigos e familiares. |

Não atribuo uma nota global de 0 a 10: ela esconderia lacunas de evidência e daria precisão artificial a critérios muito diferentes.

### 13.2 O que já tem valor — e o que ainda precisa ser provado

O projeto possui mais substância que uma landing page: motor próprio, interface navegável, modalidades, estrutura de curso, vídeos e regras documentadas. O cálculo monetário em inteiros e a separação entre formação de potes, rake e pagamentos são decisões adequadas para auditabilidade. Há implementação de assinatura HMAC em `API-Axum/src/game_actor.rs:116`, rake em `Motor-Rust/src/rake.rs:61` e cálculo de deflator em `Motor-Rust/src/loss_deflator.rs:181`.

Isso demonstra componentes de engenharia, não ausência de falhas. HMAC permite verificar integridade/autenticidade no contexto de uma chave compartilhada; não prova sozinho justiça do embaralhamento, ausência de manipulação pelo operador, solvência ou impossibilidade de colusão. Rust também não elimina erros de regra de negócio.

A disciplina de STATUS, gates e documentação ajuda, mas o pitch excede algumas provas: expressões como “precisão bancária B3”, antifraude “100%” ou dizer que suporte “cai pela metade” precisam de critérios e medições. Uma implementação de detecção não demonstra sua eficácia contra adversários reais. O projeto deve distinguir controles existentes, testes internos, auditoria independente e resultados em operação.

Antes de custodiar recursos, seria necessário demonstrar recuperação após falha, conciliação financeira, controle de acesso administrativo, tratamento de disputas, proteção contra abuso, disponibilidade e capacidade de honrar retiradas. Um calendário de KYC para 2027 não prova que o adiamento seja adequado ao modelo usado hoje.

### 13.3 Frontend: identidade boa, experiência ainda distante da referência

Na home desktop, verde, dourado, mesa ilustrativa e chamada principal formam uma identidade coerente. Os exemplos de cartas e o simulador do deflator ajudam a explicar o produto. Manter esse sistema visual faz mais sentido que financiar uma troca completa de marca agora.

Os principais problemas observados são:

1. **Ênfase monetária antes da proposta de aprendizado.** “R$ 150 para jogar agora” domina a primeira tela. Existe aviso de fichas virtuais, mas a hierarquia favorece interpretação de bônus em dinheiro. Para o público pretendido, uma proposta centrada em aprender e jogar com conhecidos seria mais clara; a unidade virtual deveria ser inequívoca.
2. **Transbordamento no celular.** Com viewport de 390 pixels, a largura útil era 380 e o documento tinha 668 pixels. O bloco de notícias alcançava 652 pixels de largura. Isso é um defeito concreto de layout, não apenas preferência estética. Rolagem interna de abas é aceitável; alargar o documento inteiro prejudica navegação.
3. **Descoberta do produto limitada para visitantes.** `/lobby` pede login antes de listar mesas. Para um produto novo, mostrar horários e disponibilidade pública pode diminuir a incerteza antes do cadastro. Isso deve respeitar privacidade e não simular jogadores ou atividade.
4. **Carga de informação.** Deflator, quatro jogos, notícias, dicas e Academy disputam atenção. O visitante iniciante precisa primeiro entender o que vai aprender, com quem vai jogar e qual ação executar.
5. **Leitura e acabamento.** Há textos secundários pequenos e discretos; na primeira aula apareceram marcadores `**` literais no material. Contraste precisa ser medido antes de declarar conformidade. Há suporte a movimento reduzido no CSS, um sinal positivo, mas não uma auditoria de acessibilidade.
6. **Promessa de produto versus apresentação.** O índice anuncia “Mão na Mesa”, “jogador virtual de IA” e vídeos de dois minutos. Na aula inspecionada, o fluxo visível é vídeo, leitura e quiz. A interface deve descrever com precisão os recursos efetivamente entregues em cada aula.

O padrão de referência de uma sala não se resume à aparência do feltro. A GGPoker documenta histórico, replay, download, filtros e análise de mãos no PokerCraft, além de política explícita sobre bots, assistência em tempo real e colusão. Esses recursos ilustram o nível de operação e confiança a comparar, sem presumir que toda funcionalidade precise ser copiada. [PokerCraft](https://ggpoker.com/poker-games/pokercraft/), [política de segurança da GGPoker](https://legal.ggpoker.com/network/security-ecology-policy/).

Para a Zero Tilt, os próximos testes devem cobrir cadastro, confirmação de e-mail, localização de uma mesa com companhia, entendimento do turno, reconexão, saída e recibo da mão. Sessões observadas com iniciantes são mais úteis agora que adicionar novas modalidades. Não é possível aprovar a experiência da mesa apenas pela vitrine animada.

### 13.4 Curso: a prioridade é correção, depois profundidade

A organização por módulos, microaulas, leitura e quiz é um bom ponto de partida. O conteúdo próprio das variantes também pode ajudar jogadores a entender regras específicas da Zero Tilt. Porém, há problemas que impedem recomendar a Academy atual como equivalente às melhores formações.

**Achados reproduzíveis:**

| Local | Problema | Por que importa / correção necessária |
|---|---|---|
| `ZeroTiltCurso/ep07-pot-odds/plan.md`, narração; `script.py:36` e `:41` | O exemplo mistura pagar 25, concorrer a 100, odds 4:1 e exigência de 20%. | Se 75 já inclui a aposta adversária, o pote final é 100 e a exigência é 25%. Se havia 75 antes da aposta adversária de 25, o pote final é 125 e a exigência é 20%. A aula precisa declarar qual pote está contando; a redação atual mistura os dois casos. |
| Mesmo episódio, `script.py:69` e narração | Compara 36% do flop ao river com preço de um call no flop e conclui lucro de forma geral. | Pagar no flop normalmente compra a próxima carta, não acesso gratuito ao river. A comparação de duas cartas requer all-in ou hipótese explícita de ausência de custo posterior; considerar realização de equity e outs limpos. |
| `Documentacao/CURSO_ESTRATEGIA_POKER.md:100` | Afirma que o limp reduz a taxa de vitória pela metade. | Não existe essa relação matemática universal. Uma recomendação simples para iniciantes precisa ser apresentada como heurística, com condições e exceções. |
| `Frontend-Web/src/data/tipsContent.json`, dica de ranges também exposta na home | A tabela inclui `KTs+` em UTG, mas o exemplo diz que `KQs` está fora do range; descreve Q-high com KQ como segundo par. | `KQs` integra `KTs+`; em um flop não pareado cuja maior carta seja Q, KQ faz top pair. O texto se contradiz independentemente da escolha estratégica do range. |
| Mesma dica | Define `KTs+` incluindo AKs e agrupa SB entre quem fala por último após o flop. | A notação usual mantém o K e varia o kicker até Q; AKs é outra categoria. O SB age primeiro após o flop quando permanece na mão. |
| Dica contra limp do SB | Um raise para 4 BB pago por outro jogador é seguido de um pote de 4 BB no exemplo; recomenda c-bet em qualquer flop. | Sem outras contribuições ou rake, dois jogadores que colocaram 4 BB formam 8 BB. Frequência e tamanho de c-bet dependem de ranges, textura e adversário. |

Esses achados não significam que todas as aulas sejam ruins. Significam que a revisão editorial atual não garante correção consistente. Uma referência externa anexada à dica não valida automaticamente os parágrafos produzidos internamente.

No código do quiz, `Frontend-Web/src/lib/course.ts:86` compara a resposta com gabarito já presente no conteúdo. Mesmo que o gabarito tenha sido gerado por um bot, isso não constitui análise de solver independente em tempo real. Quando há perguntas práticas, a aprovação usa sua nota em vez da nota total (`:128`). Isso é uma escolha pedagógica que deve ficar clara; pode aprovar quem errou teoria relevante.

Com duas questões de igual peso, as notas possíveis são 0%, 50% e 100%: o corte anunciado de 70% equivale a exigir 100% naquela aula. Também não prova transferência para uma mão diferente. Vale aumentar a variedade de situações e avaliar entendimento com exemplos novos, sem simplesmente multiplicar perguntas triviais.

Há ainda uma barreira de sequência: o iniciante passa por história do baralho, Mississippi e personalidades antes das regras práticas. História pode enriquecer uma trilha opcional; para ativação, começar com uma decisão simples e uma primeira mão tende a ser uma hipótese melhor. Deve ser testada com alunos, não tratada como resultado garantido.

Como comparação, PokerStars Learn já oferece fundamentos gratuitamente, sem exigir cadastro para começar. GTO Wizard apresenta estudo, treino de situações e análise; Run It Once oferece trilhas e conteúdo especializado com instrutores identificados. A Zero Tilt terá dificuldade para cobrar apenas por uma biblioteca introdutória genérica. Sua oportunidade é acompanhamento em português, feedback correto, vínculo comunitário e exercícios ligados à prática. [PokerStars Learn](https://www.pokerstars.com/poker/learn/), [GTO Wizard](https://gtowizard.com/), [Run It Once](https://www.runitonce.com/).

Recomendação: revisar matemática, regras, ranges e explicações com um jogador/professor competente e identificável; documentar formato, posições, stacks, rake e hipóteses de cada exemplo. Ferramentas teóricas de poker tradicional não devem ser apresentadas como automaticamente válidas para o deflator ou variantes próprias. O coach virtual precisa permanecer explicitamente virtual, sem histórico profissional inventado.

### 13.5 Vídeos: avaliação técnica e visual

Os 25 MP4 finais em `ZeroTiltCurso/epNN-*/` têm resolução **854 × 480**, aproximadamente **15 fps**, trilha AAC e duração individual entre **73,89 e 91,41 segundos**. Somam **34 minutos e 32 segundos**, aproximadamente, e **23,7 MB**. Esses números descrevem os arquivos locais; não comprovam que todos os arquivos servidos na internet sejam idênticos. O curso tem 26 aulas e uma delas não possui URL de vídeo.

Nas nove amostras extraídas, há coerência de cores e uso de cartas, barras e fichas. Em contrapartida, textos e elementos de jogo ocupam pouco da tela, com longos espaços vazios. A leitura em celular fica prejudicada quando o quadro inteiro é reduzido. O estilo é compatível com microaulas introdutórias simples, mas ainda não com a apresentação esperada de uma formação premium.

480p e 15 fps não tornam uma aula incorreta, e aumentar a resolução não corrigirá um erro matemático. Para acabamento, recomendo um master em 1080p, elementos maiores e 24–30 fps quando o movimento justificar, mantendo versão leve para conexão limitada. O objetivo é clareza, não efeitos cinematográficos. Cada vídeo deve mostrar a decisão, dar tempo para pensar, explicar o resultado e apresentar uma variação do problema.

O player usa `<video controls>`, sem faixa de legendas configurada no componente. O roteiro está disponível como alternativa no caminho de placeholder, mas não como transcrição acessível permanente ao lado de cada vídeo existente. Legendas sincronizadas, transcrição revisada, poster inicial e indicação precisa da duração melhorariam uso e acessibilidade. O critério de legendas da WCAG deve ser avaliado conforme o conteúdo e suas exceções; texto em alguns quadros não equivale automaticamente à legenda de toda a fala. [WCAG 2.2 — legendas](https://www.w3.org/WAI/WCAG22/Understanding/captions-prerecorded).

A medição de volume em três episódios encontrou pico entre −2,4 e −3,8 dBFS e média próxima de −21,5 dBFS. Isso mostra sinal presente e picos abaixo de 0 dBFS na amostra; não mede naturalidade da voz, pronúncia, sincronização integral nem loudness em LUFS. Os planos informam voz sintética AntonioNeural. Não afirmo ter ouvido e aprovado toda a narração.

A aba do navegador integrado travou ao tentar reproduzir a primeira aula. Isso limita a avaliação do streaming; não basta para atribuir o defeito ao site. Por isso, os MP4 foram examinados localmente. Fica pendente reprodução contínua em Chrome/Safari e telefone físico, inclusive em conexão limitada.

### 13.6 Loss Deflator: diferenciação que precisa ser compreendida pelos dois lados

O deflator merece atenção comercial, mas não deve ser vendido como proteção sem custo ou como prova de jogo responsável. O próprio motor explica que a devolução vem do pote disputado, não de uma reserva paga pela casa. A home informa que a parte devolvida é retirada da parcela do vencedor, o que é uma transparência positiva.

No exemplo ilustrativo da home, com pote líquido de 200 e faixa de 25%, o perdedor recebe 50 e o vencedor fica com 150. A regra muda o pagamento do jogo. Alguém pode gostar da compensação quando perde e rejeitá-la quando vence; isso precisa ser medido com jogadores que entenderam ambos os resultados.

O favorecimento de determinados all-ins também altera o valor esperado das decisões em relação ao poker convencional. Menor perda em um caso não demonstra menor tilt ou melhor retenção da população. Pode inclusive favorecer jogadores que já entram mais frequentemente com vantagem, dependendo do comportamento da mesa. É hipótese para análise, não conclusão já provada.

Recomendo testar compreensão e preferência em Play Money: apresentar resultados dos dois lados, pedir ao jogador que explique a regra e medir retorno e reclamações. Revisar cenários multiway, side pots, limites e incentivos com cuidado. Na documentação há tensão entre percentuais sobre pote líquido e limite descrito como percentual do valor perdido; reconcilie o contrato antes de comunicar uma promessa precisa.

### 13.7 Regulação: não presumir licença de bet, nem exceção por amizade

**A premissa de que toda sala de poker precisa seguir exatamente o mesmo caminho de uma bet de quota fixa não está demonstrada.** A pergunta 56 das questões técnicas da SPA, atualizada em maio de 2026, aponta exclusões do regime de evento virtual de jogo de quota fixa para jogos de habilidade, como poker, jogos multiapostador e P2P, inclusive torneios entre apostadores. Videopoker individual e jogos contra a casa não devem ser confundidos com poker entre jogadores. [Ministério da Fazenda — questões técnicas, pergunta 56](https://www.gov.br/fazenda/pt-br/composicao/orgaos/secretaria-de-premios-e-apostas/apostas-de-quota-fixa/questoes-tecnicas).

Isso não autoriza concluir que a Zero Tilt pode receber depósitos, custodiar recursos e pagar comissões sem obrigações. Também não permite afirmar um custo obrigatório de outorga para o produto sem primeiro classificá-lo. O roteiro “SPA ou white-label → ligar real”, presente no plano, precisa de revisão jurídica específica, inclusive considerando variantes, deflator, bots, premiações, comissões e atuação efetiva do operador. A pesquisa não constitui levantamento exaustivo de todas as decisões e normas aplicáveis.

O fato de clientes serem amigos ou de o cadastro exigir convite não afasta por si só obrigações de uma atividade econômica. Se a discussão envolver jogo de azar, o art. 50 da Lei das Contravenções Penais é uma referência relevante, mas seu enquadramento depende do jogo concreto; não classifico aqui poker automaticamente como contravenção. Amizade não é fundamento suficiente para dispensar análise. [Decreto-Lei 3.688/1941](https://www.planalto.gov.br/ccivil_03/decreto-lei/del3688.htm).

Educação paga também requer oferta honesta, tratamento adequado de dados, atendimento e estrutura fiscal compatível. Venda online de curso deve contemplar o direito de arrependimento aplicável; pequeno porte pode permitir simplificações em proteção de dados, mas não isenção geral da LGPD. [CDC, art. 49](https://www.planalto.gov.br/ccivil_03/leis/l8078compilado.htm), [ANPD — Resolução 2, art. 6](https://www.gov.br/anpd/pt-br/acesso-a-informacao/institucional/atos-normativos/regulamentacoes_anpd/resolucao-cd-anpd-no-2-de-27-de-janeiro-de-2022).

O gasto inicial mais útil seria uma consulta de escopo fechado, com descrição e diagrama dos fluxos: quem paga, a quem, por qual serviço, onde fica o saldo, se há resgate/prêmio e como se remunera cada indicação. Pedir respostas escritas sobre enquadramento, obrigações tributárias, pagamentos, dados, publicidade e responsabilidades. Isso é diferente de contratar de imediato uma estrutura completa de operador de apostas.

Enquanto isso, a alternativa de menor exposição operacional é treino com fichas não resgatáveis, sem acertos paralelos, acompanhado de serviço educacional independente. Não vincular compra do curso a saldo sacável ou participação em premiação monetária. Freeroll com prêmio em dinheiro também precisa de enquadramento próprio: ausência de buy-in não resolve toda a análise.

Há uma contradição prioritária: `STATUS_OPERACIONAL.md:46` documenta depósito manual em carteira real, enquanto o plano de rede e trechos das regras afirmam operação pública só Play Money. Desligar PIX automático não equivale a impedir dinheiro real. Não verifiquei movimentações financeiras nem afirmo que tenham ocorrido. Antes de convidar clientes, conferir o fluxo efetivo e alinhar STATUS, interface, termos e comunicação à decisão operacional autorizada.

### 13.8 MMN: o convite é útil; a árvore não resolve o negócio

**Para este estágio, recomendo crescimento por comunidade e indicação direta; não recomendo colocar MMN remunerado sobre rake no centro da proposta.** Trata-se de recomendação de estratégia, não de mudança aplicada ao plano canônico.

O desenho atual tem aspectos melhores que recrutamento remunerado: não cobra kit ou adesão, não promete pagamento pelo simples cadastro e vincula comissão a uma atividade existente. Contudo, dois níveis e ausência de kit não são um certificado de legalidade ou sustentabilidade. A CVM/Senacon diferencia venda legítima de produtos e serviços de estruturas dependentes de recrutamento e promessas irreais; é preciso examinar a substância econômica, não o nome escolhido. [CVM — marketing multinível e pirâmides](https://www.gov.br/cvm/pt-br/assuntos/noticias/2013/distincao-entre-marketing-multinivel-e-piramides-financeiras-e-tema-do-6-boletim-de-protecao-do-consumidor-investidor-00d2f80d3c9d4a41860a973cf591a4d2).

Também é importante entender que dois níveis remunerados por participante não limitam a profundidade total do grafo: cada novo participante pode ter seus próprios dois níveis. O painel limitado não é prova suficiente de controle do modelo econômico.

Os riscos específicos são:

- Remunerar volume de jogo e exigir 100 mãos semanais pode incentivar frequência para qualificação, em tensão com a promessa de controle do tilt. A exigência não prova, sozinha, caráter “anti-pirâmide”.
- Os 30% máximos destinados à rede diminuem a margem disponível para suporte, infraestrutura, pagamentos, fraude e tributos. Os 70% restantes não são lucro líquido.
- O segundo nível paga alguém mais distante da aquisição direta; sem comparação de retenção e receita incremental, pode ser custo sem benefício demonstrado.
- Amigos e familiares podem aderir por afeto ou constrangimento, inflando a percepção de demanda. Relações pessoais tornam disputas sobre perdas e pagamentos mais dolorosas.
- Rede comercial requer suporte, regras, extratos e resolução de disputas. Para uma pessoa com emprego em horário comercial, essa carga importa tanto quanto o custo do servidor.

O problema inicial de poker é reunir pessoas compatíveis no mesmo horário e manter a experiência justa. Uma árvore cheia de cadastros não resolve uma mesa vazia. Com pouca gente, dividir atenção entre variantes, stakes, cash e torneios piora a concentração. Proponho testar uma sessão âncora de Hold'em com fichas virtuais, sem alterar o catálogo automaticamente.

Depois de validar uma oferta educacional que as pessoas comprariam sem recrutar ninguém, pode-se testar indicação direta com condições claras e comissão compatível com a margem. A remuneração deve depender da venda efetiva do serviço, considerar cancelamentos e não exigir compras compulsórias. Uma eventual mudança para esse modelo precisará ser registrada no dono do plano, sem misturar comissões de curso com o ledger de rake.

### 13.9 Economia: contas para decidir, não projeções de renda

Sem dados de jogadores ativos, rake efetivo, custos, impostos, churn e horas do fundador, não é possível estimar lucro real. Os exemplos abaixo são **hipóteses ilustrativas**, não preços de mercado, promessa de faturamento ou recomendação de iniciar jogo real.

**Operação de rake:** suponha 30 jogadores ativos gerando 40 reais de rake individual por mês. O rake bruto seria 1.200; com os dois níveis integralmente pagos, 360 iriam à rede e 840 ficariam na casa antes de qualquer outro custo. Depositar ou apostar 40 reais não é o mesmo que gerar 40 reais de rake.

Se um custo operacional hipotético fosse 500 por mês, o ponto de equilíbrio antes de tributos e custos variáveis seria `500 / 0,70 = 714,29` de rake bruto. Para obter mais 2.000 de remuneração do fundador, seriam `2.500 / 0,70 = 3.571,43`, ainda ignorando tributos, perdas, atendimento adicional e pagamentos. Logo, esse cálculo é um piso incompleto, não viabilidade demonstrada.

O rake retira recursos da mesa. Comissões redistribuem parte e podem ou não voltar ao jogo; não criam riqueza agregada. Em um grupo fechado, a continuidade depende de as pessoas aceitarem pagar pelo entretenimento e de suas condições para fazê-lo. O orçamento doméstico dos conhecidos não deve ser tratado como motor infinito de receita. Saldos dos jogadores também não são capital de giro da empresa.

**Oferta educacional mensal, após revisão de qualidade:**

| Hipótese | Receita bruta | Indicação direta hipotética de 10% | Reserva ilustrativa de 15% para taxas, tributos e reembolsos | Custo mensal hipotético | Saldo antes do trabalho do fundador |
|---|---:|---:|---:|---:|---:|
| 20 alunos × 39 | 780 | 78 | 117 | 200 | 385 |
| 50 alunos × 49 | 2.450 | 245 | 367,50 | 300 | 1.537,50 |

A reserva de 15% não é alíquota fiscal nem orçamento validado; cada componente precisa ser apurado. Se a segunda hipótese consumir 20 horas mensais, restariam aproximadamente 76,88 por hora antes de outros custos não incluídos. Com 40 horas, cai para 38,44. Esse exercício ajuda a dimensionar a entrega e comparar com seu tempo disponível.

Uma biblioteca de 35 minutos, isoladamente, pode não sustentar assinatura. O valor recorrente precisaria vir de revisão de mãos, prática guiada, novos exercícios ou encontros. Para testar pagamento inicial, uma turma de duração definida pode ser mais simples que uma assinatura. Cancelamento, custo de suporte e retenção decidirão se a recorrência faz sentido.

### 13.10 Caminho recomendado para um fundador CLT

| Opção | Adequação agora | Condição para avançar |
|---|---|---|
| Comunidade gratuita com treino PM | Alta para aprendizado sobre usuários; receita direta inexistente | Horário previsível, fichas sem resgate e observação de retorno espontâneo. |
| Turma educacional com acompanhamento | Melhor hipótese inicial de receita | Conteúdo corrigido, oferta limitada ao que pode entregar, compra voluntária e obrigações comerciais resolvidas. |
| Assinatura da Academy | Posterior à turma piloto | Valor recorrente e retenção demonstrados; não cobrar só por promessa de conteúdo futuro. |
| Ferramenta de treino para professores ou clubes | Alternativa B2B a investigar | Cliente real e escopo contratual delimitado; venda e suporte podem ser mais lentos. |
| Parceria com operador | Possibilidade futura | Parceiro verificado, responsabilidades claras, análise jurídica e margem após dependência do fornecedor. White-label não é dispensa automática de obrigações. |
| Sala própria de dinheiro real com rede | Baixa adequação neste momento | Enquadramento, capacidade operacional, pagamentos, integridade, capital e liquidez demonstrados. |

O fato de você ser CLT não diminui o valor do projeto. Significa que tempo, caixa e capacidade de responder a incidentes são restrições de projeto. Uma turma em horário marcado pode caber nessa realidade; uma sala financeira 24 horas exige outra organização.

Eu preservaria o emprego e não assumiria dívida ou garantia de prêmios para validar esta hipótese. Definiria primeiro um teto de tempo e gasto que caiba nas finanças pessoais, sem usar reserva de emergência nem dinheiro custodiado. Não há informação suficiente para fixar um valor adequado ao seu orçamento.

### 13.11 Próximos 90 dias: proposta de validação

Esta é uma recomendação para decisão do fundador. Não substitui o backlog de `DASHBOARD.md` nem altera os percentuais do plano vigente. As metas são critérios propostos para o experimento, não benchmarks comprovados de mercado.

| Período | Trabalho principal | Evidência para decidir |
|---|---|---|
| Dias 1–14 | Clarificar operação PM/Real; obter consulta jurídica focada; revisar os erros apontados; corrigir overflow e formatação; escolher uma primeira jornada de aprendizado. | Explicação única do que o produto oferece, cinco iniciantes conseguindo completar a jornada observada e nenhum erro factual crítico nas aulas escolhidas. |
| Dias 15–30 | Pilotar com 10–15 adultos convidados, uma modalidade e horário fixo; observar dúvidas e dificuldades sem remuneração por recrutamento. | Contagens reais de presença, retorno, conclusão e erros; entrevistas individuais, inclusive com quem desistiu. |
| Dias 31–60 | Oferecer uma turma pequena paga, somente após revisão e estrutura comercial mínima, com conteúdo, calendário e suporte definidos. | Compras voluntárias, incluindo pessoas fora da família imediata; razões de compra, pedidos de reembolso e horas de atendimento. Pagamento para “ajudar o fundador” não valida a oferta. |
| Dias 61–90 | Repetir apenas se houver aprendizado e margem; testar indicação direta após medir a aquisição inicial. | Retorno ou renovação, custo por aluno, receita líquida, ganho de aprendizado em problemas novos e carga de trabalho compatível com o emprego. |

Eventos a medir: convite recebido voluntariamente, cadastro, e-mail confirmado, primeira aula, primeiro exercício, primeira sessão PM, retorno na semana seguinte e compra. Registrar quantidade e denominador; com 10 pessoas, uma mudança de um participante altera 10 pontos percentuais. Não vender pequenas oscilações como prova estatística.

Sinal para continuar: pessoas voltam sem insistência, entendem o serviço e compram pelo aprendizado. Sinal para ajustar: gostam da convivência, mas não percebem valor no curso. Sinal para interromper expansão: necessidade de subsídio permanente, pressão para jogar/recrutar, erros de conteúdo persistentes ou suporte incompatível com sua rotina.

### 13.12 Prioridades e pendências documentais

**Antes de cobrar pela Academy:** corrigir matemática e conceitos; revisar todos os gabaritos; tornar explícito o que é conteúdo introdutório e o que é regra própria; verificar vídeos, legendas e leitura mobile. Priorizar uma trilha curta correta, em vez de produzir mais episódios com o mesmo processo de revisão.

**Antes de qualquer proposta de dinheiro real:** confirmar enquadramento jurídico e fluxo financeiro; conciliar documentação e comportamento; estabelecer responsabilidades e capacidade de atendimento. As fontes consultadas não sustentam atalhos por convite, amizade, PIX manual ou mudança de nome para “clube”.

Pendências identificadas nos documentos existentes, para correção pelos respectivos donos:

- `PLANO_GO_TO_MARKET_REDE_2_NIVEIS.md:149` chama ledger e painel de implementados; `:246` ainda os chama de modelo.
- A qualificação é 100 mãos em `:111`, mas o checklist em `:276` cita 50 mãos ou volume de rake.
- STATUS descreve carteira real e depósito manual; plano e trechos das regras descrevem só PM. Falta reconciliar a operação efetiva, sem presumir movimentação já realizada.
- A seção de regras financeiras ainda apresenta split B2B 15/85 ao lado do modelo de afiliados em que clube não recebe rake. É necessário explicitar escopos e impedir leitura de dupla remuneração.
- O pitch contém catálogo e afirmações de maturidade que podem envelhecer; números operacionais devem ser conferidos no STATUS, sem copiar novamente seu catálogo para esta avaliação.

**Decisão recomendada, considerando o esclarecimento do fundador:** apresentar separadamente a assinatura educacional e o acesso ao poker com fichas reais, sem exigir assinatura para jogar. Validar a qualidade de ensino e a confiabilidade financeira com critérios próprios, sem usar depósitos como receita de desenvolvimento. O piloto entre conhecidos pode demonstrar funcionamento; expansão comercial exige também evidências de retorno, margem e condições específicas da operação. Antes de oferecer o coach IA, delimitar seu uso para estudo e revisão de mãos encerradas, evitando assistência individual durante partidas contra outros participantes; esta é uma recomendação de integridade, ainda não uma funcionalidade verificada.

---

*Números e limites vigentes: [`STATUS_OPERACIONAL.md`](STATUS_OPERACIONAL.md). O pitch original é de 2026-09-04; a avaliação crítica da seção 13 é de 2026-09-17 (PIX automático DePix na demo entrou depois). Não alegar certificação de produção nem autoexclusão pronta. Recomendações desta análise não significam implementação, mudança operacional ou autorização jurídica.*

<!-- DOCUMENTATION_SYNC:START -->
> **S24** (2026-09-21) — demo `zerotiltpoker.net` · sem certificação de produção · PIX automático ligado (DePix reconciliado).
> Fatos (catálogo, carteiras, limites): [`STATUS_OPERACIONAL.md`](STATUS_OPERACIONAL.md).
<!-- DOCUMENTATION_SYNC:END -->
