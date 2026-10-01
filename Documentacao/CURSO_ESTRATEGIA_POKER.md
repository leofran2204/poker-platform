# Curso de Estratégia de Poker — Zero Tilt (conteúdo exclusivo da plataforma)

> Produto educacional: teoria, exercícios e prática por módulo nas quatro modalidades da plataforma.
> Versão interativa atual: `Frontend-Web/src/data/courseContent.json`, rota `/curso`. O quiz atual compara respostas com gabaritos locais; não consulta um solver nem o motor Rust para avaliar estratégias.

## Reformulação integral — diretriz de 27/09/2026

O proprietário ampliou a solicitação: pesquisa histórica, acadêmica e técnica do básico ao avançado; aulas completas e detalhadas; teoria e exercícios; **uma mesa simuladora por módulo, com bots e situações alinhados ao conteúdo**. A duração deve seguir o objetivo pedagógico. Vídeos curtos de apresentação são peças de entrada, não substituem as aulas aprofundadas.

### Entrega S26

- Home reformulada; filme v3 de 81,70 segundos explica Brazilian Pineapple e Texas Hold’em Short Deck. Legendas, transcrição e duração são derivadas do mesmo roteiro/manifesto. Publicação e evidências operacionais são registradas em `DEVELOPMENT_LOG.md`.
- Os 25 vídeos técnicos foram regravados e se somam aos três históricos: as 28 aulas atuais têm vídeo em 1080p30, legendas, transcrição e capítulos. Roteiros e renderizador em `ZeroTiltCurso/editorial/`; finais em `Frontend-Web/public/videos/`.
- Vinte e uma referências vinculadas em `courseSources.json`. A grade tem 28 aulas, 65 questões e dez agrupamentos; os IDs existentes e o progresso concluído foram preservados. Os 25 textos técnicos foram ampliados com exemplos resolvidos, hipóteses e orientação de laboratório.
- **História do Omaha — 30/09/2026:** a aula `m0l2` integra testemunhos de Ciaffone/Turner, Golden Nugget em 1982, a incerteza do nome, Limit Omaha na WSOP em 1983, Pot-Limit em 1984 e o modelo acadêmico de Ho (2015). Seis fontes e duas perguntas adicionadas; vídeo histórico de 253,93 segundos, com 13 capítulos. O curso específico de Omaha permanece para outro momento, conforme orientação do proprietário.
- **Implementação de 28–30/09/2026:** nova ordem por pré-requisitos, ampliação dos textos/gabaritos, regravação dos 25 vídeos técnicos e mesas dos dez módulos com 35 cenários, bots por perfil, ações completas e revisão. A expansão curricular de 15 etapas descrita abaixo continua como arquitetura de aprofundamento; esta entrega corresponde às 28 aulas atuais. Evidências e limites da conferência em `QUALITY.md`.
- Os rascunhos duplicados com recomendações absolutas foram removidos deste documento. O conteúdo de cada aula tem um único local de edição: `courseContent.json`. As dicas locais de `/dicas` também são derivadas dessa fonte; `tipsContent.json` foi retirado. Vídeos antigos foram retirados da pasta pública; os originais em `ZeroTiltCurso/epNN-*/` e o histórico git preservam a rastreabilidade.

### Estrutura didática obrigatória de cada módulo

1. Objetivo observável, modalidade, nível e pré-requisitos.
2. Teoria desenvolvida em unidades: definição, motivo, condições de uso, limites e fontes.
3. Exemplos resolvidos passo a passo; depois, situações parecidas com uma variável alterada.
4. Exercícios de regras, cálculo e decisão; explicação tanto da resposta quanto das alternativas.
5. Mesa do módulo com bots: treino guiado, repetição com variações e prática sem dicas.
6. Revisão após a decisão ou mão, com replay, informação disponível na ocasião e justificativa.
7. Avaliação por competência e revisão posterior. Ganhar uma mão não é critério suficiente para aprovar uma decisão.

### Grade de trabalho e mesas correspondentes

Esta é a arquitetura curricular da reformulação; não um catálogo de funcionalidades já disponível.

