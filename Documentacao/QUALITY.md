# QUALITY — gates de qualidade e segurança

**Dono deste assunto.** Enciclopédia antiga: [`historico/QUALITY_v5.4.md`](historico/QUALITY_v5.4.md). Fatos do dia: [`STATUS_OPERACIONAL.md`](STATUS_OPERACIONAL.md). Contrato de agentes: [`../AGENTS.md`](../AGENTS.md).

Não é certificado de produção. PIX automático DePix está ligado na demo (crédito provisório ≤ R$ 50 no `processing`; saque travado até o `completed`).

## O que a CI realmente exige

Workflow `.github/workflows/rust-ci.yml` (não inflar com carga massiva):

| Job | Esperado |
|---|---|
| `documentation-sync` | `cargo run --locked --bin documentation-sync -- --check` |
| Motor | `cargo check`, `cargo test` determinístico, `clippy --all-targets -- -D warnings`, `fmt --check`, `audit` |
| API | check, testes (incl. contratos PostgreSQL no job), clippy, fmt |
| `frontend-web` | TypeScript + build Vite (e lint/test do `Frontend-Web` quando o job os roda) |
| Containers | scan + validação de build Docker |
| Gate de deploy | depende dos jobs acima — **não** equivale a certificação de produção |

Campanha local por situações reais de jogo (contagens somente do que foi executado): [`FULL_VALIDATION.md`](FULL_VALIDATION.md), **só** com autorização explícita. Scripts: [`../scripts/README.md`](../scripts/README.md).

Frontend local: `npm run lint` (`tsc` + ESLint) e `npm test` (Vitest) em `Frontend-Web/`, com o Node do `AGENTS.md` — nunca Node 18 do PATH.

### Exceção temporária de build — 05/10/2026 a 18/10/2026

