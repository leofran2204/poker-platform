# Validação de situações reais de poker

**Dono da campanha manual autorizada.** Gates de rotina: [QUALITY.md](QUALITY.md). Catálogo: [STATUS_OPERACIONAL.json](STATUS_OPERACIONAL.json). Executores: [scripts/README.md](../scripts/README.md).

## Método e limites

A autorização desta campanha foi dada pelo proprietário. O perfil não roda em push, PR ou `cargo test` comum. Escopo: jogo, torneios, precisão de equity e integridade. Quizzes, audiovisual, ativação comercial e provedores de pagamento reais ficam fora.

Os wrappers `scripts/full-validation.ps1` e `.sh` usam o mesmo executor `full-validation.py`. Ele compila antes de iniciar o orçamento de **até 60 minutos**, usa no máximo **dois trabalhadores de testes/simulação**, cria rede Docker exclusiva e PostgreSQL/Redis exclusivos e publica somente portas de loopback. Não lê a configuração da demo nem herda `DATABASE_URL`, origens públicas ou chaves de provedores. PIX é mock; e-mail é log. Nenhuma carga chega à demo.

O executor aplica as migrations existentes em transações, sem modificá-las, e registra seus checksums no banco descartável. Guarda um dump antes de remover seus containers. Portas reservadas: PostgreSQL 5549, Redis 6399, API 3189 e HTTPS/WSS 3449. Conflito de porta é falha, nunca motivo para reutilizar um serviço existente.

O cliente de rede exige Node nativo no Linux/WSL, disponível no `PATH` ou no cache ignorado `artifacts/full-validation/runtime/node`. O Node Windows não é usado nessa etapa: o encaminhamento Windows→WSL reúne os endereços de origem em um único IP. O cache local foi obtido da distribuição oficial Node 22 e conferido contra seu manifesto SHA-256; a origem fica em `runtime/node-source.json`. O workflow provisiona Node Linux via setup-node.

```powershell
.\scripts\full-validation.ps1 -Approved
# Execução parcial: não equivale à aprovação da campanha completa.
.\scripts\full-validation.ps1 -Phase motor -Minutes 60 -Approved
```

```bash
FULL_VALIDATION_APPROVED=1 bash scripts/full-validation.sh all
FULL_VALIDATION_APPROVED=1 bash scripts/full-validation.sh motor
FULL_VALIDATION_APPROVED=1 bash scripts/full-validation.sh api
FULL_VALIDATION_APPROVED=1 bash scripts/full-validation.sh gateway
```

O workflow **Full Validation (Manual Authorization)** usa um único job com o mesmo executor. O teto do job inclui a compilação; o teto interno começa depois dela. O workflow foi editado localmente, não executado no GitHub nesta entrega.

## O que é preparado

| Camada | Situações e evidência |
|---|---|
| Motor determinístico | Ranking/variantes, mínimos, reabertura, devolução não coberta, rake, empates, centavos restantes, potes e deflator. Regressões existentes são preservadas e inventariadas. |
| Cash | Cinco configurações lidas do catálogo, incluindo NL 0,75/1,50; rake/caps conferidos no banco migrado; todas as ocupações de 2 até o limite. Entradas fixas e evolução legal dos stacks, substituição de jogador quebrado por nova identidade. |
| Matriz | Variante, ocupação, posição, street, stack relativo, sequência, all-in, empate e potes. Pares observados e ausentes por configuração; exclusões estruturais explícitas. Pares sem testemunha permanecem lacunas, não são declarados impossíveis. |
| Lotes | 1.000 mãos; políticas seguintes favorecem sequências ausentes. Parada após três lotes sem nova cobertura ou 20.000 mãos/configuração. Seed, baralho completo, ações, entradas e liquidações em JSONL. |
| MTT | Cinco eventos do catálogo migrado, até campeão; inscrição/cancelamento, blinds, reentrada, eliminação, movimentação e soma da premiação. Fichas e centavos são contabilizados separadamente. |
| Equity | Pares dominantes, confronto equilibrado, mão dominada, draw, multiway, referências exatas e vizinhanças dos tiers. Erro absoluto, tolerância inicial de 0,5 ponto percentual e intervalo conservador de 99% para amostra limitada a [0,1]. |
| Atores internos | Estágios de 1/5/20 mesas; configurações e frentes canônicas, timeout/desconexão/reconexão e contratos de banco existentes. Isso não comprova transporte WSS. |
| HTTPS/WSS local | Clientes com certificado local explícito, disputa de assentos, join repetido, cartas privadas, ações, reconexão, latência e validação de settlements/saldos no banco. Reinício controlado apenas do processo filho desta campanha. |