| Etapa | Conteúdo a desenvolver | Situações da mesa simuladora |
|---|---|---|
| 1. Começar com segurança | Interface, fichas de treino, modalidades, limites e diferença entre resultado e decisão | Bots demonstram turnos, ações legais e encerramento da mão; sem pressão de tempo |
| 2. Regras e leitura de mãos | Blinds, posição, streets, ranking clássico, empates, kickers, all-in e potes paralelos | Showdowns reproduzíveis; aluno monta as cinco cartas e identifica elegibilidade por pote |
| 3. Matemática fundamental | Frações do pote, pot odds, equidade, EV, outs limpos, variância e hipóteses | Bots variam preço da aposta; comparação de call/fold com ranges declarados no exercício |
| 4. Hold’em pré-flop | Posições 9-max/6-max/heads-up, stacks efetivos, ranges, limps, isolamento, 3-bet/4-bet e rake | Perfis que abrem, pagam, fazem limp ou reaumentam; treino IP e OOP |
| 5. Hold’em flop | Texturas, vantagem de range e de nuts, SPR, c-bet, check, sizings, multiway | Mesmo range em boards diferentes; bots dão check, pagam e fazem check-raise conforme perfil |
| 6. Hold’em turn | Runouts, double barrel, cartas que mudam vantagem, realização de equidade, controle do pote | Bot altera continuidades por turn; aluno planeja turn e river, sem conhecer cartas futuras |
| 7. Hold’em river | Valor fino, polarização, bluff-catchers, blockers, overbets, MDF e seus limites | Ranges com composições diferentes de valor/blefe; feedback condicionado às hipóteses |
| 8. Short Deck — fundamentos | 36 cartas, seleção das cinco, ranking ZT e A-6-7-8-9 | Bot apresenta trinca contra sequência e flush contra full house; nenhuma carta 2–5 |
| 9. Short Deck — estratégia | Combinatória própria, equidades, posição, draws, stacks e ajustes | Cenários recalculados com 36 cartas; não reutilizar percentuais de Hold’em de 52 cartas |
| 10. Omaha 4 — fundamentos | Quatro privadas, exatamente 2+3, ranking clássico e estrutura de aposta aplicável | Bot explora erros de um único ás do naipe e falsas quadras; aluno seleciona 2+3 |
| 11. Omaha 4 — estratégia | Conectividade, naipes, nut potential, blockers, redraws, wraps, SPR e multiway | Bots com draws dominados, nuts e redraws; cálculo com as quatro cartas conhecidas |
| 12. Brazilian Pineapple — fundamentos | 2 privadas inicialmente, +1 após cada street, sem descarte, 2+3 e ranking clássico | Mesma mão progride com 2/3/4/5 privadas; foldados não recebem; all-in recebe |
| 13. Brazilian Pineapple — estratégia | Atualização das possibilidades a cada carta privada, seleção 2+3 e incerteza futura | Bots e aluno recebem extras legais; reavaliar a mão em cada street sem antecipar as extras |
| 14. Cash, torneios e regras ZT | Rake, Loss Deflator, side pots, blinds, stacks; chip EV versus valor de premiação e introdução a ICM | Cenários específicos de cash/MTT; modelo econômico e premiação sempre explícitos |
| 15. Estudo avançado e revisão | Equilíbrio, exploração, frequências, limites de solver, nodelocking, amostra e plano de estudo | Alternância de perfis; comparar decisões em cenários novos e revisar erros recorrentes |
| Complemento: história e ciência | Fontes materiais, Cowell, WSOP, pesquisa em jogos de informação imperfeita | Experimento de informação privada e exercícios de evidência; não bloquear regras básicas por história |

Sequência: fundamentos comuns → trilha da modalidade escolhida → aprofundamento → aplicações e revisão. A biblioteca histórica fica acessível como complemento. Não exigir que o aluno termine Hold’em avançado para conhecer Omaha ou Pineapple. Preservar os IDs das aulas e o progresso existente ao reorganizar; separar numeração de exibição, ID de aula e nome do arquivo de produção.

### Regras específicas: a plataforma prevalece

| Modalidade ZT | Baralho / privadas | Formação / ranking |
|---|---|---|
| Texas Hold’em | 52 / 2 | Melhor cinco entre sete; pode usar 0, 1 ou 2 privadas; ranking clássico |
| Texas Hold’em Short Deck | 36, de 6 a A / 2 | Melhor cinco; trinca > sequência; flush > full house; sequência baixa A-6-7-8-9 |
| Omaha 4 | 52 / 4 | Exatamente 2 privadas + 3 comunitárias; ranking clássico |
| Brazilian Pineapple | 52 / 2, depois 3/4/5 | Exatamente 2+3; ranking clássico; sem descarte; extras também para all-in |