`npm run audit:security` mantém o bloqueio de alertas high/critical, com uma exceção exata para [GHSA-vfj7-8cjw-p6xm](https://github.com/advisories/GHSA-vfj7-8cjw-p6xm), sem versão corrigida de `braces` em 05/10. A dependência 3.0.3 é exclusiva do build Tailwind 3 (chokidar/micromatch/fast-glob); seus padrões vêm dos dois globs fixos do repositório, não de jogadores. A imagem final serve somente arquivos estáticos, sem Node/Tailwind/braces. Não foi declarada ausência de vulnerabilidade no pacote.

Decisão de engenharia desta entrega: preservar a compatibilidade visual do frontend e limitar a aceitação ao vetor não exposto. O script dedicado `scripts/audit-frontend.mjs` é necessário para impor escopo e prazo, em vez de ignorar o código de saída inteiro do npm. Ele audita produção separadamente, exige `dev: true` no lockfile, versão e globs revisados, restringe a cadeia nominal de dependências e rejeita novos advisories, caminhos desconhecidos, falhas de rede e relatórios inválidos. A exceção expira automaticamente em **19/10/2026 00:00 UTC**. Cinco contratos verificam aceitação restrita, bloqueio em produção, expiração, novos alertas e ciclos. Atualizar/remover a dependência ou migrar Tailwind em entrega própria antes do prazo; não renovar automaticamente.

O scan da primeira imagem API encontrou `CVE-2026-103111` em libpcre2. A camada apt foi invalidada com `SECURITY_REFRESH=2026-10-05`; o Dockerfile agora exige libpcre2 ≥ `10.42-1+deb12u2`. Nenhuma exceção foi adicionada ao Trivy.

## Invariantes (código)

1. Valores em `u64` centavos. Potes → rake → Loss Deflator **somente** sobre o líquido.
2. Play Money e Jogo Real não se misturam.
3. Settlement de mão assinado (HMAC); uma mesa = um processo.
4. Sem segredos em git, logs ou docs. PIX live exige HMAC, allowlist e tetos; saque automático só pelo payout worker reconciliado.
5. `documentation-sync` recusa `production.certified: true`. `pix.automatic_in_production: true` exige payout worker + migration de payout.
6. Jogo Real falha fechado para KYC, autoexclusão e limites; `KYC_DATA_PEPPER` de produção tem no mínimo 32 bytes e não reutiliza JWT ou pepper de e-mail.

## Definition of Done

Uma mudança está pronta **localmente** quando:

1. Compila e clippy `-D warnings` no crate tocado (WSL).
2. Testes determinísticos relevantes passam (sem disparar full-validation).
3. Frontend tocado: `tsc` + lint + Vitest + build Vite com Node empacotado.
4. Fatos operacionais: JSON schema v3 + `documentation-sync --write` e `--check`. Prosa única só no arquivo dono (`AGENTS.md`).
5. Sem regressão óbvia no fluxo alterado.

Commit / push / deploy **não** fazem parte do DoD local — cada um exige ordem explícita (`AGENTS.md`).

## Auditoria local de consistência — 27–28/09/2026

### Escopo e evidência

Inventário automatizado: 685 caminhos versionados, mais 39 arquivos locais novos. A passagem final leu 723 arquivos existentes (600 textuais, 123 binários), com hash/tamanho por arquivo; 24 JSON válidos e nenhuma colisão de número de migration em disco. Um caminho removido corresponde à substituição já aprovada do plano de dois níveis. O inventário **não significa revisão manual linha a linha**. Evidências descartáveis em `artifacts/consistency-audit/`; código de regressão fica versionado.

| Achado | Correção local / evidência |
|---|---|
| Sequência A2345 guardava apenas o ás e podia vencer uma sequência maior; A6789 tinha ás no início do desempate | Cinco cartas em ordem correta de desempate; regressões de comparação, empate, straight flush e pagamentos por pote |
| Loss Deflator de Short Deck usava baralho/ranking tradicionais | Validação por variante e enumeração exata dos runouts legais de Short Deck, incluindo empates multiway; regressão contra todos os rivers legais de um turn |
| Teste de bot Short Deck chamava Brazilian Pineapple | Variantes separadas, caso negativo do wheel em Pineapple |
| Ranking clássico normativo invertia sequência/trinca | Tabela corrigida; contrato automatizado no frontend |
| Exercícios tinham ações ilegais, odds sem modelo, mãos mal classificadas e kicker inválido | Textos/gabaritos corrigidos; exemplos Omaha/Pineapple conferidos pelo avaliador Rust |
| Aula inexistente era liberada; reorganização podia bloquear aula já concluída | Pré-requisitos explícitos e preservação do progresso; testes de grafo, gates, IDs e cartas |
| Vídeos antigos divergiam das correções | Publicação suspensa na interface enquanto estão em revisão; formatos anunciados correspondem ao conteúdo disponível |
| Filme não apresentava as modalidades exclusivas; timeline histórica usava datas genéricas | Filme v3, manifesto único e datas específicas em ep13; decodificação e inspeção de quadros |
| Documentação de deflator contradizia modos de saldo, base e teto compartilhado | Reescrita dos exemplos com valores líquidos e cenário exato testado |

### Verificações desta etapa

- Motor: 1.859 testes de biblioteca aprovados, 31 ignorados; nove regressões específicas em `variant_audit_regressions` aprovadas. Não executar cargas massivas como parte desta contagem.
- API: 70 testes de biblioteca, 14 testes de router e 11 contratos PostgreSQL aprovados. Os 11 contratos normalmente ignorados foram executados separadamente, com conexão restrita ao banco local. `fmt` e `clippy --all-targets -- -D warnings` passaram nos dois crates.
- Dependências: `npm audit` sem vulnerabilidades conhecidas entre 409 dependências; `cargo audit` sem apontamentos nos locks da API (316 crates) e motor (111 crates), consulta de 28/09/2026. Isso não cobre falhas ainda não catalogadas.
- Frontend: TypeScript e ESLint; 75 testes em dez arquivos; build Vite. Após separar carregamento de Academy/mesa/torneio, chunk inicial reduzido de 553,16 para 410,71 kB, sem o aviso de 500 kB.
- Documentação: 22 documentos conferidos por `documentation-sync --check`.
- Chrome headless em perfil isolado: home, filme, índice Academy e aula de regras em 1440 e 390 px; sem exceção JavaScript, sem overflow horizontal nas superfícies verificadas. Filme carregou com `readyState=4`; abertura/fechamento por Escape e desafio de 25% funcionaram. Zero vídeos carregados antes de abrir o filme.
- MP4 novo e ep13 corrigido decodificados integralmente com FFmpeg; quadros de Pineapple, Short Deck e timeline inspecionados. Isso não substitui escuta editorial integral de todas as gravações.

### Limites e próximos controles

- Nesta etapa de 27–28/09, as alterações eram locais e a demo estava na migration 059. A publicação posterior está registrada em **Release S26** abaixo; a migration 060 não foi reescrita.
- Erros antigos de ranking/equity podem ter afetado mãos já liquidadas. Fazer auditoria somente de leitura de hand histories e settlements da versão publicada, quantificar impacto e elaborar reconciliação revisável antes de qualquer ajuste financeiro. Nenhum saldo foi recalculado nesta etapa.
- A regra usa fase/board do all-in e o conjunto final de oponentes elegíveis. Aderência exata a um snapshot de aceitação da aposta, sobretudo com folds posteriores e side pots, requer auditoria específica de eventos e fixtures; a correção de modalidade não prova esse contrato temporal.
- Hold’em pré-flop conserva estimativa Monte Carlo determinística; proximidade aos limites de tier exige avaliação da política de incerteza. Não confundir precisão média com garantia por mão.
- A nota do quiz de aula da Academy ainda é informada pelo cliente. Na nova mesa de estudo, perguntas objetivas são corrigidas no servidor. São contratos distintos; nenhum representa certificação de domínio nem avaliação GTO.
- Não houve campanha massiva, pentest externo, validação integral de PIX/saques reais ou certificação de produção. Inventário de arquivos, compilação e testes não demonstram ausência absoluta de falhas.

## Mesas e vídeos da Academy — verificação local de 28–29/09/2026

- Catálogo: 35 cenários, dez módulos, quatro modalidades e cobertura das 28 aulas. Testes verificam decks completos/únicos, versão, ações, reprodução determinística, proteção de cartas ocultas, respostas, check-raise do bot, conservação de fichas e distribuição de extras no all-in.
- Motor: 1.859 testes de biblioteca e nove regressões de variantes aprovados; 31 testes de biblioteca ignorados. Clippy de todos os alvos passou.
- API: 78 testes de biblioteca (oito específicos da Academy) e 17 de router aprovados; 11 contratos que requerem banco permaneceram ignorados nesta rodada. A rota de treino passou com pool de banco indisponível. Corpo limitado, campos extras/valores inválidos rejeitados e rate limit isolado da cota de autenticação. Regressões conferem atualização da dica entre streets e limites mínimos/máximos de bet/raise nos 35 cenários. `fmt` e `clippy --all-targets -- -D warnings` aprovados.
- Frontend: 83 testes em 11 arquivos, TypeScript, ESLint e build Vite aprovados, incluindo os dois gates finais de mídia. Chunk inicial de 411,42 kB (121,16 kB gzip), com curso/mesas carregados separadamente. Comparação de SHA-256 do conteúdo e MP4, manifesto, duração, capítulos e transcrição de cada vídeo técnico; rejeição de arquivos obsoletos na pasta pública. A comparação de transcrições normaliza apenas CRLF/LF; o teste de hashes tem limite de 30 segundos para ler os 25 MP4 em disco.
- Navegador: 35 cenários em 1440 px e cinco representativos em 390 px, sem erro JavaScript ou overflow. Conferidos all-in Pineapple 2→5, resposta objetiva, salvar/reabrir, replay sem cartas futuras, retorno de decisão, check-raise do bot e encerrar/retomar. Evidências locais em `artifacts/academy-audit/`.
- Audiovisual: 25 vídeos técnicos v3, três históricos e filme da home, total de 29 MP4/159,4 minutos. `render.py --check` aprovou hashes, duração, capítulos, posters e correspondência integral VTT/transcrição. Todos os fluxos de áudio/vídeo passaram pela decodificação integral por FFmpeg; `media-progress.json` registra SHA e reaproveita somente a evidência de arquivos inalterados. Foram retirados 78 arquivos obsoletos da pasta pública; restam 87 arquivos ativos (MP4/VTT/WebP).
- Narração/layout: 399 trechos de voz com cobertura completa das palavras, zero ausente/incompleto; 376 cenas técnicas sem corte de texto no layout medido. Um trecho de voz truncado sem erro de transporte foi detectado e refeito. O renderizador agora valida toda a sequência de palavras, repete a síntese incompleta e substitui o MP4 público somente após a codificação terminar e o conteúdo ser reconferido. Falha posterior de DNS interrompeu o lote sem danificar os vídeos prontos; a retomada completou as aulas restantes.
- Player em Chrome: os 29 arquivos carregaram em 1920×1080 e com a duração prevista. Capítulos, busca temporal até o último capítulo, legendas e transcrição conferidos em 1440/390 px, sem exceções ou overflow; dez links de mesa presentes no índice. Evidências em `media-browser.json` e capturas dentro de `artifacts/academy-audit/`.
- Limites editoriais: a voz é sintética. Cobertura de palavras, decodificação, conferência de dados e inspeção de quadros não substituem escuta humana integral da prosódia/pronúncia. Bots didáticos usam heurísticas com informação limitada; não são um solver GTO. Esta entrega cobre as 28 aulas atuais, não toda a expansão futura de 15 etapas.

## Complemento histórico de Omaha — verificação local de 30/09/2026

- Aula `m0l2`, conteúdo versão 7: nove fontes vinculadas (seis novas), quatro questões (duas novas); totais da grade de 28 aulas, 65 questões e 21 referências. A dissertação de Ho foi conferida quanto ao modelo de dois jogadores e decisões jam/fold; não é evidência sobre a etimologia do nome.
- `ep12-mississippi-omaha-v3.mp4`: 253,93 s, 1080p30, 13 capítulos e 63 trechos de legenda. Hashes do roteiro/conteúdo/MP4, transcrição e legenda coerentes; decodificação integral por FFmpeg aprovada. As 13 cenas passaram por inspeção visual e medição de limites de texto. Os outros 28 vídeos conservaram seus hashes e evidências de decodificação. Total audiovisual atual de 161,9 minutos.
- Chrome em 1440 e 390 px: reprodução e salto ao capítulo final, legendas, transcrição, nove links de fontes e quiz com respostas corretas conferidos. Sem exceções JavaScript ou overflow horizontal. Evidências: `artifacts/academy-audit/omaha-history-browser.json`, imagens `omaha-history-player-*`, `omaha-history-contact-sheet.png` e `omaha-history-layout.json`.
- 83 testes frontend em 11 arquivos, TypeScript, ESLint, build Vite e `git diff --check` aprovados. As três mídias anteriores de `ep12` foram preservadas fora da pasta pública após verificar suas referências; o gate confirmou 87 arquivos ativos. Nenhum código Rust ou contrato financeiro foi alterado nesta retomada.
- Conclusão local; sem publicação na demo. Mantida a pendência de escuta editorial humana integral. O curso específico de Omaha continua fora desta entrega, conforme orientação do proprietário.
- Revisão final: o log de desenvolvimento revelou chaves React repetidas para vídeo/quiz. Corrigidas com prefixos distintos; repetidos TypeScript, ESLint, build, 83 testes e navegação desktop/celular. A captura ampliada a `console.error`/`console.warn` ficou vazia, além de não haver exceções.

## Release S26 — 01/10/2026

- Motor: `fmt`, `clippy --all-targets -D warnings`, 1.859 testes de biblioteca e nove regressões de variantes aprovados; 31 testes ignorados por seus gates existentes. Nenhuma carga massiva executada.
- API: `fmt` e `clippy --all-targets -D warnings` aprovados; 77 testes de biblioteca, 17 de router e 17 contratos de banco aprovados (12 em `api_tests`, quatro em `payments_tests` e um de estorno em `estrutura`). Contratos PostgreSQL executados explicitamente no banco local com migration 060 e em banco isolado criado do zero; o novo teste cobre autorização administrativa, conciliação, rejeição de totais desatualizados, fechamento concorrente/idempotente, bônus de 35%, comissão base de 30%, déficit e separação entre pontos PM, comissão Real e saldo de jogo. O contrato de estorno é marcado como dependente de banco, sem retornar sucesso silencioso quando falta `DATABASE_URL`.
- Frontend: TypeScript, ESLint, 83 testes em 11 arquivos e build Vite aprovados (172 módulos; bundle inicial 411,43 kB antes de gzip). As dicas reutilizam a teoria canônica da Academy.
- Documentação: contrato operacional schema v3 substitui o split bruto 18/12/70 por programa direto sobre NGR mensal; 14 testes da ferramenta, `fmt`/`clippy`, geração e `--check` aprovados em 22 documentos. A migração do schema rejeita o campo antigo e bases de comissão incompatíveis.
- Inventário: JSONs válidos, sem colisão de migrations, sem links locais quebrados nos documentos alterados; nenhum candidato de publicação acima de 50 MiB ou correspondência nos padrões de credenciais examinados. Isso não substitui auditoria externa de segurança.
- Deploy principal confirmado em 01/10 às 15:27:57 UTC (12:27:57 em São Paulo), com backup PostgreSQL/Caddy validado e imagens anteriores preservadas. API, frontend, PostgreSQL e Redis saudáveis; processos de API/frontend com uid 10001. Todas as 60 migrations conferidas por SHA-384 e as 87 mídias por SHA-256 entre Git e container. Nenhum agente ativado, backfill ou fechamento financeiro executado pelo deploy.
- O primeiro CI identificou advisories novos em `brace-expansion`; atualizadas somente as duas dependências transitivas para 1.1.21 e 5.0.12. `npm ci` e `npm audit` passaram com zero vulnerabilidades, assim como TypeScript, ESLint, 83 testes e build após a correção.
- Reproduzida a falha do contrato de depósito: a conta de teste não possuía KYC. O teste agora exige 403 sem criar cobrança, verifica a conta sintética e então confere crédito e idempotência. Os quatro contratos de carteira passaram; provedor mock e chave sintética são inicializados antes das requisições paralelas. O CI também executa explicitamente o contrato de estorno do agente e interrompe migrations quando o PostgreSQL relata erro.
- O gate estrito expôs o autocommit no executor antigo do CI: a migration 026 perdia sua tabela `ON COMMIT DROP` antes do uso. Reprodução em banco isolado confirmou o erro. As 60 migrations passaram em banco vazio com uma transação por arquivo (`psql --single-transaction`), alinhado ao sqlx. Preservados os arquivos/checksums das migrations já aplicadas.
- Smoke público: `/health`, `/caddy-health`, lobby, Academy e Agente ZT acessíveis por HTTPS; vídeo Omaha com SHA-256 idêntico ao manifesto, Range HTTP 206, VTT e WebP corretos. Laboratório Pineapple confirmou distribuição 2→5 no all-in, sem carteira ou liquidação.
- Chrome público em 1440/390 px: aula Omaha com 13 capítulos, 63 legendas, transcrição, nove fontes e quiz; home, dicas e Agente ZT sem exceções/avisos de console nem overflow horizontal. Cinco cenários de treino em cada largura e fluxos de all-in, replay, caderno, check-raise e retomada aprovados. A primeira tentativa de percorrer toda a grade atingiu a cota de requisições; a amostra posterior passou respeitando a janela. Não foi alterado o rate limit.
- A conferência pública identificou dicas locais ocultas até terminar o carregamento de RSS. A correção `dc391a29` apresenta o conteúdo disponível imediatamente, com 83 testes, lint e build aprovados; redeploy somente do frontend confirmado às 22:15:50 UTC (19:15:50 em São Paulo). Evidências descartáveis da conferência pública em `artifacts/release-s26/`.
- CI da versão publicada `dc391a29`: [Rust CI](https://github.com/leofran2204/poker-platform/actions/runs/36933764739) e [Container Supply Chain](https://github.com/leofran2204/poker-platform/actions/runs/36933764974) concluídos com sucesso, incluindo contratos PostgreSQL, auditorias, build Docker, cobertura, SBOM, assinatura e atestações. A documentação final passou por `documentation-sync --check` (22 arquivos), verificação dos links locais e `git diff --check`.

## Primeira campanha de jogo — 02/10/2026 (antes das correções)

**Reprovada; carga interrompida por falha reproduzível.** Método, lacunas e reprodução em [FULL_VALIDATION.md](FULL_VALIDATION.md); evidência em [summary.json](../artifacts/full-validation/20261002T061251Z-e31cc0/summary.json).

- Um caso dirigido aprovado, um falho e uma reprodução confirmando a falha. Nenhuma das 36 etapas posteriores da versão executada foi iniciada. Inventariar 2.045 testes e compilá-los não os contabiliza como aprovados.
- Falha: após raise de 25 para 75, call e all-in curto para 100, o motor aceita o jogador que já agiu aumentar para 150. O último incremento completo era 50; o all-in acrescentou 25. A fixture parte de três entradas fixas de 2.500 centavos e obtém o short stack por uma mão legal anterior. Mantida a expectativa de rejeição, sem alteração da regra/código de produção.
- Ambiguidade no banco migrado: quatro MTTs PM têm reentradas ilimitadas (rebuy_max_count=0), enquanto o catálogo de eventos informa uma; os equivalentes Real têm uma. Registrar a decisão de negócio antes de qualquer correção. Nenhuma migration foi editada/criada.
- Lotes cash, campeões MTT, precisão Monte Carlo, atores concorrentes e HTTPS/WSS continuam sem evidência desta campanha. As novas fixtures posteriores, inclusive a auditoria temporal do Loss Deflator, foram preparadas e compiladas, mas não executadas após a interrupção.
- Executores agora usam stack exclusiva, contas sintéticas, provedores mock/log, orçamento global e até dois trabalhadores. Contagens vêm do libtest; etapas pendentes/ignoradas e timeout não viram sucesso. Removida a afirmação de milhões de entradas frontend que não eram executadas.
- Seis contratos do executor passaram; clippy --all-targets -D warnings passou para motor e API, incluindo a feature full-validation na API. Sintaxe Python/PowerShell/Bash/Node e verificações documentais constam dos artefatos finais. Esses checks não substituem os cenários interrompidos.
- Preparação/execução contabilizadas somaram 529,81 segundos, excluindo compilação. Dumps e traces foram preservados; containers temporários removidos. Sem carga na demo, modificação de carteiras reais, commit, push ou deploy.

## Retomada local — 03/10/2026

**Campanha incompleta por deadline; correções locais com regressões aprovadas.** Evidências e lacunas em [FULL_VALIDATION.md](FULL_VALIDATION.md). A última execução completou 36.000 mãos cash e quatro MTTs até campeão (234 mãos); o parcial Pineapple não é contabilizável. Permanecem pendentes cash/MTT Pineapple, precisão de equity, matriz completa e os estágios integrados de API/atores/HTTPS/WSS. Não foi iniciada nova campanha massiva após a expiração.

- Corrigidos reabertura após all-in curto, turno do small blind all-in, ações legais no WS e snapshot dos oponentes no pagamento do all-in, preservado mesmo após fold posterior. As fixtures mantêm entradas e evolução de stacks por jogo legal.
- Premiação contempla posições eliminadas e redistribui proporcionalmente quando há menos inscritos que posições pagas, conforme decisão do proprietário. Centavos são conservados por rateio inteiro; créditos, posições, status e auditoria são atômicos e idempotentes.
- O catálogo passou a representar separadamente as reentradas PM/Real já vigentes nas migrations, sem modificar SQL histórico. Comparação com banco vazio sem divergências.
- Rotina: 1.859 testes de biblioteca do motor (31 ignorados), 12 regressões dirigidas, 79 testes de biblioteca da API (dois ignorados), 14 testes do sincronizador e oito contratos do executor aprovados. `fmt` e `clippy --all-targets -D warnings` passaram; API também conferida com `full-validation`.
- Executado separadamente um dos contratos ignorados da API, em PostgreSQL descartável: rollback integral da premiação, finalizações concorrentes, repetição sem crédito duplicado e isolamento PM/Real aprovados. Não equivale à campanha integrada da API.
- O executor agora distingue deadline de falha, preserva os originais durante reproduções e registra tempos monotônico e de relógio. As últimas mudanças no gerador cash, referências de equity e transporte foram compiladas/verificadas estaticamente, mas aguardam nova campanha autorizada.
- Trabalho local, sem commit, push, deploy ou tráfego na demo. Relatórios originais e dumps preservados em `artifacts/full-validation/`.

## Nova campanha autorizada — 03/10/2026

**Resultado incompleto.** Na evidência principal, 25.000 mãos das cinco configurações cash e cinco MTTs (309 mãos), com pools integralmente premiados. Cobertos os 440 pares globais; quatro configurações sem lacunas individuais e Pineapple com um par pendente. Sete referências de equity aprovadas, erro máximo de 0,08399 ponto percentual; quatro vizinhanças dos tiers testemunhadas. As tentativas anteriores não são somadas como cobertura nova.

- Passaram 21 contratos de banco/Redis, as rotinas da API, atores internos com 1/5/20 mesas e desconexões. Corrigida a fixture antiga que confundia desconexão com remoção imediata; agora exige preservação e reconexão do assento sem duplicação.
- HTTPS/WSS de uma mesa passou, incluindo timeout, repetição imediata de call, privacidade e settlement persistido. Cinco mesas encontraram HTTP 429; vinte mesas e reinício ficaram pendentes. Duas sondas de socket comprovaram que o encaminhamento do Node Windows ao WSL agrupava IPs; runtime Linux mantém as origens distintas. O executor exige Linux, sem alterar limites da API.
- A tentativa após esse ajuste expirou por deadline antes da primeira mão cash: 93,07 s monotônicos contra 29.824,56 s de relógio. Causa da discrepância não determinada. Sem nova carga depois da expiração; containers exclusivos removidos. O relatório preserva o zero efetivamente executado nessa tentativa.
- A última fixture Pineapple e o transporte Linux em carga integrada aguardam execução. Formatação, clippy dos crates tocados, oito contratos do executor e diagnóstico de loopback passaram. Método, caminhos e limitações em [FULL_VALIDATION.md](FULL_VALIDATION.md) e no [resumo consolidado](../artifacts/full-validation/authorized-20261003-summary.json).

## Onde não procurar qualidade

- Aprendizado / protocolo Mark → `guia_aprendizado.md`
- OKR, marketing, chaos, “plano Elon” → `historico/QUALITY_v5.4.md` (arquivo, não vigente)
- Catálogo e ciclo → STATUS
- Fases e backlog → DASHBOARD


## Pineapple híbrido — entrega local de 04/10/2026

Regra `brazilian_pineapple_hybrid_v1` aplicada no motor, atores cash/MTT, WebSocket, bots, interface e treinos; sem migration. Pré-flop 1–4 BB, pós-flop com aumento limitado ao pote antes do call e reabertura completa/cumulativa. Novos históricos/settlements versionados; originais anteriores preservados.

Verificação: 1.859 testes de biblioteca do motor (31 ignorados), 19 casos dirigidos (sete novos grupos Pineapple), 80 testes de biblioteca da API (dois contratos de banco ignorados nessa rotina), 86 testes frontend e dez contratos do executor. Rust fmt/clippy de todos os alvos, TypeScript, ESLint, Vite, sincronização dos 22 documentos e integridade das 29 mídias aprovados. Três vídeos Pineapple permanecem claramente identificados como material da regra anterior; leitura e treinos já usam a regra nova. A conferência de mídia não constitui nova narração/renderização nem escuta editorial integral.

A campanha focada foi concluída dentro dos 15 minutos autorizados com dois trabalhadores: 6.000 mãos cash, 385 pares sem lacunas, MTT em 75 mãos com premiação integral, HTTPS/WSS 5/20 mesas e reinício. Contagens repetidas não são somadas; diagnóstico, fontes e limites em [FULL_VALIDATION.md](FULL_VALIDATION.md#resultado-consolidado-da-estrutura-híbrida). Sem commit, push ou deploy.

## Filme Brazilian Pineapple — verificação local de 04/10/2026

- Filme novo: 477,67 s (limite 480 s), H.264/AAC, 1280×720 a 30 fps, 31 cenas, oito capítulos HTML/nativos no MP4 e 113 trechos VTT. Mesma voz `pt-BR-AntonioNeural`, taxa original `-3%`, sem acelerar áudio. SHA-256 do MP4: `a533b0b947bb25d1c7fa45170fadc2e577d72a8d69559aedda1e49e6640208e5`.
- Roteiro e exemplos conferidos contra `BUSINESS_RULES.md`, os sete grupos Pineapple de `full_validation_situations` e três regressões em `pineapple_film_examples`: cartas do próprio roteiro, sequência baixa/empate, flush inválido 1+4, reabertura pós-flop individual e camadas dos potes. Sem mudanças na lógica de apostas, APIs ou migrations.
- Inspeção visual das 31 cenas, entradas das cartas, seleção 2+3 e contadores; corrigida a passagem do pote de R$ 35 para R$ 50 no call de Ana. Medidos 2.323 textos em entradas, destaques e finais de cena, sem ultrapassar os limites. A cobertura de palavras dos 31 segmentos é completa; VTT/transcrição coincidem. O áudio final mede -16,44 LUFS e pico verdadeiro de -0,81 dBTP, sem clipping. Essa checagem usa metadados de fala e medição do sinal; não registra escuta humana integral.
- `render.py --check` aprovou as 30 produções/90 arquivos, incluindo os originais das três aulas `prior_rules`. `--check --decode --only pineapple` decodificou o novo MP4 integralmente; FFprobe conferiu duração, codecs e oito capítulos. A repetição da decodificação dos 29 vídeos inalterados foi interrompida após oito, pois seus hashes e a evidência anterior são preservados. Relatórios em `artifacts/academy-audit/media.json` e `media-pineapple.json`.
- Frontend: 87 testes em 12 arquivos, TypeScript, ESLint e Vite aprovados. Chrome headless isolado em 1440, 390 e 320 px validou home/Academy sem overflow ou exceção JavaScript, todos os capítulos, primeiro salto sem metadados, legendas, transcrição, Enter/Escape, botão de fechar, retorno do foco e reabertura pausada. Nenhum MP4 é solicitado antes da reprodução ou seleção de capítulo. Redirecionamento logado `/` → `/curso` confirmado com sessão sintética e respostas de API locais, sem testar autenticação real ou servidor de produção.
- Evidências da produção e interface: `artifacts/pineapple-film/` (`browser.json`, `editorial-audit.json`, capturas e painéis das 31 cenas). Fontes e finais ficam no repositório; intermediários/cache permanecem ignorados. Entrega estritamente local. Publicação futura exige a regra correspondente, nenhuma mão Pineapple aberta e nenhum torneio Pineapple em andamento.

## Publicação Pineapple — 05/10/2026

- Código publicado: `6b4d8cc0a24224bcd45b9397b591eddaee53615f` (implementação em `79b1a086` e ajuste dos gates em `6b4d8cc0`), concluído às **15:47:34 UTC / 12:47:34 em São Paulo**. [Rust CI](https://github.com/leofran2204/poker-platform/actions/runs/37265453043) e [Container Supply Chain](https://github.com/leofran2204/poker-platform/actions/runs/37265453081) concluídos com sucesso antes da troca, incluindo cobertura, contratos PostgreSQL, builds, scans, SBOM, assinatura e atestações. O npm mantém a exceção de build exata e temporária descrita acima; não significa zero alertas brutos.
- Janela conferida três vezes: antes do bloqueio de novas entradas, após drenar requisições e com a API anterior parada. Zero assentos cash ativos, zero snapshots, zero torneios em andamento e zero torneios agendados com inscritos. Bloqueio temporário de joins/inscrições/admin mutável/WS no Caddy; configuração normal restaurada pelos containers novos. Nenhuma mão ou torneio foi cancelado para publicar.
- Backup anterior em `/opt/poker-platform/backups/pre-pineapple-20261005T045410Z`: dump validado por `pg_restore --list`, Caddy validado por gzip e imagens anteriores preservadas. Novo dump e checksum imediatamente antes do boot. `.env` idêntico por hash; sem migrations novas, reset de banco ou alterações financeiras de teste.
- VPS: quatro serviços `healthy`, API/frontend UID 10001, imagens correspondentes ao build aprovado, libpcre2 `10.42-1+deb12u2`, 60 migrations/checksums SHA-384 corretos e log `Migrations applied`. Os 90 arquivos servidos pelo container coincidem em SHA-256 com o repositório.
- Público: `/`, `/curso`, aula histórica Pineapple, mesa de estudo, lobby e health com HTTP 200. MP4/VTT/WebP novos com hashes idênticos, HTTP Range 206. Academy versão 2 executa o cenário de all-in curto até cinco privadas, informa `brazilian_pineapple_hybrid_v1`, rejeita all-in acima do teto e treino versão 1; sem saldo/carteira na resposta.
- Chrome isolado contra o domínio em 1440/390/320 px: home e Academy, oito capítulos, cold seek, 113 legendas, transcrição, teclado, fechamento/foco e reabertura pausada aprovados; duração 477,667 s e sem overflow/erros JavaScript. A checagem visual intercepta APIs e usa uma sessão sintética para o redirecionamento; o smoke HTTP separado chama a API pública real. Não equivale a testar login de uma conta real.
- Evidência descartável em `artifacts/pineapple-film/release/`: CI, scans, conferência de servidor/HTTP, `browser.json` e capturas. Documentos de produto e dashboard atualizados depois do deploy; imagens permanecem vinculadas ao commit de código acima. Escuta editorial humana integral e substituição da dependência de build antes de 19/10 continuam no backlog; demo segue sem certificação de produção.

<!-- DOCUMENTATION_SYNC:START -->
> **S26** (2026-10-05) — demo `zerotiltpoker.net` · sem certificação de produção · PIX automático ligado (DePix reconciliado).
> Fatos (catálogo, carteiras, limites): [`STATUS_OPERACIONAL.md`](STATUS_OPERACIONAL.md).
<!-- DOCUMENTATION_SYNC:END -->
