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

Carga 100 cenários / 1M WSS / fuzz frontend: [`FULL_VALIDATION.md`](FULL_VALIDATION.md), **só** com autorização explícita. Scripts: [`../scripts/README.md`](../scripts/README.md).

Frontend local: `npm run lint` (`tsc` + ESLint) e `npm test` (Vitest) em `Frontend-Web/`, com o Node do `AGENTS.md` — nunca Node 18 do PATH.

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
4. Fatos operacionais: JSON schema v2 + `documentation-sync --write` e `--check`. Prosa única só no arquivo dono (`AGENTS.md`).
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

- Alterações locais, sem commit, push ou deploy. STATUS continua descrevendo a demo publicada, migration 059; a migration local 060 não foi reescrita.
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
- API: `fmt` e `clippy --all-targets -D warnings` aprovados; 77 testes de biblioteca, 17 de router e 13 contratos de banco aprovados (12 em `api_tests`, um de estorno em `estrutura`). Contratos PostgreSQL executados explicitamente no banco local com migration 060; o novo teste cobre autorização administrativa, conciliação, rejeição de totais desatualizados, fechamento concorrente/idempotente, bônus de 35%, comissão base de 30%, déficit e separação entre pontos PM, comissão Real e saldo de jogo. O contrato de estorno é marcado como dependente de banco, sem retornar sucesso silencioso quando falta `DATABASE_URL`.
- Frontend: TypeScript, ESLint, 83 testes em 11 arquivos e build Vite aprovados (172 módulos; bundle inicial 411,43 kB antes de gzip). As dicas reutilizam a teoria canônica da Academy.
- Documentação: contrato operacional schema v3 substitui o split bruto 18/12/70 por programa direto sobre NGR mensal; 14 testes da ferramenta, `fmt`/`clippy`, geração e `--check` aprovados em 22 documentos. A migração do schema rejeita o campo antigo e bases de comissão incompatíveis.
- Inventário: JSONs válidos, sem colisão de migrations, sem links locais quebrados nos documentos alterados; nenhum candidato de publicação acima de 50 MiB ou correspondência nos padrões de credenciais examinados. Isso não substitui auditoria externa de segurança.
- Migration 060 local conferida por SHA-384; a VPS estava em 059 antes da publicação. Evidência de deploy será registrada após conferir containers, migração e conteúdo servido.
- O primeiro CI identificou advisories novos em `brace-expansion`; atualizadas somente as duas dependências transitivas para 1.1.21 e 5.0.12. `npm ci` e `npm audit` passaram com zero vulnerabilidades, assim como TypeScript, ESLint, 83 testes e build após a correção.
- Reproduzida a falha do contrato de depósito: a conta de teste não possuía KYC. O teste agora exige 403 sem criar cobrança, verifica a conta sintética e então confere crédito e idempotência. Os quatro contratos de carteira passaram; provedor mock e chave sintética são inicializados antes das requisições paralelas. O CI também executa explicitamente o contrato de estorno do agente e interrompe migrations quando o PostgreSQL relata erro.
- O gate estrito expôs o autocommit no executor antigo do CI: a migration 026 perdia sua tabela `ON COMMIT DROP` antes do uso. Reprodução em banco isolado confirmou o erro. As 60 migrations passaram em banco vazio com uma transação por arquivo (`psql --single-transaction`), alinhado ao sqlx. Preservados os arquivos/checksums das migrations já aplicadas.

## Onde não procurar qualidade

- Aprendizado / protocolo Mark → `guia_aprendizado.md`
- OKR, marketing, chaos, “plano Elon” → `historico/QUALITY_v5.4.md` (arquivo, não vigente)
- Catálogo e ciclo → STATUS
- Fases e backlog → DASHBOARD

<!-- DOCUMENTATION_SYNC:START -->
> **S26** (2026-10-01) — demo `zerotiltpoker.net` · sem certificação de produção · PIX automático ligado (DePix reconciliado).
> Fatos (catálogo, carteiras, limites): [`STATUS_OPERACIONAL.md`](STATUS_OPERACIONAL.md).
<!-- DOCUMENTATION_SYNC:END -->