Fontes normativas internas: `BUSINESS_RULES.md`, `Motor-Rust/src/deck.rs`, distribuição em `game.rs` e catálogo no STATUS. Referências externas de Omaha, Short Deck ou Pineapple precisam ser confrontadas com essas regras. Não anunciar Omaha Short Deck, não ensinar descarte no Brazilian Pineapple e não assumir que toda mesa Omaha implementa a estrutura de apostas de um site externo.

### Arquitetura das mesas de estudo

Implementação de 28/09/2026: `courseTraining.json` define 35 cenários para os dez módulos e cobre todas as 28 aulas. `/curso/mesa/:moduleId` usa `POST /api/academy/play`, com o `GameLoop` Rust e sem estado financeiro. Os bots operacionais do lobby permanecem separados das políticas didáticas.

| Módulo | Cenários implementados |
|---|---|
| Fundamentos | Ranking/kickers, posição, primeira mão, raise/fold, total do aumento, potes paralelos |
| Brazilian Pineapple | Distribuição progressiva, flop, combinações no turn, board que não pode ser usado inteiro, extras após all-in |
| Texas Short Deck | A6789, flush no ranking ZT, denominador de 36 cartas |
| Omaha 4 | Combinações 2+3, um único ás do naipe, mão feita e possibilidades futuras em pote multiway |
| Pré-flop | Abertura, limp do small blind, resposta a uma abertura |
| Flop | Textura conectada, check-raise por valor do bot, pote multiway |
| Turn | Carta que muda a análise, controle do pote, preços de call variados |
| River | Valor, bluff-catcher, leitura de flush antes de escolher tamanho |
| Consolidação | Hipóteses de EV, limite da sessão, revisão integrada |
| História opcional | Informação privada, blefe, processo de decisão |

As cartas que definem cada exercício são fixadas antes da mão. Nova seed varia adversários, cartas futuras não fixadas e decisões aleatórias; trocar de cenário varia a situação/posição. Há replay, comparação a partir de uma decisão anterior, caderno, exportação e reabertura de sessões locais. O modo sem dicas oculta o auxílio durante a decisão. A correção objetiva usa cartas atuais, preço, contribuições e regras; não atribui nota GTO nem finge calcular equity sem modelo.

- Um simulador compartilhado com configurações por módulo; uma experiência de mesa própria para cada módulo. Reutilizar componentes visuais e regras verificadas do motor, evitando quatro motores divergentes.
- Sessões educacionais isoladas de carteiras, com fichas sem valor, sem depósitos, saques, rake cobrado ou comissões de agente. Simulações de rake/deflator devem ser identificadas como cálculos didáticos.
- Contrato de cenário: ID/versão, módulo, variante, objetivo, estado inicial legal, seed, posições, stacks, potes, histórico, política de cada bot, decisões avaliadas, fontes, critérios e explicações.
- Bots pedagógicos com perfis configuráveis e política de ação compatível com a informação que possuem. O gerador pode selecionar o cenário antes da mão; não pode rearranjar cartas durante a mão para forçar um resultado.
- Treino guiado explica antes e depois; desafio avalia sem dicas durante a decisão; prática variada percorre os cenários do módulo. Perguntas objetivas acompanham ações reais de uma mão completa.
- Reiniciar o cenário, repetir a mesma distribuição, usar nova seed ou escolher outra situação/posição; comparar alternativas e consultar replay. Feedback distingue regras exatas e contas sob hipóteses; nenhuma nota estratégica ótima é alegada.
- Nenhuma consulta paga a LLM ou solver por ação por padrão. Produzir cenários e referências previamente, executar políticas locais/servidor com limites de recursos e cachear avaliações reutilizáveis. Estimar custo por sessão antes de escalar.
- Não chamar os bots de GTO sem prova específica. Não apresentar frequências, EV ou equidade como exatos quando forem estimativas. O coach planejado para mãos reais continua separado e exclusivamente pós-mão.
- O caderno local registra cenário, seed, ações, respostas e anotações, com reabertura/exportação de até 50 sessões. As perguntas da mesa são corrigidas no servidor; o endpoint de progresso do quiz de aula ainda recebe a nota informada pelo cliente. Nenhum dos dois contratos constitui certificação de competência estratégica.