A simulação do motor não é duplicada por PM/Real. O banco/API verifica a separação. Na simulação longa, o cálculo caro do deflator fica desligado **explicitamente**: sua evidência pertence às fixtures dirigidas e à campanha de equity. Essa escolha nunca conta como cobertura do deflator nas mãos aleatórias.

A enumeração de referência usa percurso independente dos runouts, mas compartilha o avaliador de ranking, verificado por regressões determinísticas. Os casos conhecidos de turn (36/44) e empate triplo têm expectativas explícitas. A repetição de uma mesma entrada serve apenas para determinismo. O intervalo de 99% é uma garantia do desenho amostral; seed determinística não cria ensaios independentes. Se o intervalo cruza um tier, a classificação é inconclusiva, sem alterar a regra financeira.

A distribuição de mesas MTT no harness exercita os auxiliares do motor; a persistência dos atores é verificada separadamente. A ordem determinística de eliminações simultâneas no harness não certifica o desempate do coordenador em produção.

## Contabilidade e aprovação

Cada execução grava em `artifacts/full-validation/<UTC-id>/`:

- `campaign.json`, `metrics.tsv`: status por etapa, duração e contagens extraídas do libtest.
- `test-inventory.json`: teste, localização, associação a situações e resultado. Classificação por módulo não substitui leitura de assertions nem constitui prova por si só.
- `coverage-matrix.json`: fatores, pares observados/ausentes, exclusões e casos críticos.
- `catalog-db.json`, `catalog-differences.json`, `migrations.json`, `isolation.json` e dump do banco exclusivo.
- Logs, checkpoints por lote/evento/entrada de equity, traces e reprodução da falha.
- `source-fingerprints.json` nas execuções com a versão final do executor.

Zero testes, testes todos ignorados, timeout ou ausência de resumo não viram sucesso. Etapas posteriores a uma falha ficam `not_run`. Falha de regra/conservação, cartas impossíveis, vazamento ou liquidação duplicada interrompe a campanha; a reprodução repete somente o caso falho. Checkpoints preservam o progresso, mas não autorizam pular uma falha na próxima execução.

Aprovação exige situações obrigatórias cobertas, expectativas verificadas, nenhuma divergência inexplicada e nenhuma ambiguidade pendente. O cliente exige timeout observado por WSS e persistido como fold, além de repetição imediata de call com apenas uma ação no histórico. Esse teste não garante idempotência de mensagens atrasadas que reapareçam em outra vez do jogador. Resultados não executados permanecem lacunas. Resultado local não é promessa de capacidade da VPS.

## Primeira execução de 02/10/2026 (antes das correções)

**Campanha reprovada e interrompida.** Evidência principal: [summary.json](../artifacts/full-validation/20261002T061251Z-e31cc0/summary.json), [campanha](../artifacts/full-validation/20261002T061251Z-e31cc0/campaign.json) e [fixture](../artifacts/full-validation/20261002T061251Z-e31cc0/short-allin-reopening.json).

