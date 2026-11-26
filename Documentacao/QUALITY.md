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

Frontend local: `npm run lint` (`tsc` + ESLint) e `npm test` (Vitest) em `Frontend-Web/`, com o Node do `AGENTS.md` (≥ 22.22; o CI e o Docker usam o 26).

### Exceção temporária de build — 05/10/2026 a 18/10/2026

**Encerrada antecipadamente na revisão de 05–06/10/2026.** Tailwind 4.3.3 e `@tailwindcss/postcss` retiraram `braces`, `micromatch`, `chokidar` e `fast-glob` do lockfile frontend. `npm run audit:security` voltou à auditoria npm integral, sem exceções, com zero vulnerabilidades na conferência. O prazo original abaixo foi preservado como histórico, não como pendência vigente.

Na primeira publicação, a exceção se restringia a [GHSA-vfj7-8cjw-p6xm](https://github.com/advisories/GHSA-vfj7-8cjw-p6xm) na dependência de build `braces` 3.0.3, sem versão corrigida em 05/10. Os padrões eram controlados pelo repositório e a imagem pública não executava Node/Tailwind. O script temporário exigia dependências `dev`, cadeia e globs revisados, rejeitava novos alertas/produção e expiraria em **19/10/2026 00:00 UTC**. Script e cinco testes específicos foram removidos com a dependência. Nenhuma exceção foi renovada nem convertida em aceitação permanente.

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
3. Frontend tocado: `tsc` + lint + Vitest + build Vite com o Node do `AGENTS.md`.
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

## Remoção da exceção npm — 06/10/2026

- Tailwind 4.3.3 com o plugin PostCSS oficial, tema em CSS e classes renomeadas conforme a migração. Preservados cores, alturas de linha, bordas, raios e a camada dos componentes. `braces` e sua cadeia anterior foram removidos; o CI executa novamente `npm audit --audit-level=high` diretamente, sem filtros ou exceções.
- A repetição final da auditoria identificou [GHSA-68fv-2mgg-jv7q](https://github.com/advisories/GHSA-68fv-2mgg-jv7q), incorporado à base revisada em 05/10. Atualizada somente a dependência transitiva `source-map-js` de 1.2.1 para 1.2.2; auditoria integral voltou a zero.
- O scan remoto do commit `31003c55` detectou a camada Debian em cache com `perl-base` anterior a `5.36.0-7+deb12u4`. Atualizado `SECURITY_REFRESH` e exigida essa versão mínima no Dockerfile, conforme [Debian Security](https://security-tracker.debian.org/tracker/CVE-2026-42497). Isso exige reconstruir também a imagem da API, sem alterar o binário de regras ou as migrations. Mantido o bloqueio do scan, sem exceções adicionais.
- Verificação local: 87 testes em 12 arquivos, TypeScript/ESLint, build Vite, auditoria npm com zero vulnerabilidades, sincronização dos 22 documentos e `git diff --check`. Nenhuma alteração no motor, API, migrations ou arquivos de mídia.
- Comparação em Chrome isolado de home, Academy, aula, login, lobby e mesa de treino em 1440/390/320 px: posições, dimensões, tipografia e altura das páginas preservadas, sem overflow ou exceções JavaScript. Inputs usam `outline-hidden`, equivalente de acessibilidade previsto na migração oficial. Evidências e capturas em `artifacts/tailwind-migration/`.
- Player da home e Academy aprovado nas três larguras: oito capítulos, primeiro salto antes dos metadados, 113 legendas, transcrição, Enter/Escape, retorno do foco, reabertura pausada e carregamento sob demanda. Sessão sintética confirma o redirecionamento para `/curso`; APIs interceptadas para a inspeção visual. O filme permanece em 477,667 s.
- CI da documentação anterior `81fe247b` concluído com sucesso: [Rust CI](https://github.com/leofran2204/poker-platform/actions/runs/37336245372) e [Container Supply Chain](https://github.com/leofran2204/poker-platform/actions/runs/37336245196). A escuta editorial humana integral continua sem atestação; a migração de estilos não altera a narração.
- Publicado `4c9541a3d41d3447eb514adb9ce476db91ceac4f` em **06/10 às 06:20:18 UTC / 03:20:18 em São Paulo**, após [Rust CI](https://github.com/leofran2204/poker-platform/actions/runs/37422327983) e [Container Supply Chain](https://github.com/leofran2204/poker-platform/actions/runs/37422328005) concluírem com sucesso. Scans, SBOM, assinaturas e atestações aprovados. Backup validado em `/opt/poker-platform/backups/frontend-tailwind-4c9541a3d41d`; imagens anteriores preservadas. Janela verificada sem assentos, snapshots ou torneios ativos, com novas entradas bloqueadas durante a troca. Banco/Redis conservaram os mesmos containers; `.env` idêntico por hash.
- API/frontend saudáveis e UID 10001; `perl-base` `5.36.0-7+deb12u4`. Binário da API preservado: SHA-256 `32b8a4013ab562c068ae1f88ddf70ea104b0a1e3bda581a811bd6e9451ddfa66`. Todas as 60 migrations/checksums e os 90 arquivos de mídia conferidos. Boot confirmou `Migrations applied`; nenhuma nova migration ou alteração financeira de teste.
- Conferência pública repetiu as 18 vistas e o player nas três larguras, sem overflow ou exceções. Foco de inputs conferido também em alto contraste no build local. Smoke HTTP real aprovou health, rotas, hashes MP4/VTT/WebP, Range 206 e treino Pineapple versão 2, incluindo rejeição do all-in acima do teto e da versão antiga. Evidências em `artifacts/tailwind-migration/release/`, `live/` e `live-player/`. A documentação final não muda as imagens publicadas.

## Pronúncia dos vídeos — 08/10/2026

- Regravadas as 30 produções da plataforma (28 aulas, filme Pineapple e institucional), mantendo a voz `pt-BR-AntonioNeural` e a taxa `-3%`. O perfil `pt-BR-poker-v1` aplica guias fonéticas inglesas à síntese, incluindo “Pineapple” → “painépou”; textos, exemplos e transcrições permaneceram iguais aos do git. Os tempos reais da voz são projetados sobre a grafia original das legendas e das marcações visuais.
- As três aulas `prior_rules` foram incluídas a pedido do proprietário, usando `sourceSnapshot`. Conteúdo, hashes das fontes, versão `legacy_no_limit` e avisos preservados; `originalMediaSha256` registra os MP4 anteriores. A revisão da voz não altera a regra ensinada nessas gravações.
- Integridade: 30 MP4/169,4 minutos e 90 arquivos de vídeo/legenda/capa conferidos. Os 437 trechos têm cobertura completa das palavras e cabem nas respectivas cenas; todos os MP4 passaram pela decodificação integral, conferência de codecs, dimensões, duração e níveis de áudio. Evidências por SHA-256 em `artifacts/narration-pronunciation/speech-audit.json`; gate geral em `artifacts/academy-audit/media.json`.
- Pineapple: 475,37 s no manifesto (475,40 s no contêiner), oito capítulos, 113 legendas, 31 cenas e 2.323 instâncias de texto medidas sem corte. Áudio medido em −16,42 LUFS, pico verdadeiro −0,93 dBTP; MP4 SHA-256 `b6bdd31ee64dc56ef44739ce0f48881c52a4b47a78d9956ce3f17ac5034d4f57`. As 376 cenas técnicas também passaram pela medição de limites de texto.
- Frontend: 87 testes, TypeScript/ESLint, build Vite e auditoria npm com zero vulnerabilidades. Seis testes Python cobrem palavras inteiras, apóstrofos, termos compostos, projeção temporal, fala incompleta e invalidação de cache. O CI passa a executar esses testes e o gate de publicações.
- Chrome isolado: as 28 aulas em 1440/390 px (56 vistas), além de home/Academy em 1440/390/320 px (seis vistas). Duração, legendas, capítulos, primeiro salto antes dos metadados, transcrição, ausência de autoplay, carregamento sob demanda e URLs com `narrationHash` aprovados. Avisos das três aulas antigas preservados; sem overflow ou exceções JavaScript. Foco, Enter/Escape, fechamento e reabertura do modal também aprovados. Progresso/login sintéticos e APIs interceptadas: não é uma validação de autenticação real.
- Evidências locais e capturas em `artifacts/narration-pronunciation/`. As verificações técnicas e as guias fonéticas não constituem escuta humana integral da prosódia/pronúncia; essa atestação permanece pendente. APIs de apostas, motor e migrations não foram alterados.
- Publicado o frontend `75971a0cf204bc006a440583a652cbc2409a3618` em **08/10 às 13:48:04 UTC / 10:48:04 em São Paulo**, após [Rust CI](https://github.com/leofran2204/poker-platform/actions/runs/37727987285) e [Container Supply Chain](https://github.com/leofran2204/poker-platform/actions/runs/37727987182) aprovarem, incluindo o novo gate editorial, scans, SBOM, assinaturas e atestações. Uma queda de SSH interrompeu o primeiro build antes da publicação; a retomada preservou e reconferiu os backups.
- Backup validado em `/opt/poker-platform/backups/frontend-pronunciation-75971a0cf204`, com imagem anterior disponível para rollback. Janela conferida sem assentos cash, snapshots ou torneios ativos. Apenas o frontend foi recriado: API, PostgreSQL e Redis mantiveram os mesmos containers, imagens e horários de início; `.env` idêntico por hash. API segue na imagem da publicação de 06/10. Quatro serviços saudáveis, aplicações em UID 10001, 60 migrations/checksums SHA-384 corretos e os 90 arquivos servidos idênticos ao Git por SHA-256.
- HTTP público: home, Academy, aula histórica, mesa de estudo, lobby e health aprovados. Nas 30 produções, Range 206 e primeiros bytes dos MP4 corretos, legendas/capas idênticas ao Git; MP4 principal também conferido integralmente. Comparação das legendas usa os bytes LF versionados, pois o checkout Windows das aulas usa CRLF. Treino Pineapple versão 2 executa o all-in curto e rejeita aposta acima do teto e versão antiga, sem movimentar carteiras.
- Home e Academy públicas aprovadas em 1440/390/320 px: oito capítulos, primeiro salto antes dos metadados, 113 legendas, transcrição, Enter/Escape, retorno do foco, ausência de autoplay e URLs da nova narração; sem overflow ou exceções JavaScript. Redirecionamento logado conferido com sessão sintética e APIs interceptadas; o smoke HTTP separado usou a API pública real. Evidências em `artifacts/narration-pronunciation/release/`.

<!-- DOCUMENTATION_SYNC:START -->
> **S28** (2026-11-23) — demo `zerotiltpoker.net` · sem certificação de produção · PIX automático ligado (DePix reconciliado).
> Fatos (catálogo, carteiras, limites): [`STATUS_OPERACIONAL.md`](STATUS_OPERACIONAL.md).
<!-- DOCUMENTATION_SYNC:END -->


## Pineapple v2 — regras e distribuição sequencial (09/10/2026)

Todas as rodadas usam o pote antes do call; ante obrigatório de 1 BB exclusivamente do big blind, com prioridade ao blind e pagamento parcial. Motor, atores cash/MTT, campos públicos `ante`/`ante_paid`/`ante_player_id`, controles e Academy v3 seguem `brazilian_pineapple_pot_before_call_v2`. Sem migration, reescrita de históricos ou reliquidação. O executor de validação acompanha a nova regra; resultados massivos antigos permanecem evidência histórica, sem nova campanha implícita.

- Motor: 1.859 testes de biblioteca, 31 ignorados, 19 situações dirigidas e sete testes dos exemplos audiovisuais. Verificados R$ 2,50 iniciais, R$ 4 com calls, abertura máxima R$ 3,50 e R$ 11,50 após os calls; pós-flop R$ 10 → R$ 15 → R$ 35 → R$ 70, botão, turno e saldos conferidos lendo o roteiro. Inclui mínimos, reabertura cumulativa, all-ins acima do limite, ante parcial, heads-up, níveis e potes paralelos/conservação.
- API: rotina de biblioteca/HTTP/segurança aprovada; novo contrato dos níveis em Play Money/Jogo Real preserva os antes das outras modalidades. Rust fmt/clippy sem warnings. Frontend: 88 testes, TypeScript, ESLint, Vite e auditoria npm sem vulnerabilidades. Nove regressões editoriais e dez contratos do executor aprovados; documentação sincronizada em 22 arquivos.
- As 30 produções foram renderizadas com a encenação compartilhada. Entrada comunitária de 0,6 s, 1 s fechada, virada de 0,8 s, flop escalonado em 0,35 s e pausa de 2 s antes da extra. Testes/manifestos conferem tempos, identidade das cartas e elegibilidade. Ante, pagamentos, totais e pote são distintos; exemplos independentes identificados. Renderer Academy v5/editorial v4; snapshots e avisos dos três vídeos `prior_rules` preservados.
- Filme principal: 455,40 s, 30 cenas, oito capítulos nativos/HTML, 103 legendas, H.264/AAC 1280×720/30 fps. Voz `pt-BR-AntonioNeural`, taxa `-3%`, guia “painépou” preservado. Conferência das 30 cenas, 1.377 textos sem overflow e cobertura completa dos 30 segmentos falados. Áudio medido em -16,43 LUFS e pico -0,99 dBTP. SHA-256 MP4 `1b63b23730542a79f148e01313e488e6e3c7689010d1c737603f703bd3b351b1`.
- `render.py --check` aprovou as 30 produções/90 arquivos, total 169,1 minutos. URLs usam revisão conjunta de MP4/VTT/WebP; VTT e transcrições em LF mantêm os hashes no Windows/Linux. A decodificação integral dos 30 novos MP4 terminou com sucesso antes da publicação.
- Chrome isolado: 56 vistas das 28 aulas (1440/390 px), seis da home/Academy (1440/390/320 px) e três do treino conectado à API local. Sem exceções/overflow; capítulos, salto sem metadados, legendas, transcrição, reprodução sob demanda e redirecionamento logado conferidos com sessão/progresso sintéticos. Treino confirmou ante R$ 1 e pote R$ 11,50. Medição dos textos das 376 cenas técnicas sem cortes. Evidências em `artifacts/pineapple-v2/` e logs `artifacts/pineapple-v2-*.log`.

Commit/push/deploy autorizados no plano. Publicação concluída em 10/10 após todos os gates, conforme registro abaixo. **Escuta editorial humana integral permanece pendente**; inspeção visual, análise do sinal e cobertura das palavras não são atestação de audição humana.

### Disponibilidade dos registros no CI (09/10/2026)

As primeiras execuções de `1980c4d` falharam antes dos testes afetados por timeouts de `auth.docker.io` e HTTP 429 ao buscar imagens. Os builds agora configuram o cache público `mirror.gcr.io` tanto no daemon quanto no BuildKit; o bootstrap do BuildKit busca explicitamente a cópia nesse cache, fixada por digest, com até três tentativas. Isso evita que o bootstrap retorne ao endpoint de autenticação indisponível do Hub. Os serviços PostgreSQL/Redis usam a publicação oficial Docker no ECR Public, com os mesmos digests anteriores, conferidos por SHA-256. As imagens da aplicação, scans, SBOM, assinaturas e gates permanecem obrigatórios. Referências: [configuração oficial BuildKit](https://docs.docker.com/build/ci/github-actions/configure-builder/#registry-mirror) e [Docker Official Images no ECR](https://www.docker.com/blog/news-from-aws-reinvent-docker-official-images-on-amazon-ecr-public/).

A decodificação integral local dos 30 MP4 terminou com sucesso: 169,1 minutos, registrados em `artifacts/academy-audit/media.json`. O CI completo e a janela ociosa foram confirmados para a publicação registrada abaixo.


O scan da imagem web de `4c6078cb` encontrou quatro ocorrências HIGH no binário Caddy: CVE-2026-78667, CVE-2026-78669 (stdlib e x/net) e CVE-2026-97031, divulgadas em 08/10. O Caddy 2.11.4 passa a ser recompilado com Go 1.26.9 (imagem fixada por digest), x/net 0.60.0 e suas versões mínimas de x/crypto 0.57.0 e x/text 0.42.0. Nenhuma exceção de scan foi adicionada. Fontes do projeto Go: [Range HTTP](https://pkg.go.dev/vuln/GO-2026-6609), [HTTP/2](https://pkg.go.dev/vuln/GO-2026-6611) e [TLS](https://pkg.go.dev/vuln/GO-2026-6607).


### Publicação conjunta Pineapple v2 — 10/10/2026

Publicado `ff2f0b61c5aae50ef9ae8bbe6519bbbe7e22e724` às **04:08:31 UTC / 01:08:31 em São Paulo**, após [Rust CI](https://github.com/leofran2204/poker-platform/actions/runs/37995757466) e [Container Supply Chain](https://github.com/leofran2204/poker-platform/actions/runs/37995757464) concluírem com sucesso. Inclui testes de contrato PostgreSQL/Redis, cobertura, builds, scans, SBOM, assinaturas e atestações. API, interface, Academy v3 e 30 produções entraram juntas; Caddy usa Go 1.26.9/x/net 0.60.0 corrigidos.

Backup validado em `/opt/poker-platform/backups/pineapple-v2-ff2f0b61c5aa`: dump PostgreSQL com índice conferido, volumes Caddy e hashes, imagens anteriores disponíveis. Durante o bloqueio de novas entradas, repetidas as consultas e criado um novo dump imediatamente antes da troca. As três conferências encontraram zero assentos cash ativos, torneios em andamento, inscrições em eventos agendados e snapshots de mesas. Nenhum jogo foi cancelado. PostgreSQL/Redis mantiveram containers, imagens e início; `.env` idêntico por hash. Proxy normal restaurado, quatro serviços saudáveis e aplicações em UID 10001. As 60 migrations/checksums permanecem corretas; nenhuma migration nova.

Os 90 arquivos na imagem servida conferem integralmente com o Git por SHA-256. Pelo domínio público, as 30 produções responderam Range 206 com os primeiros bytes corretos; VTT e capas conferidos integralmente, assim como o MP4 principal. Home/Academy e 28 aulas passaram em 62 vistas desktop/celular, com capítulos, 103 legendas no principal, transcrição, teclado/foco, reprodução sob demanda e avisos históricos. Sem overflow ou exceções JavaScript. Sessão/progresso sintéticos para esses testes de interface; não é um teste de autenticação real.

A API pública real confirmou Academy v3/regra v2, ante nominal e pago de 100 centavos por `bot-2`, pote inicial de 250, máximo inicial de 350 e pote de 1.150 após raise/calls. Rejeitou all-in acima do limite e catálogo versão 2. Mesa de treino passou em 1440/390/320 px, totalizando **65 vistas**. Os torneios Pineapple públicos em `play` e `real` exibem ante igual ao BB em todos os níveis. Nenhuma carteira foi movimentada pelos testes. Evidências: `artifacts/pineapple-v2/release/` (CI, servidor, HTTP, navegador e capturas). A escuta editorial humana integral continua pendente e separada das validações técnicas.

## Dependências da API e disponibilidade do CI — 19/10/2026

O CI de `83002fb9` identificou nove advisories no lockfile da API: cinco em `quinn-proto` 0.11.16 e quatro em `rustls` 0.23.45. Aplicadas as versões corrigidas 0.11.18 e 0.23.46, respectivamente, conforme [RustSec para Quinn](https://rustsec.org/advisories/RUSTSEC-2026-0340.html) e [RustSec para rustls](https://rustsec.org/advisories/RUSTSEC-2026-0347.html). Isso é uma correção de dependências; a presença no lockfile não demonstra que cada funcionalidade vulnerável esteja ativa no binário.

`cargo audit` local sem alertas após atualizar a base; `fmt --check`, `clippy --locked --all-targets -D warnings` e 82 testes de biblioteca da API aprovados, com dois contratos de banco ignorados conforme seus gates existentes. Não foi executada carga massiva nem alterada migration. O resolvedor reutilizou `getrandom` 0.3.4 na dependência de testes `tempfile`, sem atualizar sua versão.

A etapa PostgreSQL/Redis falhou antes de executar testes por `toomanyrequests: Data limit exceeded` no ECR Public, inclusive nas três tentativas do runner. Os serviços do CI passam a usar as cópias disponíveis em `mirror.gcr.io`, preservando os digests de ambas as imagens. Os manifests responderam HTTP 200 e `Docker-Content-Digest` idêntico ao versionado. Esse cache pode expirar; uma futura ausência deve falhar o job, nunca dispensar os contratos. Os scans, auditorias, SBOM, assinaturas e atestações permanecem obrigatórios.

## Publicação da API corrigida e retomada automática — 07/11/2026

Publicado o código `36731d5c37c6b03034ad4f213cd1b82351c35bbf` às **18:58:25 UTC / 15:58:25 em São Paulo**. [Rust CI](https://github.com/leofran2204/poker-platform/actions/runs/39347692636) e [Container Supply Chain](https://github.com/leofran2204/poker-platform/actions/runs/39347692599) haviam concluído com sucesso em 20/10, incluindo os contratos PostgreSQL/Redis e a auditoria das dependências corrigidas. Na retomada, verificados branch, saúde atual, janela ociosa e atestação GitHub da imagem exata `ghcr.io/leofran2204/poker-api@sha256:4b8572928bd2b2c3456f3e0d6986457d3f75220f4390290116ec51365ca0ee21`; label de revisão corresponde ao commit aprovado.

Checkout da VPS atualizado por fast-forward, incluindo `restart: unless-stopped` nos quatro serviços. A política aplicada em 19/10 já estava ativa: na conferência de 07/11, os quatro containers tinham iniciado em 23/10 após outro boot do host e estavam saudáveis. Não foi provocado reboot para esta publicação.

Backup restrito em `/opt/poker-platform/backups/api-security-36731d5c37c6`, com dump, índice validado e SHA-256; imagem anterior retida para rollback. Antes da troca, bloqueadas temporariamente mutações da API e conexões WS, conferidos HTTP 503, drenadas requisições e repetida a janela ociosa com a API parada. Zero assentos, torneios em andamento/agendados com inscritos e snapshots. Novo dump consistente imediatamente antes do boot. Somente a API foi recriada; frontend, PostgreSQL e Redis preservaram IDs, imagens e início. `.env` preservado por hash; configuração normal do Caddy recarregada após a troca.

Quatro serviços saudáveis e com política de reinício conferida, aplicações UID 10001, 60 migrations bem-sucedidas e checksums conferidos na pré-validação. Log da API confirmou `Migrations applied` e zero linhas ERROR no boot. Pela conexão externa, home, Academy, lobby, `/health`, `/caddy-health` e `/api/lobby/tables` responderam HTTP 200; treino Pineapple v3 confirmou regra v2, ante pago de 100 centavos, pote inicial de 250, máximo de 350 e pote de 1.150 após raise/calls. Sem login, apostas ou transações de carteira de teste. Evidências operacionais no backup: `published.json`, `verification.json` e pré-validações. Atualizações documentais posteriores não alteram essa imagem.

## Publicação: integridade, saque por titularidade e passivo — 08/11/2026

Publicado o código `ecc79b816d0757e6212be1e820b80c4e956d6180` às **01:38:11 UTC de 08/11 / 22:38:11 de 07/11 em São Paulo**, após [Rust CI](https://github.com/leofran2204/poker-platform/actions/runs/42432659772) e [Container Supply Chain](https://github.com/leofran2204/poker-platform/actions/runs/42432659865) concluírem com sucesso no commit exato, incluindo os contratos PostgreSQL e a nova jornada e2e (KYC → depósito → mesa → saque) no CI.

Imagens: `ghcr.io/leofran2204/poker-api@sha256:676c59cb4544719d8cbce72789b5f853c8c8d041614ae5ec9bd23a4f8cbf9087` e `ghcr.io/leofran2204/poker-web@sha256:50adb6b4fefa372c8a24c529bde222906731dd8560e6e08572000f9e0b6054a5`. As atestações de proveniência SLSA e SBOM foram lidas pela API pública de atestações do GitHub: proveniência de `.github/workflows/supply-chain.yml` em `refs/heads/master`, commit `ecc79b816d07`; na VPS, label `org.opencontainers.image.revision` conferido igual ao commit. Assinatura Sigstore verificada após a troca com `cosign` v3.1.4 (checksum oficial conferido): as duas imagens foram assinadas pela identidade `supply-chain.yml@refs/heads/master` do repositório, com o commit `ecc79b81` exigido no certificado; o controle negativo com outro commit foi recusado, e na VPS os containers rodam exatamente esses digests. Desde então a verificação roda antes da troca em `scripts/vps-release-attested.sh`.

Antes da troca, scan local com Trivy 0.69.1 (`vuln,misconfig,secret`, HIGH/CRITICAL, `--ignore-unfixed`) zerado nas duas imagens e `caddy validate` aprovado. Backup restrito em `/opt/poker-platform/backups/release-ecc79b816d07` (dois dumps com índice validado, SHA-256, script e log da publicação); imagens anteriores retidas como `poker-api:before-ecc79b816d07` e `poker-frontend:before-ecc79b816d07`. Mutações bloqueadas pela parada da API (sem modo 503 no Caddy), com a janela ociosa repetida e dump renovado com a API parada. API e frontend recriados sem build; PostgreSQL, Redis e `.env` (hash) preservados.

Quatro serviços saudáveis, 60 migrations bem-sucedidas (máx. 60), `Migrations applied` no boot e zero linhas ERROR/WARN. Pela conexão externa, `/`, `/health`, `/caddy-health`, `/api/lobby/tables` (5 mesas), `/curso` e `/api/presence/online` responderam HTTP 200; `/api/admin/liabilities` respondeu 401 sem sessão (rota nova, exige admin). Sem login, apostas ou transações de teste em produção; as regras de saque e de cartas foram validadas pelos testes de contrato com PostgreSQL local e no CI.

## Publicação: saque Pix com CPF do titular — 08/11/2026

Publicado `182e13bda4647eec5aabeb7e17c72474ea01d3aa` às **03:01:58 UTC / 00:01:58 em São Paulo**, após [Rust CI](https://github.com/leofran2204/poker-platform/actions/runs/42440186483) e [Container Supply Chain](https://github.com/leofran2204/poker-platform/actions/runs/42440186482) verdes no commit. Imagens `ghcr.io/leofran2204/poker-api@sha256:4e44df099336a8a6f3d15eac5a7f0734ccb88edf86e6524c7f60dd9d930f109a` e `ghcr.io/leofran2204/poker-web@sha256:dd3d1c3bdcf5cd76df744a2c8b0261c1aba8bd443a5571d1f2cddedd95f1ce38`, com assinatura `cosign` verificada na VPS antes da troca (identidade `supply-chain.yml@refs/heads/master`, repositório e commit exigidos no certificado) e label de revisão conferido, primeiro em `--verify-only` e de novo na publicação.

Frontend: lint, 95 testes Vitest (7 novos em `lib/pixWithdraw.test.ts`) e build aprovados. Janela ociosa (zero online, assentos, torneios e transações pendentes), dois dumps validados em `/opt/poker-platform/backups/release-182e13bda464`, imagens anteriores retidas como `before-182e13bda464`. Quatro serviços saudáveis, 60 migrations, zero ERROR no boot, `.env` preservado. Externamente, `/`, `/health`, `/caddy-health`, `/api/lobby/tables`, `/carteira` e `/curso` responderam HTTP 200, e o bundle publicado contém o aviso de mesma titularidade. A tela de saque não foi exercitada com login em produção.

## Publicação: painel de passivo no admin — 08/11/2026

Publicado `4088a5f65c393f30908cf137c446d8e50eb09d77` às **03:25:00 UTC / 00:25:00 em São Paulo**, após [Rust CI](https://github.com/leofran2204/poker-platform/actions/runs/42441855964) e [Container Supply Chain](https://github.com/leofran2204/poker-platform/actions/runs/42441855921) verdes. Imagens `ghcr.io/leofran2204/poker-api@sha256:79845d5b72fc027aa12db854a4ae82e1a1e43f149a2c1f98f7e185bdb5599ad8` e `ghcr.io/leofran2204/poker-web@sha256:2bff0ebf36c4cd5b1be9dfeeeba2eab64631498eec432e2f737811b3a828a064`, assinatura `cosign` e label de revisão verificados na VPS antes da troca (em `--verify-only` e na publicação).

Frontend: lint, 98 testes Vitest (3 novos em `lib/liabilities.test.ts`) e build aprovados. Janela ociosa, dois dumps em `/opt/poker-platform/backups/release-4088a5f65c39`, imagens anteriores retidas como `before-4088a5f65c39`. Quatro serviços saudáveis, 60 migrations, zero ERROR no boot, `.env` preservado. Externamente, `/`, `/health`, `/caddy-health`, `/api/lobby/tables` e `/admin` responderam HTTP 200 e `/api/admin/liabilities` 401 sem sessão; o chunk publicado `AdminOverviewPage-upcn987E.js` contém o painel e o bundle principal mantém o aviso de saque. O painel não foi aberto com sessão de admin em produção.

## Guarda de rede e Loss Deflator em todos os jogos — verificação local de 08/11/2026

Não publicado; sem commit. Escopo e decisões em [DEVELOPMENT_LOG](DEVELOPMENT_LOG.md#2026-11-08--varredura-documental-guarda-de-rede-e-loss-deflator-em-todos-os-jogos-local).

- Motor: `fmt` e `clippy --all-targets -D warnings` limpos; 1.867 testes de biblioteca em debug (59 s, como a CI) e todas as suítes de integração em release, inclusive `short_deck_omaha_massive` e `short_deck_massive` (esta corrigida: o teste esperava sequência acima de trinca no Short Deck). Novos: equivalência da pontuação de 5 cartas com o avaliador oficial nas 2.598.960 mãos (`--ignored`, release, 9 s), equity exata do Omaha no flop contra força bruta, enumeração do Pineapple no turn (59.280 desfechos) contra referência independente, determinismo do Monte Carlo e mãos jogadas de Omaha (flop) e Pineapple (turn) com deflator auditado.
- API: `fmt`, `clippy -D warnings`, 93 testes de biblioteca (5 da guarda de rede). PostgreSQL 15 descartável com as 61 migrations aplicadas uma por transação; a 061 reaplicada sem erro. Contratos: 13 de `api_tests` (inclusive `test_contract_cash_join_refuses_same_network_at_same_table`: mesma /24 recusada com 403, outra rede aceita, reentrada idempotente, impressões apagadas ao sair), 5 de `payments_tests` e a jornada e2e. O container foi removido ao final.
- Frontend: 102 testes Vitest (4 novos em `pages/TermsPage.test.ts`, que confrontam a página com `TERMOS_DE_USO_E_SERVICO.md`), `tsc`, ESLint e build Vite, com o Node 24 do harness.
- Mídia: filme do Pineapple regravado só na cena final (“O Loss Deflator também vale no Brazilian Pineapple…”), 457,43 s, 104 legendas. `render.py --check --decode --only pineapple` e `--check` das 30 publicações aprovados; 9 testes de `test_narration.py`. A síntese exigiu `edge-tts` 7.3.1: a 7.2.7 passou a receber 403 do serviço. Escuta humana integral continua pendente.
- Documentação: `documentation-sync --check` (22 documentos), links locais e `git diff --check` aprovados.

## Vídeos em Remotion — verificação local de 09/11/2026

Escopo: filme completo das regras do Brazilian Pineapple, aula M2L2 completa e trailer, em `ZeroTiltCurso/editorial/motion/`. Finais em `artifacts/video-pilot/<id>/` (fora do git e do player até aprovação).

- Código: `tsc --noEmit` (Node 24) aprovado. Conferência estática de todas as deixas `cue("…")` da aula e do trailer contra as palavras cronometradas da narração: nenhuma ausente. O filme de regras usa deixas tolerantes, com posição proporcional quando a frase não existe.
- Narração: Azure Speech F0, voz `pt-BR-AntonioNeural`, tempos por palavra do serviço; 9.776 caracteres no mês, abaixo do limite de 400 mil do `narrate.py`.
- Revisão visual: prévias em meia resolução, um quadro ao fim da fala de cada uma das 46 cenas e quadros isolados após cada correção. Nove defeitos de leiaute ou sincronia corrigidos (lista no [registro](DEVELOPMENT_LOG.md#2026-11-09--filme-completo-do-pineapple-e-aula-m2l2-completa-em-remotion-local)).
- Mídia final, conferida com FFprobe e FFmpeg:

| Vídeo | Duração | Formato | Volume integrado | Pico real | Capítulos |
|---|---|---|---|---|---|
| `pineapple-rules.mp4` | 460,6 s | 1080p30, H.264 + AAC | −16,0 LUFS | −1,4 dBTP | 8 |
| `academy-m2l2.mp4` | 298,3 s | 1080p30, H.264 + AAC | −16,0 LUFS | −1,3 dBTP | 8 |
| `pineapple-trailer.mp4` | 91,6 s | 1080p60, H.264 + AAC | −16,0 LUFS | −1,4 dBTP | 5 |

- Decodificação integral dos três MP4 sem erro. O pico fica 0,1 a 0,2 dB acima do alvo de −1,5 dBTP por causa da codificação AAC, ainda abaixo de −1 dBTP.
- Vocabulário nas transcrições: zero ocorrência de “rua”, “rival” ou “adversário”; “street” presente na aula e no trailer.
- Não verificado: escuta editorial integral pelo proprietário e integração ao player (manifesto e auditoria no padrão de `courseAudit.test.ts`), que só ocorre após aprovação.

## Pineapple Draw — verificação local de 09/11/2026

Escopo: troca do Brazilian Pineapple pelo Pineapple Draw em motor, API, frontend, Academy, mídia, documentação e migration 062.

- Motor: `fmt` e `clippy -D warnings` (todos os alvos); 1.875 testes de biblioteca; integração `full_validation_situations` (19), `pineapple_draw_betting_examples` (5), `loss_deflator_regressions` (9), `variant_audit_regressions` (9), `loss_deflator_seven_percent` e `pineapple_draw_simulation` (rápido em debug; referência de 20 mil mãos por modalidade em release com `massive-tests`). Os testes cobrem distribuição de 3 cartas, troca em cada street, ordem, rejeição de fold/aposta na troca, all-in sem troca, ausência de troca sem apostas possíveis, baralho de 6 jogadores trocando sempre (44 cartas únicas), equity exata no flop e no turn e deflator com troca antes do all-in.
- API: `fmt`, `clippy -D warnings` e 97 testes de biblioteca, inclusive WebSocket com `drawing`/`draw_count` sem vazar cartas, ação automática `keep` na troca, `swap` por posição, bots na troca e laboratório v4 (pergunta da troca com empates aceitos).
- Frontend: 102 testes, `tsc`, ESLint e build Vite. Mesa com seleção de carta e sem fold na troca; demonstração da home com 3 cartas e trocas conferida (vencedor declarado, 2 + 3, cartas únicas); auditoria do curso com 22 vídeos técnicos publicados, m5l3–m5l5 sem vídeo e 75 arquivos públicos sem mídia do jogo antigo.
- Editorial: `test_narration` (9) e `render.py --check` com 25 publicações (135,9 minutos); m0l4, m5l1 e m6l3 renderizadas de novo e conferidas.
- Documentação: `documentation-sync --write/--check` com o novo campo `money_modes`.
- Migration 062 em PostgreSQL 16 descartável (cluster temporário, apagado ao fim): 001–062 aplicadas uma por transação e a 062 reaplicada sem erro; mesas Brazilian Pineapple fechadas e privadas, `PM · Pineapple Draw 0,50` aberta, nenhuma mesa Pineapple no Jogo Real, torneio Play Money convertido (6/18) e o de Jogo Real cancelado, auditoria `CATALOG_PINEAPPLE_DRAW_062` única. Versão sem reembolso: com um jogador sentado em cada mesa (Play Money e Jogo Real) e uma inscrição em cada torneio, a 062 aplica; a mesma devolução do início da API credita as fichas (R$ 90 em Play Money e R$ 64 no Jogo Real nas contas de teste) com dois `CASH_OUT`; o torneio Play Money vira Pineapple Draw com o inscrito; o de Jogo Real é cancelado com uma linha de auditoria sem reembolso, sem duplicar ao reaplicar. Com torneio Brazilian Pineapple em andamento, a 062 aborta e nada muda.
- Não verificado: campanha `full-validation -Phase pineapple` (exige autorização de carga), jogo em navegador com pessoas reais e publicação.

## Vídeos do Pineapple Draw — verificação local de 10/11/2026

Escopo: filme completo das regras, trailer e aulas m5l3–m5l5, produzidos em `ZeroTiltCurso/editorial/motion/` e publicados localmente no site (`Frontend-Web/public/videos/`, `courseContent.json`, `pineappleDrawFilms.json`). Sem commit, push ou deploy.

- Regras: `pineapple_draw_film_examples` (5 testes, motor em release) confere os cinco roteiros: melhor 2 + 3 ou invalidez declarada, ordem da rodada de troca e cartas únicas, descarte fora do melhor par, pagamentos antes e depois das trocas. Entrou no Rust CI.
- Código: `tsc --noEmit` do projeto Remotion; frontend com 103 testes Vitest (novo: filmes contra manifesto, transcrição e mídia; 25 vídeos técnicos; 90 arquivos públicos), `tsc -b`, ESLint e build Vite (90 mídias em `dist/videos`), com o Node 24 do harness.
- Revisão visual: prévias em meia resolução (93 quadros), quadros isolados após cada correção e quadros dos arquivos finais (um por capítulo do filme, aberturas, exercícios e fechamentos). Fontes Fraunces (com itálico verdadeiro) e Manrope carregadas do projeto, sem rede.
- Mídia final, medida com FFprobe e o filtro EBU R128 do FFmpeg:

| Vídeo | Duração | Formato | Tamanho | Volume integrado | Pico real | Capítulos | Legendas |
|---|---|---|---|---|---|---|---|
| `zt-pineapple-draw-rules-v1` | 587,3 s | 1080p30, H.264 + AAC | 36,9 MB | −16,0 LUFS | −1,5 dBTP | 10 | 174 |
| `zt-pineapple-draw-trailer-v1` | 73,8 s | 1080p30, H.264 + AAC | 4,3 MB | −16,0 LUFS | −1,5 dBTP | 1 | 24 |
| `zt-academy-m5l3-v4` | 244,9 s | 1080p30, H.264 + AAC | 16,6 MB | −16,0 LUFS | −1,5 dBTP | 7 | 77 |
| `zt-academy-m5l4-v4` | 191,8 s | 1080p30, H.264 + AAC | 11,5 MB | −16,0 LUFS | −1,2 dBTP | 6 | 60 |
| `zt-academy-m5l5-v4` | 194,4 s | 1080p30, H.264 + AAC | 11,7 MB | −16,0 LUFS | −1,5 dBTP | 6 | 60 |

- Codificação web: CRF 26 escolhido após comparar 23, 26 e 28 em recorte 1:1 (sem diferença visível; 23 rendia 21 MB na m5l3). Capas WebP 1280×720.
- Publicação: `render.py --check --decode` aprovou as 30 publicações (28 aulas e os 2 filmes, 157,4 minutos), com decodificação integral de todos os MP4, legendas dentro da duração e sem sobreposição, transcrição igual às legendas, capítulos e hashes de roteiro, narração, aula e mídia.
- Player em Chrome headless (DevTools Protocol, build Vite servido localmente), em 1440 e 390 px: botão “Assistir ao trailer” visível; modal com abas trailer/regras; os dois MP4 carregaram em 1920×1080 com a duração prevista, 24 e 174 legendas, busca até o fim e capítulos; sem overflow horizontal nem exceção de JavaScript. Cartão do filme no topo da Academy presente. Ajustes após a captura: título do modal sem repetir o do player e abas com duração compacta (cabem numa linha a 390 px).
- Vocabulário: zero ocorrência de “Brazilian”, “rua”, “rival” ou “adversário” nas cinco transcrições (o teste de auditoria cobre os filmes); “street” e “oponente” usados onde o roteiro precisa.
- Não verificado: escuta editorial integral pelo proprietário; player das aulas m5l3–m5l5 em sessão logada (para visitante elas aparecem bloqueadas, como as demais trilhas; o componente é o mesmo do modal conferido).

## Publicação: Brazilian Pineapple, site reconstruído e vídeos novos — 14/11/2026

Publicado `073d73aa448af3e0b64bda1ab150ae4179706202` às **19:02:11 UTC / 16:02:11 em São Paulo**, com Rust CI e Container Supply Chain verdes. Imagens `ghcr.io/leofran2204/poker-api@sha256:ffb9aecd8e260a52d54f9a7b3b742f0e72c0e20414b00120588658b361c94631` e `ghcr.io/leofran2204/poker-web@sha256:ce79b9f9c6f1855a14cbd4231b224c5702be01671d088c641d5be4fb22c6dab1`, assinatura `cosign` e label de revisão verificados na VPS (em `--verify-only` e na publicação).

Antes da troca, consulta só de leitura: 5 contas que não são bots, R$ 10,00 de saldo real no total, nenhum jogador sentado, nenhuma inscrição em torneio e nenhuma transação pendente. Dois dumps em `/opt/poker-platform/backups/release-073d73aa448a`. Depois: quatro serviços saudáveis, 62 migrations bem-sucedidas (061 e 062 aplicadas nesta troca), `Migrations applied` no boot, zero linhas ERROR, `.env` preservado, as mesmas 5 contas e os mesmos R$ 10,00. Externamente, `/` e `/api/health` responderam HTTP 200 e os vídeos `zt-pineapple-draw-rules-v1.mp4`, `zt-pineapple-draw-trailer-v1.mp4` e `zt-academy-m5l3-v4.mp4` responderam 200 como `video/mp4`.

Auditoria de dependências: `rustix` do motor atualizado para 1.1.6 (RUSTSEC-2026-0486). RUSTSEC-2026-0459 (`libsqlite3-sys`, via `sqlx-sqlite`) entrou como exceção em `API-Axum/.cargo/audit.toml`, aprovada pelo proprietário em 14/11: o pacote fica só no `Cargo.lock`, `cargo tree -i libsqlite3-sys --target all` não acha caminho compilado e nem o `sqlx` 0.9.0 aceita a versão corrigida (`>=0.39`). Retirar a exceção quando o `sqlx` aceitar.

Não verificado em produção: nenhuma mão de Brazilian Pineapple foi jogada no site publicado (o teste completo, 12 mãos com 4 jogadores, foi numa cópia local do banco); som e vibração da mesa não foram ouvidos.

## Publicação: carteira com depósito e saque em cartões — 14/11/2026

Publicado `9dd61f7c41114ac1f40c5f8fb3e4ff5a12213b2c` às **19:26:40 UTC / 16:26:40 em São Paulo**, com Rust CI e Container Supply Chain verdes. Imagens `ghcr.io/leofran2204/poker-api@sha256:b7989ea45ce0301c819c17a6459aea0386fad9b5ca383386d73c5b55cda5b67b` e `ghcr.io/leofran2204/poker-web@sha256:284a8d8834d39968e27339fba1217e102dec0b5967e0a4c0e89bce273d7f7e4b`, assinatura `cosign` e label de revisão verificados na VPS (em `--verify-only` e na publicação). Dois dumps antes da troca; quatro serviços saudáveis, 62 migrations, zero linhas ERROR, `.env` preservado, as mesmas 5 contas e R$ 10,00 de saldo real. `/` e `/api/health` responderam HTTP 200 e o CSS publicado contém `zt-wallet-real`.

Local: `tsc`, ESLint, build e Vitest 103/103; a carteira em Jogo Real foi vista no computador (1366 px) e no celular (390 px) numa cópia local do banco. Os scripts e2e que jogam Brazilian Pineapple passaram a responder ao descarte do river; só a sintaxe foi conferida (`node --check`), sem rodar a bateria.

Não verificado em produção: a carteira em Jogo Real não foi aberta com sessão no site publicado.

## Publicação: refresh token de uso único, banco de tempo e mesas da home — 19/11/2026

Publicado `17211d39947f0637856c4f7642c258cd46f5b79a` às **00:01:18 UTC de 20/11 / 21:01:18 de 19/11 em São Paulo**, com Rust CI e Container Supply Chain verdes. Imagens `ghcr.io/leofran2204/poker-api@sha256:0215b3ec33ea70e2aa15b64079f201fb6ce378a4b593a5e4ef1441e2457d5676` e `ghcr.io/leofran2204/poker-web@sha256:97bc298d36c25eacb687da12bcd70d5e99ab8111186202a818dc01f575fe2984`, assinatura `cosign` (v3.1.4) e label de revisão verificados na VPS (em `--verify-only` e na publicação). A VPS estava em `9dd61f7` com 62 migrations; depois da troca, 63 (tabela `auth_refresh_tokens` criada), um "Migrations applied", zero linhas ERROR, `.env` preservado, quatro serviços saudáveis, as mesmas 77 contas e R$ 10,00 de saldo real. `/` e `/api/health` responderam HTTP 200; `POST /api/auth/logout` existe (415 sem corpo JSON, em vez de 404) e o JavaScript publicado chama essa rota.

Até o CI ficar verde: o `75c08475` reprovou em dois avisos publicados depois de 14/11, alheios às mudanças. A CVE-2026-97054 (HIGH, `net/http`) no Go 1.26.10 que compila o Caddy da imagem web foi corrigida com o Go 1.26.11 (`1df99174`, Trivy local limpo). O RUSTSEC-2026-0543 (`chrono` sem manutenção, só aviso, sem versão corrigida) virou exceção temporária aprovada pelo proprietário (`17211d39`); a troca do `chrono` está na S28.

Local: site com `tsc`, ESLint, Vitest 137/137 e build; API com fmt, clippy `-D warnings` e testes (os de banco rodaram no CI); motor com 1.951 testes, nenhuma falha; `documentation-sync --check`.

Não verificado em produção: nenhuma mão foi jogada com o banco de tempo no site publicado, e o refresh token de uso único não foi exercitado com uma sessão real.

## Publicação: Brazilian Pineapple v4, relógio de 15 + 5 s e suíte massiva — 22/11/2026

Publicado `195ae908b0ba588c6da06f7465a49a52b11905c5` às **02:32:38 UTC de 23/11 / 23:32:38 de 22/11 em São Paulo**, por ordem do proprietário, com Rust CI e Container Supply Chain verdes. Imagens `ghcr.io/leofran2204/poker-api@sha256:1e9057d317702170a12f6cecee28ceb558b7d284680f9fc84411f57f1843e452` e `ghcr.io/leofran2204/poker-web@sha256:95553b8bdcd1212278ee226b617c30c0fd4e577136c9fc6ceec02b9fa9882d9b`, assinatura e revisão conferidas pelo `vps-release-attested.sh`. Dois backups (o segundo com a API parada), migrations 63 (sem migration nova), boot com 1 "Migrations applied" e nenhuma linha ERROR, `.env` preservado, os 4 serviços saudáveis. No site publicado, conferido que o código serve a `pineapple_draw_v4`, o cartão "3 CARTAS · 3 TROCAS · 1 DESCARTE" e o "TEMPO EXTRA", e que o vídeo novo da m6l3 tem o tamanho do arquivo local (9.979.829 bytes).

Até o CI ficar verde: o `121837b7` reprovou só na API, pelo aviso RUSTSEC-2026-0592 (`url` 2.5.8, publicado em 20/11; atualizado para 2.5.9) e por dois testes de `api_tests.rs` presos ao catálogo da Academy na versão 6. Os dois foram corrigidos no `195ae908`.

Antes da publicação:
- suíte massiva `pineapple_draw_massive` em release: 100 mil avaliações e 100 mil mãos com o Loss Deflator ligado, nenhuma divergência;
- campanha do Brazilian Pineapple v4 aprovada ([FULL_VALIDATION](FULL_VALIDATION.md#campanha-do-brazilian-pineapple-v4-com-relógio-de-15--5-s--22112026));
- site com `tsc`, ESLint, Vitest 150/150 e build; `test_narration` 13/13 e `render.py --check` com 30 publicações (24 delas na exceção temporária de voz antiga aprovada pelo proprietário);
- motor com fmt e clippy `-D warnings`; API com fmt e os testes do CI; `documentation-sync --check`.

Não verificado em produção: nenhuma mão de Brazilian Pineapple v4 foi jogada no site publicado, nem o relógio de 15 + 5 s com jogador real.

## Publicação: Loss Deflator com um cálculo por foto do all-in — 23/11/2026

Publicado `3f1ac1155ade3b62fd1dda334842acb997528276` às **08:41:41 UTC / 05:41:41 em São Paulo de 23/11**, por ordem do proprietário, com Rust CI e Container Supply Chain verdes. Imagens `ghcr.io/leofran2204/poker-api@sha256:8389a5ea56932aa49ff70c5750640e2cda22302693b88648566ab943d7d56cf1` e `ghcr.io/leofran2204/poker-web@sha256:a9ed2dd141e9ec65f454d0a7da8e2f80d3d8528cd5924434112a39f294a1e577`, assinatura e revisão conferidas pelo `vps-release-attested.sh`. Dois backups (o segundo com a API parada), migrations 63 (sem migration nova), boot com 1 "Migrations applied" e nenhuma linha ERROR, `.env` preservado, os 4 serviços saudáveis; `/api/health` e a home responderam 200 por fora.

Mudança: no Omaha 4 e no Brazilian Pineapple, perdedores da mesma foto do all-in dividem um único cálculo de equity, com o mesmo valor do cálculo individual; o fechamento da mão roda em `spawn_blocking` (cash e torneio). Antes da publicação: suíte completa do motor, `pineapple_draw_massive` em release (mesmas 140 devoluções de 22/11; mão mais lenta 12,2 s → 4,2 s), `short_deck_omaha_massive` em release, 111 testes da lib da API, `fmt` e `clippy -D warnings` nos dois.

Não verificado em produção: nenhuma mão com all-in multiway de Omaha ou Pineapple foi jogada no site publicado.

## Publicação: rebuy e add-on nos torneios — 23/11/2026

Publicado `73d0ab998c43435ca740429dd12ea04085d96a3c` às **21:02:18 UTC / 18:02:18 em São Paulo de 23/11**, por ordem do proprietário ("pode publicar"), com Rust CI e Container Supply Chain verdes. Imagens `ghcr.io/leofran2204/poker-api@sha256:c91751d826b9e7d8e0fd043a03f83df784f72a6b82d5a454a0e0988009808a55` e `ghcr.io/leofran2204/poker-web@sha256:c885e1d6ca1f73f707fb3ce503fcf2ca3db54d290bd730c09f8a8f51572e1340`, assinatura e revisão conferidas pelo `vps-release-attested.sh`. Dois backups (o segundo com a API parada), migrations 64 (nova: `064_tournament_rebuy_addon`), boot com 1 "Migrations applied" e nenhuma linha ERROR, `.env` preservado, os 4 serviços saudáveis. Por fora: `/api/lobby/tournaments` traz `allow_addon` e `current_level`; `POST /api/tournament/rebuy` sem login responde 401.

Mudança: rebuy e add-on configurados por evento no admin, rotas do jogador, fichas entregues no começo da mão seguinte, volta à mesa de quem foi eliminado, unicidade de `tournament_seats` só para assentos ativos. Gate novo no CI: `mtt_rebuy_and_addon_deliver_chips_and_charge_wallet` (PostgreSQL). Antes da publicação: API `fmt`, `clippy -D warnings`, testes determinísticos e contratos PostgreSQL; motor `fmt`, `clippy -D warnings` e 976 testes de torneio (suíte completa no CI); site tsc, eslint, Vitest 154/154 e build.

Não verificado em produção: nenhum rebuy ou add-on foi comprado no site publicado; nenhum torneio do catálogo tem add-on ligado (fica para o admin ou para a etapa 2, com bots).


## Publicação: entrada na mesa de torneio e bots no site publicado — 23/11/2026

Publicado `42551d906a74890f4a376644dd32ebbbfef803f2` às **23:02:50 UTC / 20:02:50 em São Paulo de 23/11**, dentro da etapa 2 autorizada pelo proprietário (testes e CI verdes), com Rust CI e Container Supply Chain verdes. Imagens `ghcr.io/leofran2204/poker-api@sha256:3305ab6cee4fc54aaf4f2ae69474e8074cb80f5fbb4981bb81e1771e6172dc54` e `ghcr.io/leofran2204/poker-web@sha256:84c8c4fab90e4b9ec6cc2334bb4e2553bc2fdb6a5b906d94ee54d9c85a77e220`, assinatura conferida; migrations 64; boot sem `ERROR`; `.env` preservado.

Mudança: a consulta do assento na entrada pelo WebSocket da mesa de torneio devolvia BIGINT onde a linha lê SMALLINT; desde 05/09/2026 ninguém entrava na mesa e o servidor desistia por todos. Gate novo no CI: `mtt_websocket_admission_reads_the_tournament_seat` (reproduz o `ColumnDecode` com a consulta antiga).

Verificado em produção com bots (Play Money, contas de teste com e-mail temporário, relatórios em `artifacts/ritual/`):
- Cash nas 4 modalidades com mesa cheia: 580 mãos; pagamentos + rake = pote e liquidação assinada em todas; soma das carteiras das contas = −rake ao centavo (31.761 + 14.524 + 9.910); potes paralelos, empates, all-in multiway, tempo esgotado e tempo extra, Loss Deflator (4 disparos), troca, manutenção e descarte. Nenhum jogador de fora sentou e nenhum assento ficou ativo depois.
- Torneio "PM · Texas R$25" depois da correção: 27 inscritos, 26 rebuys, 27 add-ons, mesa final, campeão; prêmios 100.000/60.000/40.000 = pool 200.000.
- Jogo Real sem dinheiro: saldo 0; mesa, torneio e saque recusados.

Não fechado: `tournaments.total_fees` não registrava o fee das inscrições pelo site (corrigido localmente com teste, entra na próxima publicação). Loss Deflator raro em mesa cheia por causa da regra de oponentes no pagamento do all-in (decisão do proprietário).


## Publicação: time no lugar do chrono e fee de inscrição — 24/11/2026

Publicado `f6e91d9d0c45ae7e67ff360e71a9e6436160b809` às **00:09:00 UTC de 24/11 / 21:09 em São Paulo de 23/11** (etapa 3 autorizada pelo proprietário), com Rust CI e Container Supply Chain verdes e imagens assinadas; migrations 64; boot sem `ERROR`; `.env` preservado.

Mudança: API sem `chrono` (RUSTSEC-2026-0543), com `time` e o módulo `clock.rs`; exceção retirada do `audit.toml` e `cargo audit` limpo. A inscrição pelo site passou a gravar o fee em `tournaments.total_fees` (`cf28cf8e`).

Verificado em produção: login, extrato e inscrição com cancelamento numa conta de teste (fee de 225 cobrado e devolvido). A coluna `total_fees` desse torneio tinha resíduo antigo carregado pela reidratação (1.575 com 1 inscrito): corrigido na etapa 4 (entradas × fee e migration `065`).

## Publicação: login por cookie e fee dos torneios — 24/11/2026

Publicado `f5d690addc565f6c079d0ab9125b02762cfa31fb` às **00:34:54 UTC de 24/11 / 21:34 em São Paulo de 23/11** (etapa 4 autorizada pelo proprietário), com Rust CI e Container Supply Chain verdes e imagens assinadas; migrations 65; boot sem `ERROR`; `.env` preservado.

Mudança: sessão por cookie `HttpOnly; Secure; SameSite=Strict` (`zt_access`, `zt_refresh`) com `X-ZT-CSRF` nas escritas; o site não guarda token no `localStorage`. Migration `065` corrige `tournaments.total_fees` (entradas × fee).

Verificado em produção com uma conta de teste: login com `X-ZT-Session: cookie` devolve 2 cookies com os três atributos e o corpo sem token; `me` pelo cookie; escrita sem `X-ZT-CSRF` = 403 e com ele = 200; refresh pelo cookie; logout apaga os cookies e o refresh seguinte dá 401. O bundle publicado envia `X-ZT-Session`. A 065 corrigiu 4 torneios (o "PM · Texas R$25" passou a mostrar 10.125 de fee, 27 × 375).

Não verificado em produção: login pelo navegador de um jogador real.

## Publicação: livro único do dinheiro — 24/11/2026

Publicado `c5c6ac201aaacdd87355f7241b5119c6bcb8bc90` às **02:46:25 UTC de 24/11 / 23:46 em São Paulo de 23/11** (etapa 6 autorizada pelo proprietário, depois da campanha `full-validation` completa aprovada), com Rust CI e Container Supply Chain verdes e imagens assinadas; migrations 66; boot sem `ERROR`; `.env` preservado.

Mudança: migration `066` (`wallet_ledger` só de inclusão, gatilho em `users`, saldo inicial, `wallet_ledger_mismatches`), motivo em todas as mudanças de saldo da API, extrato completo da carteira e conferência do admin.

Verificado em produção: 261 linhas de saldo inicial na virada; numa conta de teste, entrada e saída de mesa Play Money gravaram −2.500 e +2.500 com a referência da mesa e o saldo resultante; nenhuma linha `unspecified` e nenhuma divergência entre saldo e livro.

## Publicação: os 24 vídeos de voz antiga regravados — 24/11/2026

Publicado `c14c82c1a419559b686a72b19577b447bfb630c1` às **04:21:48 UTC / 01:21 em São Paulo de 24/11** (etapa 5 autorizada pelo proprietário), com Rust CI e Container Supply Chain verdes e imagens assinadas; migrations 66; boot sem `ERROR`; `.env` preservado.

Mudança: episódios ep11–ep13 e 21 aulas regravados no padrão de narração de 19–21/11; `LEGACY_VOICE_LESSONS` vazia; `&` falado como "e".

Verificado em produção: `zt-academy-m1l1-v3.mp4`, `zt-academy-m2l3-v3.mp4` e `ep12-mississippi-omaha-v3.mp4` servidos com o mesmo tamanho dos arquivos renderizados.

Não verificado: escuta dos vídeos pelo proprietário (amostra m1l1 de 60 s em `artifacts/voz-pronuncia/2026-11-23/`).

## Publicação: sqlx 0.9.1 e cargo audit sem exceções — 24/11/2026

Publicado `02f9ec2992ec10840902a6ab75fa3afd2bcee54c` às **23:28:58 UTC / 20:28 em São Paulo de 24/11** (pedido do proprietário: "pode fazer a troca do sqlx e publicar"), com imagens assinadas e conferidas (Container Supply Chain verde), dois dumps, migrations 66, boot com um "Migrations applied" e sem `ERROR`, `.env` preservado e health HTTPS 200.

Rust CI: no `02f9ec29`, só o job de cobertura do motor falhou, na segunda rodada dos mesmos testes (exit 101), com o motor igual ao do `c14c82c1`. Quatro rodadas locais de `cargo test --lib` passaram. O Rust CI completo ficou verde no `e00f7d59`, que só muda docs em cima do mesmo código. O teste instável ficou na DASHBOARD S28.

Mudança: `sqlx` 0.8.6 → 0.9.1; `AssertSqlSafe` nos três SQL montados com partes fixas; `libsqlite3-sys` 0.40.0 e `rsa` fora do `Cargo.lock`; `API-Axum/.cargo/audit.toml` apagado, `cargo audit --deny warnings` sem nenhuma exceção.

Verificado em produção: a API nova aceitou as 66 migrations gravadas pela versão antiga (mesmo checksum), como já tinha feito antes numa cópia local de `poker_ritual_local`.

Não verificado: login e mãos jogadas no site depois da publicação.

## Publicação: tempo esgotado sem aposta vira check — 24/11/2026

Publicado `679eb7c15449eec2c40b03b5760b0b08ff401877` às **00:54:05 UTC de 25/11 / 21:54 em São Paulo de 24/11** (autorização do plano de 24/11 e ordem do proprietário nesta sessão: "pode publicar"), com Rust CI e Container Supply Chain verdes no commit, imagem `ghcr.io/leofran2204/poker-api@sha256:727e563ddccc24e23f9d20c983ffe01d2a567ea08fa8beea4199fd5be582150d` assinada e conferida, dois dumps, migrations 66, boot com um "Migrations applied" e sem `ERROR`, `.env` preservado e health HTTPS 200.

Mudança (etapa 1 do plano de 24/11): tempo esgotado sem aposta a pagar dá check em todas as modalidades, em cash e torneio; com aposta, fold; saída voluntária da mesa cash desiste direto (BUSINESS_RULES 10.2 e 10.3).

Verificação local antes do push: `fmt`, `clippy --all-targets -D warnings`, `cargo test` da API (118 da biblioteca, com os dois testes novos do `game_actor`) e os 5 testes de torneio com PostgreSQL (`tournament_actor_tests -- --ignored`) num banco novo com as 66 migrations.

## Publicação: Loss Deflator em mesa cheia — 24/11/2026

Publicado `809bc84a0b347b3b90b72da43862027e97e7126b` às **02:41:06 UTC de 25/11 / 23:41 em São Paulo de 24/11** (ordem do proprietário nesta sessão: "prossiga etapa 2 e em seguida pode publicar"), com Rust CI e Container Supply Chain verdes no commit, imagens `ghcr.io/leofran2204/poker-api@sha256:fdc7e70fcc33ec70639cafea41f3d0cce506b0925f1361512d847f80e5b2d07f` e `ghcr.io/leofran2204/poker-web@sha256:3b0067e78873fcc478b2df1a3c7d3d66d252211bc874956aaaf919df7e28f5a7` assinadas e conferidas, dois dumps, migrations 66, boot com um "Migrations applied" e sem `ERROR`, `.env` preservado, health HTTPS 200 e `/termos` com o texto novo da 6.4 no pacote publicado.

Mudança (etapa 2 do plano de 24/11): os oponentes do all-in no Loss Deflator são os que ainda estavam na mão no fim da rodada de apostas em que ele foi pago (BUSINESS_RULES 11, Termos 6.4). Medição com as mesmas 2.000 mãos por mesa: devoluções 35 → 87 na mesa de 9 e 42 → 74 na de 6; fechamento mais rápido (pior mão 22,2 s → 15,2 s na mesa de 9). [Registro](DEVELOPMENT_LOG.md#2026-11-24--loss-deflator-em-mesa-cheia-oponentes-no-fim-da-rodada-do-all-in-etapa-2).

Verificação local antes do push: `fmt`, `clippy --all-targets -D warnings` do motor, 1.486 testes do motor (`game_loop`, `loss_deflator`, `side_pots`, `tournament`) com os 2 novos, Vitest da página de Termos, `tsc`, build do Vite e `documentation-sync --check`.

## Publicação: guarda de rede nos torneios — 25/11/2026

Publicado `25788bf09d827b6e3aca2abbc6eadccc10aa194b` às **03:19:01 UTC / 00:19 em São Paulo de 25/11** (ordem do proprietário nesta sessão: "prossiga p a etapa 3 e depois pode publicar"), com Rust CI e Container Supply Chain verdes no commit, imagens `ghcr.io/leofran2204/poker-api@sha256:d75f6e6269b39fddd8d293340e5a2bceb92f7aa2968c57c78d4aee87e7dab87a` e `ghcr.io/leofran2204/poker-web@sha256:6eedd82cf6a773598e0da0c16d96302142232328b5e58ff494b1ffaec2dde80f` assinadas e conferidas pelo `vps-release-attested.sh`, dois dumps, migrations 67 (nova: `067_tournament_network_guard`), boot com um "Migrations applied" e sem `ERROR`, `.env` preservado, os 4 serviços saudáveis. Por fora: `/api/health` 200, `/termos` 200 e o bundle servido traz o texto novo da cláusula 2.3 e a data de 25/11 (o texto antigo não aparece). No banco de produção: coluna `tournament_players.subnet_fingerprint` e gatilho `trg_clear_tournament_network_fingerprints` presentes.

Mudança (etapa 3 do plano de 24/11): contas da mesma rede ficam em mesas diferentes nos torneios (distribuição inicial, rebalanceamento, rebuy), a quarta inscrição da mesma rede é recusada com 403 sem débito, e a mesa final ou o torneio que cabe numa mesa ficam registrados em `audit_logs` (BUSINESS_RULES 15.5, Termos 2.3). [Registro](DEVELOPMENT_LOG.md#2026-11-25--guarda-de-rede-nos-torneios-etapa-3).

Verificação local antes do push: `fmt`, `clippy --all-targets -D warnings` da API, suíte inteira da API com `--include-ignored` num banco novo com as 67 migrations (125 da biblioteca, 38 de contrato, 5 do ator de torneio e demais, todos verdes), 5 testes unitários novos da guarda e o contrato novo `test_contract_tournament_register_refuses_fourth_account_of_same_network`, Vitest da página de Termos, `tsc`, build do Vite e `documentation-sync --check`.

## Publicação: inscrição tardia com assento — 25/11/2026

Publicado `62e76325a586902c88b2908eb2d19758a6fd31ca` às **03:45:05 UTC / 00:45 em São Paulo de 25/11** (ordem do proprietário nesta sessão: "corrija a inscrição tardia e depois pode publicar"), com Rust CI e Container Supply Chain verdes no commit, imagens `ghcr.io/leofran2204/poker-api@sha256:6ba1e08b543382721948a1c1d4fec50b1f5b6943e9c7323fdf38da918d9d6f20` e `ghcr.io/leofran2204/poker-web@sha256:ae00bfb872a589956b533907902f72903371c96d4f7779763fb013fe42a45ffe` assinadas e conferidas pelo `vps-release-attested.sh`, dois dumps, migrations 67 (sem migration nova), boot com um "Migrations applied" e sem `ERROR`, `.env` preservado, os 4 serviços saudáveis. Por fora: `/api/health` 200, `/api/lobby/tournaments` 200, `POST /api/tournament/register` sem login 401.

Mudança: a inscrição tardia senta o jogador na mesa viva com menos jogadores e o ator o coloca na mão seguinte; inscrição que falha restaura prêmio, contagem e totais do torneio. Antes da publicação, consulta só de leitura no banco de produção comparou o prêmio de cada torneio com buy-ins, rebuys, add-ons e garantido: 24 torneios, nenhum inflado (excesso 0). [Registro](DEVELOPMENT_LOG.md#2026-11-25--inscrição-tardia-passa-a-dar-assento).

Verificação local antes do push: `fmt`, `clippy --all-targets -D warnings` da API, suíte inteira da API com `--include-ignored` no banco com as 67 migrations (10 binários verdes, incluindo o teste novo `mtt_late_registration_seats_player_and_failed_one_keeps_prize`) e `documentation-sync --check`.

## Publicação: nota do quiz das aulas corrigida no servidor — 25/11/2026

Publicado `15700afca92461824054e12a4311e9493ed2d88e` às **15:21:03 UTC / 12:21 em São Paulo de 25/11** (ordem do proprietário nesta sessão: "depois q terminar pode publicar"), com Rust CI e Container Supply Chain verdes no commit, imagens `ghcr.io/leofran2204/poker-api@sha256:2636f6a098e8cba914447c91558d04095cfe45aa38d568329bb9a15bc5113f79` e `ghcr.io/leofran2204/poker-web@sha256:55586257568cef7f06a56d71a47cb1a2802a85fa950384b00cea357cff1d6fca` assinadas e conferidas pelo `vps-release-attested.sh`, dois backups, migrations 67 (sem migration nova), boot com um "Migrations applied" e sem `ERROR`, `.env` preservado, os 4 serviços saudáveis. Por fora: `/api/health` 200, `/api/lobby/tournaments` 200, `/curso/m0l1` 200, `POST /api/course/progress` sem login 401.

Mudança: `POST /api/course/progress` recebe as respostas e corrige pelo gabarito de `courseContent.json` embutido na API; nota pronta do navegador = 400; aula bloqueada pelo pré-requisito = 403. [Registro](DEVELOPMENT_LOG.md#2026-11-25--nota-do-quiz-das-aulas-corrigida-no-servidor-etapa-4).

Verificação local antes do push: `fmt`, `clippy --all-targets -D warnings` da API, suíte da API (126 da biblioteca, 3 deles novos do gabarito) e os 22 contratos de `api_tests` com banco (`poker_test_067`), incluindo o novo `test_contract_course_quiz_is_graded_by_the_server`; site com `tsc`, ESLint, Vitest (160) e build do Vite.

## Auditoria: mãos antigas com o wheel e a equity de Short Deck — 25/11/2026

Somente leitura no banco de produção (transação `READ ONLY` encerrada em `ROLLBACK`, com autorização do proprietário nesta sessão). Nenhum saldo, mão ou registro foi alterado.

- **Corte:** a correção (`2bd14a0e`) entrou em produção em 01/10/2026, 15:27:57 UTC. Antes dela há 6.369 mãos, de 07/08 a 10/09/2026; entre 10/09 e 23/11 não há nenhuma; depois do corte são 704 (23/11, bots da etapa 2).
- **Defeitos antigos:** (a) no wheel A-2-3-4-5 as cartas de desempate guardavam só o ás, então ele vencia qualquer sequência até a de rei alto e empatava com a de ás alto; (b) no baralho curto, A-6-7-8-9 desempatava como ás alto e vencia de 6-7-8-9-10 até 9-10-J-Q-K; (c) a equity do Loss Deflator usava 52 cartas e o ranking comum em todas as variantes. Até a migration 057, as mesas hoje chamadas `omaha` e `brazilian_pineapple` eram `short_deck_omaha` e `ultimate_pineapple`, com 36 cartas e o mesmo avaliador; nenhuma das 1.837 mesas dessas variantes tem carta de 2 a 5.
- **Limite do método:** o histórico não guarda cartas fechadas (migration 017, por privacidade), só a categoria da mão, os pagamentos e as cartas comunitárias. O defeito só muda o resultado quando dois jogadores chegam ao showdown com sequência (ou sequência do mesmo naipe) e a mesa tem ao menos 3 cartas do wheel. Empate entre duas sequências não vem do defeito nessas mãos: a única troca para empate (wheel contra sequência de ás alto) exige uma mesa que não aparece entre as candidatas.
- **Jogo Real:** 3 mãos antes do corte, todas em 29/08/2026, 19:10, com conta de teste e potes de R$ 0,50 a R$ 2,00; nenhuma mesa permitia o wheel. **Impacto em dinheiro real: zero.**
- **Loss Deflator:** nenhuma devolução antes do corte, em nenhuma variante; a equity errada não pagou nem deixou de cobrar nada. Devolução possivelmente negada: 9 mãos de Play Money com all-in perdido fora do Hold'em (3 Pineapple, 5 Omaha, 1 Short Deck), sem recálculo possível sem as cartas; pela regra atual só a de Short Deck seria elegível.
- **Play Money:** 154 mãos com duas ou mais sequências no showdown e mesa que forma o wheel (36 Pineapple, 91 Omaha, 25 Short Deck, 2 Hold'em; 113.638 em potes). Em 84 delas uma sequência perdeu para outra (21 Pineapple, 49 Omaha, 14 Short Deck; 20.684 em potes): é o teto de mãos com vencedor possivelmente errado. As 2 de Hold'em terminaram empatadas e não foram afetadas. Troca certa: 1 mão de Short Deck (`42032598`, 07/09, pote 100, mesa A-6-7-8-9 inteira; quem tinha o 10 perdeu para quem jogou a mesa).
- **Participantes:** `hand_participants` registra só 5 contas nessas 154 mãos (4 bots e 1 conta de Play Money); as demais mãos não têm registro de participante.
- **Conclusão:** nada a devolver nem a corrigir; não houve dinheiro real nem devolução envolvidos, e Play Money não tem valor de saque. O item sai do DASHBOARD.

## Publicação: quiz e vídeo das aulas contados no servidor — 25/11/2026

Publicado `99bd782163e5d2d106b8c57c96609cb29b0d9344` às **21:33:26 UTC / 18:33 em São Paulo de 25/11** (ordem do proprietário nesta sessão: "depois q o CI ficar verde pode publicar"), com Rust CI e Container Supply Chain verdes no commit, imagens `ghcr.io/leofran2204/poker-api@sha256:3b75807225bfe183289018b294352d9c135a0a3bb72835e39380be2eb948f161` e `ghcr.io/leofran2204/poker-web@sha256:426a9cb53ecb42b973d3b2a70b101c483dbf2c1011ffbe6571a7a4d0c42375ed` assinadas e conferidas pelo `vps-release-attested.sh`, dois backups, migrations 68 (nova: `068_course_attention.sql`), boot com um "Migrations applied" e sem `ERROR`, `.env` preservado, os 4 serviços saudáveis. Por fora: `/health` 200; `GET /api/course/lesson` e `POST /api/course/watch` sem login = 401; o pacote publicado das aulas (`course-*.js`) tem as 68 perguntas e nenhum `expectedIndex`.

Mudança: quiz só abre com 90% do vídeo contado pela API e as perguntas surpresa acertadas; nova tentativa 15 min depois de reprovar; gabarito só depois da aprovação e fora do pacote do site. [Registro](DEVELOPMENT_LOG.md#2026-11-25--aula-cumprida-de-verdade-quiz-e-vídeo-etapa-6a).

Verificação local antes do push: `fmt`, `clippy --all-targets -D warnings` da API, suíte da API (130 da biblioteca, 4 novos em `course_watch.rs`) e os 23 contratos de `api_tests` com banco (`poker_test_068`), incluindo o novo `test_contract_course_video_is_counted_by_the_server` e o refeito `test_contract_course_quiz_is_graded_by_the_server`; site com `tsc`, ESLint, Vitest (160) e build do Vite; `documentation-sync --check`. Não feita: conferência visual da página da aula com login (sessão sem navegador).

## Publicação: vídeo e texto das aulas só com conta — 25/11/2026

Publicado `053107a15803abce78ed79694fc2e8bbce0961ec` às **22:08:48 UTC / 19:08 em São Paulo de 25/11** (ordem do proprietário nesta sessão: "faça commit, push e deploy"), com Rust CI e Container Supply Chain verdes no commit, imagens `ghcr.io/leofran2204/poker-api@sha256:56ba9d51bab23baad3542c8157dd35fd69d211cf7b3698e9eacbc610954482e4` e `ghcr.io/leofran2204/poker-web@sha256:ae603ee38a44c8789090385fbbe58faa5c65fcce3072fbcce846e32fdface369` assinadas e conferidas pelo `vps-release-attested.sh`, dois backups, migrations 68 (sem migration nova), boot com um "Migrations applied" e sem `ERROR`, `.env` preservado, os 4 serviços saudáveis.

Conferido por fora, sem login: `/health` 200; vídeo e legenda de aula (`zt-academy-m0l4-v3.mp4`/`.vtt`) 401, também com cookie inválido; capa `.webp` 200; episódio de história e trailer do Brazilian Pineapple 200 (`video/mp4`); `/api/course/material` 401; pacote publicado das aulas sem texto, transcrição ou `expectedIndex` das 25 aulas de aluno e com o texto dos 3 episódios. Com login, conferido só localmente (contrato com banco: `/api/auth/check` 204 e material da m0l4; Caddy de teste com login falso: 200 com sessão); o login com conta de teste no site publicado foi barrado pelo classificador de permissões e fica para o proprietário conferir no navegador. [Registro](DEVELOPMENT_LOG.md#2026-11-25--vídeos-e-texto-das-aulas-só-com-conta).

Verificação local antes do push: `fmt`, `clippy --all-targets -D warnings` da API, suíte da API sem banco e os 23 contratos com banco (`poker_test_068`); site com `tsc`, ESLint, Vitest (161) e build do Vite; `caddy validate` nos Caddyfiles; `documentation-sync --check`.

## Publicação: prova final e certificado da Academy — 25/11/2026

Publicado `3b7e8e96272d16f58ebddadec9ea773ee5b5facd` às **00:04:53 UTC de 26/11 / 21:04 em São Paulo de 25/11** (ordem do proprietário nesta sessão: "pode publicar"), com Rust CI e Container Supply Chain verdes no commit, imagens `ghcr.io/leofran2204/poker-api@sha256:0105b20f641a7166c2581612fc5161bfebae47a4eee4582d059fe59d2d6c3848` e `ghcr.io/leofran2204/poker-web@sha256:aaf0ffb1bd0a301097676cd6af31d7f61c48121dae275081727b70b520873867` assinadas e conferidas pelo `vps-release-attested.sh`, dois backups, migrations 69 (nova: `069_course_certificate.sql`), boot com um "Migrations applied" e sem `ERROR`, `.env` preservado, os 4 serviços saudáveis.

Mudança: certificado emitido só pelo servidor, com as 25 aulas obrigatórias cumpridas e prova final de 20 perguntas sorteadas em 30 min (mínimo 70%, uma tentativa a cada 24 h); código público conferido em `/certificado/<código>`; Termos 3.3. [Registro](DEVELOPMENT_LOG.md#2026-11-25--certificado-e-prova-final-etapa-6b).

Conferido por fora, sem login: `/health` 200; `GET /api/course/certificate` e `POST /api/course/exam/start` = 401; `GET /api/certificate/verify` com código inexistente ou malformado = 404 ("Certificado não encontrado."); `/certificado/<código>` e `/curso/certificado` = 200; Termos 3.3 no pacote publicado; nenhum `expectedIndex` no pacote principal nem nos pacotes do curso. Não conferido: prova e certificado com login em produção (nenhum aluno com as 25 aulas; conta de teste não usada em produção) e aparência das páginas novas (sem navegador).

Verificação local antes do push: `fmt`, `clippy --all-targets -D warnings` da API, suíte da API (135 da biblioteca, 5 novos em `course_exam.rs`) e os 24 contratos de `api_tests` com banco (`poker_test_069`, novo `test_contract_course_certificate_needs_lessons_and_final_exam`); site com `tsc`, ESLint, Vitest (161) e build; `documentation-sync --check` e `git diff --check` limpos.

## Publicação: banco próprio da prova final — 25/11/2026

Publicado `3e6cd5e351744800aab17523fe5a667a92a6cde6` às **00:29:08 UTC de 26/11 / 21:29 em São Paulo de 25/11** (ordem do proprietário nesta sessão: "pode"), com Rust CI e Container Supply Chain verdes no commit, imagens `ghcr.io/leofran2204/poker-api@sha256:98173ee0bd520e60b53accea6544b19ea9f234f832c2daed2092cd7acdd51173` e `ghcr.io/leofran2204/poker-web@sha256:14e03f0bf6627130e033c4935dc0d8fbf0038d2bb72140f7ad53a995e029b4ac` assinadas e conferidas pelo `vps-release-attested.sh`, dois backups, migrations 69 (sem migration nova), boot com um "Migrations applied" e sem `ERROR`, `.env` preservado, os 4 serviços saudáveis.

Mudança: a prova final sorteia do banco próprio `API-Axum/data/course_exam_bank.json` (77 perguntas, fora do site, diferentes do quiz). [Registro](DEVELOPMENT_LOG.md#2026-11-25--banco-próprio-da-prova-final).

Conferido por fora, sem login: `/health` 200; `GET /api/course/certificate` 401; verificação de código inexistente 404; `/curso/certificado` 200; nenhum id nem enunciado do banco no pacote principal nem nos demais pacotes do site. Verificação local antes do push: `fmt`, `clippy --all-targets -D warnings`, suíte da API (136 da biblioteca, novo `banco_da_prova_cobre_as_aulas_sem_repetir_o_quiz`) e os 24 contratos de `api_tests` com banco (`poker_test_069`); `documentation-sync --check` e `git diff --check` limpos.

## Publicação: 28 aulas em Remotion, legenda opcional e Brazilian Pineapple no Jogo Real — 26/11/2026

Publicado `ca64a535154024cc82f38065aade99924f554c1a` às **14:59:31 UTC / 11:59 em São Paulo de 26/11** (ordens do proprietário nesta sessão: "pode executar tudo e depois publicar", "tá aprovado o jogo real de Brazilian Pineapple. faça a limpeza e depois pode fazer commit, push e deploy" e "pode entrar na VPS e publicar"). O commit inclui `30348ae7` (aulas) e `4a3ea00a` (legenda). Rust CI e Container Supply Chain estavam verdes nos três commits. Imagens `ghcr.io/leofran2204/poker-api@sha256:7182db998ec5a0e8367e4d4c863d0676377b7f3a291f47d3bf7ef3cc1facff98` e `ghcr.io/leofran2204/poker-web@sha256:c60ba4dc3779bda75473072155942b359964b09ced94415f8d9ef6cae58cf803`, assinadas e conferidas pelo `vps-release-attested.sh`. Dois backups, migrations 71 (novas: `070_course_watch_video.sql` e `071_pineapple_draw_real_money.sql`), boot com um "Migrations applied" e sem `ERROR`, `.env` preservado, os 4 serviços saudáveis.

Mudanças: as 28 aulas da Academy no Remotion; apresentação m0l1–m0l3 em `zt-intro-*`; `ep11`–`ep13` fora do site; legenda desligada por padrão; contagem de vídeo por versão (`course_watch.video_url`); mesa e torneio do Brazilian Pineapple no Jogo Real. Registros: [aulas](DEVELOPMENT_LOG.md#2026-11-26--todas-as-aulas-da-academy-em-remotion-etapas-2-a-5-do-plano-de-2511) e [Pineapple](DEVELOPMENT_LOG.md#2026-11-26--brazilian-pineapple-no-jogo-real).

Conferido em produção sem login: `/api/health` 200; `zt-intro-m0l1-v4.mp4` e `.vtt` 206 (abertos); `zt-academy-m1l1-v4.mp4` e `.vtt` 401 (pedem conta); `ep11-cartas-historia-v2.mp4` cai na página do app (arquivo removido); no pacote do site, o `<track>` de legendas sem `default`. No banco: mesa "Real · Brazilian Pineapple 0,50" `OPEN` em `real`, "Brazilian Pineapple — Torneio" `registering` em `real`, `course_watch` sem linhas; log da API sem `ERROR`. Verificação local antes do push: `render.py --check` (30 publicações), `test_narration.py`, Vitest 162/162, `tsc`, ESLint, build do Vite, `fmt`, `clippy --all-targets -D warnings`, 137 testes da biblioteca da API e os 25 contratos de `api_tests` com banco (`poker_test_071`, 71 migrations); `documentation-sync --check` e `git diff --check` limpos. Não conferido: reprodução dos vídeos e o player com login num navegador (sem navegador nesta sessão).

## Publicação: Node 26 no build do site — 26/11/2026

Publicado `195fb04cdcc22b8603f8a1b9fa48ba2c4d18a947` às **21:12:51 UTC / 18:12 em São Paulo de 26/11** (ordem do proprietário: "autorizo" commit, push e deploy). Rust CI e Container Supply Chain verdes. Imagens `ghcr.io/leofran2204/poker-api@sha256:5506e47c626a44e42f7a2a3e3c98fd34ab953623958008bea3ccb05c28a6ee9a` e `ghcr.io/leofran2204/poker-web@sha256:212bd18bd88dfedbdbb055683c5fc1c5a9db248ebd23e146ca716a4d269af9ba`, assinadas e conferidas pelo `vps-release-attested.sh`. Dois backups, migrations 71 (nenhuma nova), boot com um "Migrations applied" e sem `ERROR`, `.env` preservado, os 4 serviços saudáveis.

Mudança: estágio `builder` do site em `node:26-alpine` e `setup-node` 26 no CI ([registro](DEVELOPMENT_LOG.md#2026-11-26--node-26-no-build-do-site-e-no-ci)). Conferido em produção sem login: página inicial e `zt-intro-m0l1-v4.mp4` 206, `/api/health` 200, `zt-academy-m1l1-v4.mp4` 401 (pede conta).