### Pesquisa: profundidade e atualização

Consulta bibliográfica iniciada em 27/09/2026. Registrar autor, data, seção/página, modalidade, hipóteses, nível de evidência e uso no curso. Priorizar documentos primários, artigos e material técnico dos autores. Conteúdo de fornecedores é referência técnica com interesse comercial, não evidência acadêmica independente. Publicação recente não substitui validação.

| Fonte consultada | Uso e limite |
|---|---|
| [The Met — The Cloisters Playing Cards](https://www.metmuseum.org/art/collection/search/475513) | Objeto e data aproximada; imagem em domínio público. Não documenta a invenção do poker. |
| [Joe Cowell, 1844, p. 94](https://archive.org/details/thirtyyearspasse00cowe/page/n97/mode/2up) | Memórias e relato de poker. A edição contém uma inconsistência entre o número de cartas mencionado e os valores enumerados; não repetir esse número sem crítica da fonte. |
| [Kuhn, 1950, pp. 97–103](https://sites.math.rutgers.edu/~zeilberg/akherim/PokerPapers/Kuhn1951.pdf) | Modelo de três cartas, informação privada e estratégia. O nome do PDF inclui 1951; a referência da obra é 1950. |
| [MIT — Basic Strategy, 2015](https://ocw.mit.edu/courses/15-s50-poker-theory-and-analytics-january-iap-2015/resources/mit15_s50iap15_l3_basic/) | Pot odds e EV; exemplos originais com hipóteses explícitas. Não copiar slides/vídeos sob licença não comercial. |
| [Bowling et al., Science, 2015](https://pubmed.ncbi.nlm.nih.gov/25574016/) | Resumo consultado; resultado restrito a heads-up limit Hold’em. |
| [Brown e Sandholm — Pluribus, Science, 2019](https://noambrown.github.io/papers/19-Science-Superhuman.pdf) | Pesquisa em no-limit com seis jogadores; resultado experimental não valida automaticamente nossos bots. |
| [Zinkevich et al. — CFR, 2007](https://poker.cs.ualberta.ca/publications/NIPS07-cfr.pdf) | Fundamentos de minimização de arrependimento e aproximações em jogos de informação imperfeita. Garantias dependem das hipóteses do jogo. |
| [Palomäki et al., 2013](https://researchportal.helsinki.fi/fi/publications/this-is-just-so-unfair-a-qualitative-analysis-of-loss-induced-emo/) | Resumo de estudo qualitativo, 60 participantes; não causal, não teste de tratamento. |
| [Dunlosky et al., 2013](https://www.wku.edu/senate/documents/improving_student_learning_dunlosky_2013.pdf) | Recuperação ativa e estudo distribuído; aplicação pedagógica ao poker é nossa inferência. |
| [WSOP — história oficial](https://www.wsop.com/about/world-series-of-poker/) e [Moss](https://www.wsop.com/players/162565666/johnny-moss/) | Datas e formatos; separar testemunho institucional de alegações gerais de retorno. |
| [PokerStars — PLO Rules](https://www.pokerstars.com/poker/learn/lesson/plo-rules/) | Confirma a regra 2+3. Estrutura pot-limit depende da mesa e precisa corresponder ao motor ZT. |
| [GTO Wizard — Multiway Preflop Solving, 03/02/2026](https://blog.gtowizard.com/introducing-multiway-preflop-solving/) | Referência técnica atual sobre parametrização por rake, stacks, limps e perfis. Alegações de desempenho são do fornecedor. Não foi contratada assinatura nem reutilizada sua base proprietária. |

### Correções aplicadas na auditoria de 27/09/2026

- Gabaritos editoriais são identificados como critérios, sem alegar execução do motor. Showdowns pedem classificação ou vencedor.
- Omaha exibe quatro privadas e diferencia draw de mão feita; o kicker da trinca de ases no exemplo Pineapple é Q/2, respeitando exatamente 2+3. Casos reproduzidos em testes do motor.
- Removidas regras absolutas e promessas de lucro, corrigidos pot odds, posição, ações legais, blockers e leitura de flush. Exemplos estratégicos declaram hipóteses e objetivos.
- Ranges dependem da modalidade, número de lugares, stacks, sizings, rake e oponentes. Tabelas ilustrativas não são chart universal nem política ótima do motor.
- Legendas, cartazes numerados e chamadas “próximo episódio” precisam acompanhar a grade final. Não renumerar apenas a lista e manter áudio apontando para aula errada.

### Critérios para declarar um módulo completo

Teoria e fontes revisadas; exemplos e exercícios específicos da variante; vídeo/legendas/transcrição concordantes; mesa funcional com ações legais e bots adequados; repetição variada; feedback que respeita informações ocultas; avaliação por objetivo; navegação e pré-requisitos coerentes. A entrega será incremental por módulos completos. Não declarar o curso inteiro concluído após trocar a aparência ou produzir apenas a abertura.

## Grade e publicação audiovisual

| Grupo | Aulas | Acesso |
|---|---|---|
| Fundamentos | m0l4–m0l8 | Regras primeiro; primeiros passos disponível separadamente |
| Brazilian Pineapple | m5l3, m5l4, m5l5 | Regras comuns → distribuição → combinações/probabilidades → showdown/equity |
| Texas Short Deck | m5l1 | Após regras comuns |
| Omaha 4 | m5l2 | Após regras comuns |
| Texas pré-flop, flop, turn, river | m1–m4 | Progressão a partir dos fundamentos |
| Consolidação | m6 | EV após matemática; orçamento disponível desde o início |
| História opcional | m0l1–m0l3 | Acesso independente, inclusive visitante |

Conclusões anteriores mantêm acesso mesmo se a ordem mudar. IDs inexistentes não liberam aula. Quiz e estado do player reiniciam ao mudar de aula. O backend atual registra a nota declarada pelo cliente; isso é acompanhamento de estudo, não certificação de domínio ou avaliação antifraude. A avaliação futura precisa ser validada no servidor.

### Base matemática do Brazilian Pineapple

As aulas m5l4/m5l5 contêm derivações próprias: C(h,2)×C(b,3) dá 3 candidatas no flop, 24 no turn e 100 no river após as extras. São combinações correlacionadas, não probabilidades independentes de vitória. O exemplo de extra de ás usa 2/47 sob informação e distribuição explicitadas. A mesa atual distribui board e extras sem reposição, inclusive aos jogadores em all-in. Futuras estimativas de equity por amostragem precisam repetir essa distribuição para todos os participantes ativos e computar frações de empates. Nenhuma tabela de equity ou range ótimo foi publicada como validada.

Referências metodológicas: [MIT 18.443, aula 17, p. 10](https://ocw.mit.edu/courses/18-443-statistics-for-applications-spring-2015/c9f6cf548a6e918d3dc798fc0d7c8170_MIT18_443S15_LEC17.pdf) para amostragem sem reposição; [NIST, intervalos para proporções](https://itl.nist.gov/div898/handbook/prc/section2/prc241.htm) para os limites de estimativas. As fontes não estudam nem endossam o Brazilian Pineapple da ZT.

### Produção e rastreabilidade

- Roteiros da home/história: `ZeroTiltCurso/editorial/episodes.json`. Os 25 roteiros técnicos são derivados de `courseContent.json` e `courseTraining.json`, incluindo teoria, exercícios resolvidos e orientação de prática. Renderizador: `render.py`; dependências: `requirements.txt`.
- Finais 1080p30 H.264/AAC: home v3, ep11–ep13 v2 e 25 arquivos `zt-academy-*-v3` em `Frontend-Web/public/videos/`; VTT e WebP acompanham cada MP4. Manifestos de duração/tamanho/capítulos e transcrições ficam junto ao renderizador.
- A home lê `src/data/homeFilm.json`, gerado pelo renderizador. Os vídeos históricos têm fontes e capítulos nas aulas. Marcos visuais de ep13 corrigidos para 1970/1971 e 2003/2008.
- Voz sintética PT-BR AntonioNeural; efeitos sonoros originais, sem música de terceiros. Fontes locais atuais: Georgia e Segoe UI do Windows. Para reprodução em outro sistema, configurar fontes disponíveis/licenciadas e manter a inspeção visual.
- A imagem histórica do Met é de domínio público, objeto 1983.515.1–.52. Arte do hero gerada por IA e preservada em `editorial/assets/hero-source.png`; derivados WebP em `public/brand/home/`. Nenhum slide acadêmico foi reproduzido.
- Todas as 28 aulas atuais têm `publicationStatus: published`. Foram retirados 78 arquivos obsoletos da pasta pública, incluindo o protótipo anterior da home; permanecem apenas os 87 arquivos dos 29 vídeos ativos. Os arquivos-fonte anteriores continuam arquivados em `ZeroTiltCurso/epNN-*/` e no git.
- Cada vídeo técnico guarda SHA-256 do conteúdo/cenários e do MP4. Alterar teoria, exercício ou cenário invalida o gate e exige regenerar as aulas afetadas. O renderizador valida a cobertura integral de palavras da voz, repete síntese incompleta e substitui o MP4 público somente após concluir a codificação e reconferir o conteúdo. `render.py --check` confere publicações, VTT/transcrição, capítulos, duração, imagens e hashes; `--decode` acrescenta decodificação integral por FFmpeg.
- Auditoria de dados, gabaritos, cartas, arquivos e pré-requisitos: `Frontend-Web/src/lib/courseAudit.test.ts`. Regras e exemplos de mãos: `Motor-Rust/tests/variant_audit_regressions.rs`.

## Coach virtual pós-mão — especificação pedagógica futura

O coach será uma ferramenta opcional de **estudo de mãos encerradas**, não assistência durante partidas. Ele não está implementado nesta release e não pode ser anunciado como disponível. A primeira versão deve usar uma única identidade pedagógica e analisadores internos por rua; quatro personagens separados de pré-flop, flop, turn e river aumentariam inconsistência, manutenção e custo sem melhorar a jornada do aluno.

### Jornada eficaz

1. O aluno escolhe uma mão própria já liquidada no histórico.
2. Seleciona um ponto de decisão e vê novamente posição, stacks, pote, ações e cartas que eram conhecidas naquele instante.
3. O sistema separa **fatos calculáveis** (preço, pot odds, tamanho, combinação e ações legais), **heurísticas declaradas** e informações indisponíveis.
4. A resposta apresenta uma alternativa principal e, quando pertinente, uma segunda linha plausível com os respectivos trade-offs — sem rotular toda divergência como erro.
5. Tags como `position`, `preflop_range`, `bet_sizing`, `pot_odds`, `equity`, `fold_equity` e `bankroll` apontam para lições existentes em `courseContent.json`.
6. O aluno resolve uma situação curta de fixação antes de marcar a revisão como concluída.

### Regras pedagógicas e de integridade

- Nunca emitir recomendação enquanto a mão estiver ativa nem usar WebSocket de mesa para feedback; isso seria RTA e conflita com os Termos.
- Nunca revelar cartas não mostradas, ranges privados ou dados de outro jogador. A análise reconstrói somente o que o aluno podia conhecer no ponto escolhido.
- Não prometer solução “GTO”, ação universalmente ótima ou lucro. Cada resposta declara variante, número de jogadores, posições, stacks, tamanho do pote, rake/deflator aplicável e hipótese de range.
- Hold’em é o primeiro escopo. Omaha 4 e Brazilian Pineapple só entram depois de fixtures específicos que garantam exatamente 2 hole + 3 board e o ranking correto de cada variante.
- Feedback deve ser curto e acionável: decisão → motivo → conta → alternativa → lição. Texto persuasivo não substitui evidência do motor.
- Personalização futura usa apenas histórico do próprio aluno, com consentimento e possibilidade de exclusão; não inventa leitura psicológica ou perfil profissional.

### Critério de qualidade do conteúdo

Um caso de referência precisa ter resultado reproduzível, explicação revisada, link de lição válido e teste que impeça regressão matemática. Antes de liberar o produto, amostras devem ser revisadas por responsável de conteúdo identificado; o sistema continua sendo coach virtual, sem biografia ou credenciais humanas inventadas.

<!-- DOCUMENTATION_SYNC:START -->
> **S26** (2026-10-01) — demo `zerotiltpoker.net` · sem certificação de produção · PIX automático ligado (DePix reconciliado).
> Fatos (catálogo, carteiras, limites): [`STATUS_OPERACIONAL.md`](STATUS_OPERACIONAL.md).
<!-- DOCUMENTATION_SYNC:END -->