- **Um teste aprovado:** baralho inválido e raise abaixo do mínimo rejeitados.
- **Um teste falhou**, confirmado por uma reprodução mínima. As 36 etapas seguintes da versão executada ficaram não executadas.
- **Zero mãos dos lotes cash**, zero MTTs até campeão, zero entradas de Monte Carlo e zero estágios de atores/rede executados. As duas mãos da fixture são evidência dirigida, não volume de simulação.
- Inventário de 2.045 testes nos binários compilados; essa quantidade **não é contagem de testes aprovados**.
- Tempo contabilizado: 236,52 s na primeira preparação de banco + 293,29 s na execução com o caso dirigido = **529,81 s**, sem compilação. A segunda execução recebeu teto de 55 minutos para respeitar o orçamento conjunto. A preparação anterior interrompida durante o inventário não iniciou a campanha.
- Os containers exclusivos foram removidos após preservar dumps. O container local preexistente permaneceu intacto.

### Falha reproduzida: all-in curto reabre a ação

Três jogadores entram com 2.500 centavos cada. Uma mão legal anterior produz stacks de 100, 4.925 e 2.475. Na mão seguinte: p1 aumenta para 75 (incremento completo de 50), p2 paga, p0 vai all-in para 100 (incremento de 25). O motor aceita p1 aumentar novamente para 150.

A expectativa dirigida é rejeitar esse novo raise, pois p1 já agiu e não enfrenta incremento completo. Referência de reabertura: [Poker TDA 2026, regra 49](https://www.pokertda.com/view-poker-tda-rules/). A conservação total de fichas não torna essa ação legal. Não foi estimado impacto financeiro histórico.

```bash
cd Motor-Rust
CARGO_TARGET_DIR=$HOME/poker-build/motor-stress-release cargo test --release --locked \
  --test full_validation_situations short_allin_does_not_reopen_a_completed_action \
  -- --exact --nocapture
```

Na primeira execução, o teste era dirigido/ignorado na rotina e incluído no perfil manual. A expectativa não foi relaxada. Na retomada autorizada, virou regressão de rotina e o motor foi corrigido para respeitar a reabertura da ação.

### Divergência documental de reentradas PM (reconciliada na retomada)

No banco vazio após as 60 migrations, quatro eventos PM (Texas R$15, Freeroll, Omaha e Pineapple) têm `rebuy_max_count=0`, interpretado pelo motor como ilimitado. A migration 044 já autorizava essa regra; a 053 criou Texas R$25 com limite 1 também em PM. O catálogo canônico passou a representar `play_reentries` por evento e a documentação gerada exibe limites PM/Real separados. Nenhuma migration foi alterada.

Para evitar duplicar o mesmo motor por carteira, os detalhes comuns dos cinco MTTs são extraídos das linhas Real correspondentes; os limites PM também são comparados com o catálogo. Nenhum evento foi executado na primeira rodada. Na retomada, `catalog-differences.json` ficou vazio.

A fixture temporal do Loss Deflator não foi executada na primeira campanha. Na retomada, o proprietário confirmou preservar os oponentes no instante do pagamento, inclusive quem folda depois. O motor congela fase, board e cartas dos oponentes nesse instante; elegibilidade financeira continua dependendo dos potes finais. A regressão dirigida passou com dois oponentes preservados.

## Retomada e correções locais

- Reabertura separada da obrigação de responder à aposta: all-in curto não libera novo raise ou shove para quem já agiu; incrementos curtos cumulativos podem reabrir. Check em rua sem aposta mantém o direito de aumentar uma abertura posterior. A API cash/MTT reflete isso nas ações legais.
- Premiação inclui posições pagas já eliminadas. Com menos inscritos que posições previstas, normaliza os pesos proporcionalmente, conforme decisão do proprietário em 02/10/2026. Rateio com inteiros e maiores resíduos conserva todos os centavos.
- Finalização de torneio grava créditos, posições premiadas, status e auditoria em uma transação com bloqueio da linha do evento. Falha deixa o estado em andamento para nova tentativa; repetição não credita novamente.
- As três fixtures que usavam `Bet` no big blind foram corrigidas para `Raise`; as antigas expectativas de pagar apenas o campeão agora exigem todas as posições premiadas e conservação do total.
- O lote Short Deck revelou small blind all-in mantido como próximo a agir. Corrigido o salto para jogador ativo, com regressão a partir de duas entradas legais de 2.500 e mão anterior que deixa 23 centavos. Quando todos estão all-in nos blinds, o harness chama o mesmo runout disponível aos atores.
- Outro bloqueio era do gerador: stacks profundos sustentavam mais de 500 aumentos mínimos legais, classificados erroneamente como travamento. A política agora limita a quantidade de raises gerados por mão e mistura apostas, pagamentos e folds para explorar as lacunas.
- Verificação inicial após essas correções: 1.859 testes de biblioteca do motor e 12 casos dirigidos aprovados; 31 testes massivos permanecem ignorados na rotina. Isso não substitui o resultado da campanha completa.

## Verificação da implementação

Oito contratos do executor verificam contagem efetiva, etapa pendente, no-op/ignorados, interrupção por falha, encerramento do processo em timeout, descarte de credenciais externas, deadline informado pelo teste e preservação dos artefatos originais durante reprodução. Também foram conferidas compilação/clippy dos testes Rust e sintaxe dos clientes. Fixtures e transportes não alcançados não são declarados validados em runtime.

## Evidência consolidada da retomada — 03/10/2026

**Cobertura incompleta; sem aprovação da campanha.** A execução [20261002T135136Z-6cbf06](../artifacts/full-validation/20261002T135136Z-6cbf06/campaign.json) completou 36.000 mãos cash (11.000 NL 0,25; 6.000 NL 0,75/1,50; 11.000 Short Deck; 8.000 Omaha) e quatro MTTs até campeão, em 234 mãos de torneio. Os quatro eventos pagaram o prize pool integral. As contagens são dessa execução; repetições anteriores não são somadas como cobertura nova.

O lote cash Pineapple parou por deadline. A versão executada marcou o teste como falho e iniciou reprodução, que sobrescreveu o checkpoint parcial; por isso nenhuma quantidade desse lote é contabilizada. A versão final classifica esse caso como `incomplete`, não reproduz deadlines e usa diretórios separados para reproduções. Os relatórios originais foram preservados. O executor agora registra duração monotônica e duração de relógio, pois a expiração por relógio não coincide necessariamente com o tempo monotônico após suspensão/ajuste do sistema.

As duas tentativas anteriores registraram 406,26 s e 409,01 s, interrompidas respectivamente pelo turno do small blind all-in e pelo limite artificial de ações. A terceira registrou 434,38 s monotônicos e deadline expirado no lote Pineapple. Não foi iniciada outra campanha massiva após a expiração. A preparação que falhou na compilação não executou cenários.

Pendências: concluir cash/MTT Pineapple, executar a campanha de precisão de equity e os estágios completos de atores, banco, HTTPS/WSS e reinício. A matriz ainda tinha pares ausentes. Os últimos ajustes de geração (evolução HU para stack curto, mistura de raises/all-ins/folds), distinção entre side pot disputado e devolução não coberta, testemunhas próximas aos tiers e clientes de rede foram compilados/verificados estaticamente, mas **não receberam nova campanha completa**. Não inferir aprovação a partir de compilação.

Checks de rotina finais: 1.859 testes de biblioteca do motor; 12 regressões dirigidas; 79 testes de biblioteca da API; 14 testes do sincronizador de documentação; oito contratos do executor; clippy de motor/API sem warnings. Separadamente, o [contrato PostgreSQL da premiação](../artifacts/full-validation/20261003T043904Z-63e10c/routine-prize-transaction-contract.log) passou em banco descartável: rollback integral, duas finalizações concorrentes, repetição e isolamento PM/Real. É um contrato dirigido, não substitui a campanha API/rede.

Rede de teste: bridge exclusiva e portas somente em loopback, pois Docker Desktop não publica portas de bridge `--internal`. Clientes sintéticos usam endereços distintos de loopback e o Caddy local fornece os headers de proxy; os limites por IP da API permanecem os mesmos. Essa alteração de transporte continua aguardando execução integrada. Containers exclusivos foram removidos após preservar os dumps.

Os arquivos novos têm propósitos delimitados dentro deste dono: executor Python comum aos wrappers; cliente de transporte real; testes do executor; fixtures dirigidas e precisão de equity. Não há segundo documento de status ou qualidade.

## Nova autorização de 03/10/2026 — resultado final incompleto

O proprietário autorizou mais 60 minutos, mantendo dois trabalhadores. A primeira tentativa, [20261003T045932Z-cac053](../artifacts/full-validation/20261003T045932Z-cac053/campaign.json), terminou em 367,58 s monotônicos: passou cash/MTT/equity/banco, mas reproduziu uma expectativa antiga de remoção imediata do assento após desconexão. A fixture foi corrigida para enviar `Disconnect`, exigir reserva do assento com as mesmas fichas e confirmar reconexão sem duplicação. A regra de produção não foi alterada.

A continuação, [20261003T051020Z-cb15a3](../artifacts/full-validation/20261003T051020Z-cb15a3/campaign.json), é a evidência principal desta autorização:

- **25.000 mãos cash:** 6.000 NL 0,25; 5.000 NL 0,75/1,50; 5.000 Short Deck; 5.000 Omaha; 4.000 Pineapple. Cinco MTTs até campeão, com **309 mãos** e premiação integral. Repetições de outras tentativas não são somadas como cobertura nova.
- **440 pares globais cobertos.** Quatro configurações sem pares pendentes; Pineapple ainda sem `check_raise × stack medium`. A fixture final prepara esse stack por duas mãos legais, mas a alteração posterior não chegou a ser executada na campanha.
- **Sete referências de equity aprovadas**, erro máximo de 0,08399 ponto percentual frente à tolerância de 0,5; quatro vizinhanças dos tiers com testemunhas. A referência compartilha o avaliador de ranking; os limites metodológicos descritos acima permanecem.
- **21 contratos de banco/Redis aprovados**, além dos testes de rotina da API. Atores internos com 1/5/20 mesas e desconexões aprovados.
- **HTTPS/WSS com uma mesa aprovado:** 23 requisições, 28 ações, reconexão, fold por timeout, repetição imediata de call, privacidade, HMAC e conservação persistida. A etapa com cinco mesas falhou com HTTP 429; vinte mesas e reinício não foram executados.

Um diagnóstico posterior de apenas duas conexões locais confirmou a causa ambiental do agrupamento de IPs: o Node Windows solicitou origem `127.1.23.1`, mas o servidor WSL observou `127.0.0.1`; o Node Linux preservou `127.1.23.1`. Evidência: [loopback-peer-diagnostic.json](../artifacts/full-validation/loopback-peer-diagnostic.json). O executor passou a exigir runtime Linux; limites da API permanecem iguais. Isso corrige a configuração do teste, mas ainda não comprova a carga integrada de cinco/vinte mesas.

A tentativa final, [20261003T153704Z-ed6cd9](../artifacts/full-validation/20261003T153704Z-ed6cd9/campaign.json), recebeu teto reduzido de 35 minutos (a anterior tinha 45). Houve grande diferença entre relógios durante a retenção das ferramentas: **93,07 s monotônicos e 29.824,56 s de relógio**. O deadline expirou antes da primeira mão cash dessa tentativa. O executor marcou `incomplete`, preservou o checkpoint com zero mãos, não reproduziu o timeout e removeu os containers. A causa da diferença entre os relógios não foi determinada. Não foi iniciada outra carga após a expiração.

**Pendências finais:** executar a última fixture Pineapple e confirmar a matriz individual; HTTPS/WSS com clientes Linux em cinco/vinte mesas; reinício controlado. Checks de rotina finais: oito contratos do executor, sintaxe, formatação e clippy aprovados; diagnóstico de rede isolado aprovado. [Resumo consolidado](../artifacts/full-validation/authorized-20261003-summary.json). Trabalho local, sem commit, push, deploy ou tráfego na demo; esta autorização não produziu aprovação integral da campanha.


## Campanha focada da estrutura híbrida — 04/10/2026

Autorização: **15 minutos, dois trabalhadores**. Perfil `pineapple`: fixtures da regra, cash Pineapple, MTT Pineapple até campeão, contrato motor/atores/WS, HTTPS/WSS com 5/20 mesas Pineapple e reinício controlado. Compilação precede o orçamento, conforme o executor; preparação de infraestrutura e execução contam no teto. Deadline interrompe a carga e preserva etapas pendentes, sem iniciar outra campanha.

```powershell
.\scripts\full-validation.ps1 -Phase pineapple -Minutes 15 -Approved
```

Os geradores consultam `GameLoop::legal_actions` e registram a ação efetivamente executada. A matriz desta campanha abrange apenas Pineapple; o restante não é revalidado por inferência. O cliente HTTPS/WSS confere os limites usando uma fórmula independente, exerce aumentos e verifica `betting_rule_version` no settlement persistido. Todos os resultados Pineapple das campanhas anteriores desta página são **evidência da regra anterior**, sem aproveitamento como cobertura da estrutura híbrida. Artefatos antigos permanecem intactos.


### Resultado consolidado da estrutura híbrida

**Escopo focado aprovado localmente**, com [resumo e referências de evidência](../artifacts/full-validation/pineapple-hybrid-20261004-summary.json). Foram contabilizados **577,983 segundos** (9 min 38 s), tomando o maior valor entre relógio e duração monotônica de cada tentativa; compilação excluída. Sempre dois trabalhadores. Não se somam repetições como cobertura nova.

- [Primeira tentativa](../artifacts/full-validation/20261004T123446Z-43435c/campaign.json): interrompida pelo verificador de retry que confundia dois calls legais de 50 centavos, separados por raise, com duplicação. O dump original foi preservado. Corrigida a associação por `hand_id` e contribuição anterior na rodada; o contrato continua rejeitando pagamento repetido ilegal.
- [Integração](../artifacts/full-validation/20261004T124428Z-42c8ca/campaign.json), com teto reduzido de dez minutos: todos os estágios passaram. **MTT Pineapple até campeão em 75 mãos**, 17 reentradas e R$ 520 integralmente premiados. **HTTPS/WSS com 5/20 mesas**, respectivamente 85/340 requisições e 89/500 ações: zero erro inesperado, reconexões, privacidade, timeout, retry imediato, limites legais, HMAC, versão da regra e conservação persistida aprovados. Reinício controlado aprovado. A matriz cash ainda tinha uma lacuna: `sequence=allin|street=Preflop`.
- [Complemento cash](../artifacts/full-validation/20261004T125322Z-741b6f/campaign.json), perfil `pineapple-cash`, teto reduzido de seis minutos: gerador passa a priorizar all-in pré-flop quando permitido para stacks curtos alcançados pela evolução legal das mãos. **6.000 mãos, 385 pares observados, zero par pendente**. Essa é a única contagem cash adotada no consolidado; as 7.000 mãos de cada tentativa anterior não são adicionadas. Não foi necessário repetir rede ou torneio já aprovados.

Os hashes das fontes de produção da integração conferem com a entrega; apenas a política do gerador cash mudou no complemento. Bancos, dumps, traces e relatórios são locais e exclusivos. Containers temporários removidos; container preexistente preservado. A aprovação cobre esta regra e este perfil, sem certificação de produção ou nova campanha completa de equity/demais variantes. Publicação futura continua condicionada a não haver mãos Pineapple abertas nem torneios Pineapple em andamento.

<!-- DOCUMENTATION_SYNC:START -->
> **S26** (2026-10-09) — demo `zerotiltpoker.net` · sem certificação de produção · PIX automático ligado (DePix reconciliado).
> Fatos (catálogo, carteiras, limites): [`STATUS_OPERACIONAL.md`](STATUS_OPERACIONAL.md).
<!-- DOCUMENTATION_SYNC:END -->
