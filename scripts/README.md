# scripts/

Automação operacional do monorepo.

Auditoria frontend: `npm run audit:security` em `Frontend-Web/` executa `npm audit --audit-level=high`, sem exceções. O script temporário de exceção e seus testes foram retirados na revisão de 05–06/10/2026 após a remoção de `braces` na migração para Tailwind 4. Histórico em [QUALITY](../Documentacao/QUALITY.md#exceção-temporária-de-build--05102026-a-18102026).

## Ambiente local e permissões do Codex

Abra `C:\Users\leofr\Projetos\Poker_Project` como pasta do projeto. Informar `workdir` em um comando não muda a pasta nem o perfil de permissões da sessão aberta em `C:\`.

Diagnóstico de 28/09/2026: o projeto já consta como confiável e `[windows].sandbox = "elevated"` está configurado. O log registrou configuração do sandbox sem erros; as ACLs consultadas permitem modificação ao grupo `CodexSandboxUsers`. A sessão, porém, ainda apresentou `EPERM` na gravação pelo Node e `Wsl/Service/E_ACCESSDENIED`. A inspeção não identificou uma falha única que explique todos os bloqueios. Não alterar ACLs em massa nem afirmar que uma reinstalação resolveu sem repetir as verificações.

O perfil ativo é escolhido no controle de permissões do aplicativo (na CLI, `/permissions`). **Aprovar por mim**, quando disponível, mantém o sandbox e encaminha pedidos elegíveis à revisão automática; recusas ainda podem ocorrer. **Acesso completo** remove os limites do sandbox sobre arquivos e rede, inclusive fora do projeto. Apenas definir `approval_policy = "never"` não libera recursos bloqueados. Confira o perfil efetivamente recebido pela sessão antes de declarar uma mudança aplicada.

Para ferramentas Linux, executar o próprio Codex no WSL é uma alternativa suportada. Chamar `wsl.exe` de dentro do sandbox Windows é um caminho diferente. Não mover o repositório nem criar outra cópia para contornar uma recusa.

Referências oficiais: [sandbox e aprovações](https://learn.chatgpt.com/docs/sandboxing), [Windows](https://learn.chatgpt.com/docs/windows/windows-sandbox) e [WSL](https://learn.chatgpt.com/docs/windows/wsl).

Após o usuário selecionar acesso completo na CLI em 28/09/2026, a sessão recebeu `danger-full-access` e `approval_policy=never`. Gravação pelo Node, WSL, testes Rust e renderização voltaram a funcionar. A mudança vale para o perfil ativo; não é garantia contra erros de rede, aplicação ou sessões futuras com outro perfil. Nenhuma configuração global ou ACL foi alterada pelo agente.

## Prévia isolada da Academy

Na raiz do repositório, API de estudo sem banco/carteira:

```powershell
wsl.exe -d Ubuntu -- bash -lc 'cd /mnt/c/Users/leofr/Projetos/Poker_Project/API-Axum && CARGO_TARGET_DIR=$HOME/poker-build/api-target cargo run --locked --example academy_preview'
```

Em outro terminal, dentro de `Frontend-Web`:

```powershell
$env:POKER_DEV_API_URL = 'http://127.0.0.1:3188'
& 'C:\Users\leofr\AppData\Local\hermes\node\node.exe' node_modules/vite/bin/vite.js --host 127.0.0.1 --port 5181 --strictPort
```

Abrir `http://127.0.0.1:5181/curso/mesa/pineapple`. O exemplo fornece apenas `POST /api/academy/play`; outras rotas da API não existem nessa prévia. Ctrl+C encerra cada processo. Remover `POKER_DEV_API_URL` para voltar ao backend completo.

Regravação a partir do conteúdo canônico e das dependências de `ZeroTiltCurso/editorial/requirements.txt`:

```powershell
& 'C:\Users\leofr\AppData\Local\hermes\hermes-agent\venv\Scripts\python.exe' ZeroTiltCurso/editorial/render.py --academy all
# Ou uma aula: --academy m6l3
```

O comando publica os arquivos somente no checkout local. Reutiliza cache de narração validado, confere a sequência completa de palavras, gera legendas/capítulos/transcrição e compara o hash da aula antes de atualizar o JSON. Um fluxo de voz truncado é repetido; após quatro falhas, o lote para. `courseAudit.test.ts` rejeita vídeo técnico anunciado como atual com hash diferente do texto, quiz, fontes ou cenários. Desde 09/11/2026 nenhuma aula usa `prior_rules`. As três aulas do Brazilian Pineapple (m5l3–m5l5) têm vídeo do renderizador em Remotion (seção abaixo); `render.py` não as regrava (`MOTION_LESSONS`), e `--academy all` as ignora.

`PRONUNCIATIONS`, em `render.py`, aplica guias fonéticas apenas à síntese (por exemplo, `Pineapple` → `painépou`). A grafia original continua nas cenas, legendas e transcrições. Os tempos reais da voz são projetados de volta às palavras originais, inclusive quando um termo exige duas palavras faladas, como `all-in`. Voz, velocidade e texto fonético entram no hash do cache; `pronunciationVersion` e `narrationHash` vinculam o manifesto à produção. Alterar uma guia invalida as gravações que usam o termo. O cache de síntese usa esse hash. As URLs dos players usam `mediaRevision`, derivada dos hashes do MP4, VTT e capa, para invalidar também alterações somente visuais.

Regressões da síntese e da projeção temporal: `python -m unittest discover -s ZeroTiltCurso/editorial -p test_narration.py -v`. Para os três filmes de história, usar `--episode all`; para as aulas técnicas, `--academy all` (as três do Brazilian Pineapple saem de `motion/produce.py`). Executar os dois lotes em sequência, pois atualizam o mesmo `courseContent.json`.

O job frontend do Rust CI executa esses testes e `render.py --check` em ambiente Python isolado. Mudanças em `ZeroTiltCurso/editorial/` também disparam o workflow; a comparação com `narrationHash` bloqueia gravações desatualizadas em relação ao glossário.

Validação audiovisual independente:

```powershell
& 'C:\Users\leofr\AppData\Local\hermes\hermes-agent\venv\Scripts\python.exe' ZeroTiltCurso/editorial/render.py --check
# Acrescentar --decode para decodificar integralmente todos os MP4;
# --only <aula> limita a auditoria a uma publicação (ex.: --only m6l3).
```

O verificador exige todos os vídeos ativos, confere hashes, duração, capítulos, poster 1280×720, limites temporais de cada legenda (inclusive as de duas linhas) e correspondência completa entre VTT e transcrição. Para as aulas e os filmes em Remotion, confere também o hash do roteiro (`scriptHash`), a narração recalculada do roteiro, o texto atual da aula (`contentHash`) e a regra `pineapple_draw_v1`. A decodificação adicional verifica os fluxos de áudio/vídeo e produz evidência em `artifacts/academy-audit/media.json`. Interrupção de rede não invalida vídeos já concluídos: repetir `--academy all` retoma as aulas ainda pendentes ou alteradas.

O filme do Brazilian Pineapple, o institucional anterior (`home`) e o desenho `pineapple.py` foram retirados em 09/11/2026 com a troca de modalidade; o histórico git preserva as fontes. A função `cue_time` passou para `render.py`. Arquivos novos de legenda/transcrição têm LF fixo para manter seus hashes em Windows/Linux.

### Produção em Remotion (Brazilian Pineapple desde 10/11/2026)

`ZeroTiltCurso/editorial/motion/` é o renderizador novo: vídeo programado em React (Remotion 4.0.538), 1920×1080 a 30 quadros, com animações sincronizadas pelo tempo de cada palavra da narração. Roteiros em `src/data/*.json` (fala por extenso; valores em R$/BB só na tela); cenas em `src/videos/*.tsx`; composições em `src/Root.tsx`. Publicados com ele: o filme de regras (`pineapple-draw-rules.json`), o trailer (`pineapple-draw-trailer.json`) e as aulas m5l3–m5l5 (`academy-m5l3.json`… `academy-m5l5.json`). A aula M2L2 (`m2l2.json`) continua só como piloto de formato, fora do site. Os cinco roteiros do Brazilian Pineapple são gerados por `tools/build_scripts.py` a partir de uma mão-modelo (quatro jogadores, blinds R$ 0,50/R$ 1, ante do big blind), que calcula stacks, apostas e pote de cada cena; edite o gerador, não os JSON. A cena `hand` (`src/videos/HandTable.tsx`) desenha a mesa oval com quatro jogadores e anima os eventos do roteiro: apostas saem da pilha do jogador, são recolhidas para o pote, cartas são trocadas ou descartadas e o pote vai para o vencedor. Pronúncia aprovada pelo proprietário em 11/11/2026 (`MOTION_PRONUNCIATION_VERSION`): com a voz do Azure, os termos em inglês do glossário de `render.py` são lidos em inglês dentro de `<lang xml:lang="en-US">` (`render.speech_ssml`); o motor Edge não aceita esse SSML e fica só como reserva. Vocabulário: “street” (não “rua”) e “oponente” (não “rival”).

Os cinco roteiros são conferidos no motor por `Motor-Rust/tests/pineapple_draw_film_examples.rs` (também no Rust CI): cartas sem repetição; a mão-modelo inteira jogada no motor na mesma ordem e com os mesmos valores (troca só depois das apostas do flop e do turn, descarte no river antes da última rodada, stacks e pote de cada cena); mãos destacadas, descartes e vencedores pelo avaliador; e os máximos da cena do limite do pote.

```powershell
cd ZeroTiltCurso/editorial/motion
# Node 24 do harness + npm do sistema; cache em C:\tmp\poker-npm-cache
node 'C:\Program Files\nodejs\node_modules\npm\bin\npm-cli.js' ci --cache C:\tmp\poker-npm-cache
python narrate.py --engine azure src/data/pineapple-draw-rules.json src/data/pineapple-draw-trailer.json src/data/academy-m5l3.json src/data/academy-m5l4.json src/data/academy-m5l5.json
python sound.py                                                         # efeitos e trilha originais
python produce.py PineappleDrawRules src/data/pineapple-draw-rules.json --poster 84.62 --publish
python produce.py PineappleDrawTrailer src/data/pineapple-draw-trailer.json --poster 5.58 --publish
python produce.py AcademyM5L3 src/data/academy-m5l3.json --poster 67.32 --publish   # idem M5L4 e M5L5
```

Voz: `pt-BR-AntonioNeural` (Microsoft, neural) com as guias de pronúncia aproximadas de `render.py`; o Antonio não é multilíngue, então os termos em inglês saem aproximados (amostra 8 da comparação, escolhida pelo dono). `narrate.py --engine azure` usa o Azure Speech no nível gratuito F0 (recurso `zt-academy-speech`, grupo `rg-zerotilt-academy`, região eastus; chave só na variável de usuário `AZURE_SPEECH_KEY`, nunca em arquivo ou log). O script mantém um contador mensal em `public/narration/azure-usage.json` e recusa passar de 400 mil caracteres no mês (cota gratuita: 500 mil). Fala idêntica já sintetizada em outro vídeo é copiada, sem gastar cota. Atenção: a assinatura de avaliação do Azure é desativada após 30 dias se não for convertida para pagamento conforme o uso, e uso acima das quantidades gratuitas é cobrado; não há garantia documentada de que o F0 só bloqueie. Sem chave, `--engine edge` usa o Edge TTS (edge-tts 7.3.1), com a mesma voz e sem marcação de idioma. O cache de cada cena considera motor, voz, velocidade, texto falado e versão das guias.

`narrate.py`, `sound.py` e `produce.py` rodam no Python do ambiente editorial (edge-tts 7.3.1, azure-cognitiveservices-speech 1.44.0, numpy, Pillow). `produce.py` renderiza (H.264 CRF 18), normaliza o áudio para −16 LUFS / −1,5 dBTP e gera VTT (até duas linhas de 42 caracteres, sem sobreposição), capítulos, transcrição e capa WebP 1280×720 no mestre em `artifacts/video-pilot/<id>/` (fora do git). Com `--publish`, grava a versão do site em `Frontend-Web/public/videos/` (1080p, H.264 CRF 26 ajustado para gráfico chapado (testado contra CRF 23 e 28 em recorte 1:1), GOP de 2 s, AAC 128 kbps, a partir do render bruto), o manifesto e a transcrição em `ZeroTiltCurso/editorial/` e o bloco de vídeo: aulas em `courseContent.json` (`renderer: "remotion"`, `contentHash` do texto da aula); filme e trailer em `Frontend-Web/src/data/pineappleDrawFilms.json`, lidos por `PineappleDrawFilm.tsx` (home e Academy). Aula alterada durante o render cancela a publicação. `--reuse-render` refaz só o pós-processamento. Cenas de troca recebem `padAfter` calculado sobre os tempos da narração para a animação terminar antes do corte. A página local `artifacts/video-pilot/index.html` reúne os cinco vídeos. Narração, efeitos, tempos e renders intermediários estão no `.gitignore`.

Licença: o Remotion é gratuito para pessoas físicas e empresas com até 3 pessoas; acima disso exige licença da empresa. As fontes (Fraunces, Manrope) são OFL; trilha e efeitos são sintetizados no próprio projeto.

## Canônicos (usar)

> **Guarda de rede (Termos 2.3, migration 061):** a API recusa uma segunda conta na mesma mesa cash quando ela usa o mesmo IP público ou a mesma sub-rede de quem já está sentado. Smokes que sentam várias contas da mesma máquina na mesma mesa da demo (`live-e2e-*`, `full-catalog-100-vps.mjs`, `live-sim-full-ritual.mjs`) só passam com o IP de saída listado em `SEAT_IP_GUARD_EXEMPT_IPS` no `.env` da VPS durante a janela autorizada, e removido depois. No laboratório local e na `full-validation`, os endereços de loopback e de rede privada já ficam fora da comparação.

| Script | Uso |
|--------|-----|
| `live-e2e-ten-users.mjs` | Smoke demo: 10 users / 100 hands + settlement assinado (`ALLOW_TEMP_MAIL=true`) |
| `live-e2e-seeded-catalog.mjs` | Smoke mesa a mesa (Real ou Play): login seed → join → ≥1 mão → leave; torneios |
| `live-e2e-real-catalog.mjs` | Variante com Mail.tm + crédito admin opcional (`ADMIN_TOKEN`) |
| `live-sim-full-ritual.mjs` | Ritual Play Money: 1 jogador/e-mail por assento + 2 reservas/mesa; `HANDS_PER_TABLE` (default 2000); Mail.tm (`ALLOW_TEMP_MAIL=true`) |
| `clear-zombie-play-seats.sql` | Ops: cash-out de assentos Play Money `ACTIVE` órfãos (fallback; o actor S20f faz isso sozinho) |
| `full-validation.ps1` / `.sh` / `.py` | Campanha isolada por situações, até 60 minutos após compilação e dois trabalhadores; método/resultado em Documentacao/FULL_VALIDATION.md |
| `full-validation-network.mjs` | Cliente interno HTTPS/WSS local; não usar contra a demo |
| `test_full_validation.py` | Contratos do executor: contagem, interrupção e isolamento; sem carga de poker |
| `deploy.ps1` / `deploy.sh` | Deploy assistido |
| `verify-public-https.sh` | Checagem HTTPS/Caddy público |
| `vps-redeploy-frontend.sh` | Redeploy na VPS (`REBUILD_API=1` para API+migration); exige fast-forward, containers saudáveis e API por HTTPS |
| `vps-release-attested.sh` | Publicação na VPS com as imagens do CI, sem build: `cosign verify` (supply-chain.yml/master/commit) antes de qualquer troca, janela ociosa, dois dumps, rollback automático. `--verify-only` só confere |
| `coverage.ps1` / `.sh` | Cobertura (quando autorizado) |
| `ws-probe.mjs` | Sonda WS local pontual (`node scripts/ws-probe.mjs <email>`) |
| `full-catalog-100.mjs` | Catálogo completo local: 100 contas (72 cash + 28 MTT) |
| `full-catalog-100-vps.mjs` | Catálogo completo contra a VPS (sessão longa real) |
| `estrutura-rede-e2e.mjs` / `.py` | Histórico do modelo 18/12; não valida o programa Agente ZT v2 e não é gate da release atual |
| `estrutura-auditoria.sql` | Histórico do modelo 18/12: gates da Fase D sobre `estrutura_ledger` (`program_version = 1`); não audita o Agente ZT v2 |
| `estrutura-ws-hold.mjs` | Histórico da Fase D: mantém contas com WebSocket aberto em check/fold durante a simulação 18/12 |
| `estrutura-bots-jogar.mjs` | Bots WS da Estrutura (fase VP + allin-fest) |
| `setup-test-users.mjs` | Cria contas Play Money de teste no Docker local e imprime `BOT_USERS` para `strategy-bots.mjs` |
| `strategy-bots.mjs` | Bots externos via WebSocket que decidem com `bot/dist` (`decideAction`); exige `cd bot && npx tsc` antes |
| `stop_bots.py` | Ops pontual na VPS: para os bots da casa numa mesa pelo endpoint admin, lendo `JWT_SECRET` do `.env` local sem imprimi-lo |
| `wipe-*.sql` / `wipe-emalupe-users.ps1` | Limpeza pontual de contas sintéticas/teste (ops, com backup antes) |
| `wipe-all-test-accounts.sql` | Limpeza geral de contas de teste (ops, com backup antes) |

## Stress do motor (não são scripts shell)

| Teste | Uso |
|-------|-----|
| `Motor-Rust/tests/cash_catalog_10k_hands.rs` | Nome preservado; cinco configurações, lotes de 1.000, parada por cobertura/teto e replay; acionado pelo executor |
| `Motor-Rust/tests/tournament_to_champion.rs` | Cinco eventos do banco isolado, fichas e premiação separadas; sem reservas fora da late registration |
| `Motor-Rust/tests/short_deck_massive.rs` | Regras SD + 1M evals + 100k mãos 6-max |
| `cargo test --features massive-tests …` | Fuzz/fairness/stress gated |

A campanha atual é acionada por ./scripts/full-validation.ps1 -Approved no Windows (WSL Ubuntu) ou FULL_VALIDATION_APPROVED=1 bash scripts/full-validation.sh all no Linux. Os testes de lotes são ignorados na rotina e exigem variáveis/artefatos preparados pelo executor.

**Resultado atual:** campanha ainda incompleta. A nova autorização de 03/10 comprovou 25.000 mãos cash, cinco MTTs/309 mãos, equity, 21 contratos de banco/Redis, atores 1/5/20 e HTTPS/WSS de uma mesa. Pendentes última fixture Pineapple, rede de cinco/vinte mesas e reinício. O cliente agora exige Node Linux no PATH ou em `artifacts/full-validation/runtime/node`: Windows→WSL agrupava os IPs e atingia o rate limit. A última tentativa expirou por deadline após discrepância de relógios; nova carga exige novo orçamento autorizado. [Método, evidências e reprodução](../Documentacao/FULL_VALIDATION.md). Os scripts históricos de demo listados acima não fazem parte desta campanha local.

## Seeded catalog e2e

```bash
# Contas e2ecat01/02 com saldo Real (criar via SQL na VPS se necessário)
MODE=real HANDS_PER_TABLE=1 node scripts/live-e2e-seeded-catalog.mjs
MODE=play HANDS_PER_TABLE=1 node scripts/live-e2e-seeded-catalog.mjs
```

UI canônica: **`Frontend-Web/`** (`npm run build` / Docker). O antigo `Frontend-Dioxus/` foi removido do monorepo.

## Dependência `ws`

Os scripts `.mjs` que usam WebSocket resolvem `ws` em `scripts/node_modules/` (declarado em
`scripts/package.json`). Rode sempre a partir da raiz: `node scripts/<nome>.mjs`.

## DePix Sandbox local

`install-depix-local-secrets.ps1` solicita a chave `sk_test_` e o webhook secret sem ecoá-los, valida a chave em `https://api.depixapp.com/api/me` e grava somente em `Infraestrutura-Docker/.env`, ignorado pelo Git. Use `-AllowedDepositorId <UUID>` para limitar quem pode criar/simular cobranças. Não use esse instalador na VPS pública.
`install-depix-vps-live-secrets.ps1` valida `sk_live_` em `/api/me` (conta verificada). Escopos `merchant_read`/`merchant_write` e aprovação da API são confirmações explícitas. Allow-list de UUIDs é **opcional**: omitida, todo usuário logado gera PIX (o cadastro por convite é o portão). Use primeiro sem `-Apply`. A instalação na VPS exige `-ConfirmProviderApproval -ConfirmRegulatoryAuthorization -Apply`. Segredos nunca vão por pipe/chat.

### Perfil focado Pineapple

`scripts/full-validation.ps1 -Phase pineapple -Minutes 15 -Approved` executa a campanha local autorizada com dois trabalhadores, banco/rede exclusivos e deadline de 15 minutos após compilação. Equivalente WSL: `FULL_VALIDATION_APPROVED=1 FULL_VALIDATION_MINUTES=15 bash scripts/full-validation.sh pineapple`. Abrange fixtures, cash/MTT Pineapple, contrato motor/atores/WS, HTTPS/WSS 5/20 mesas e reinício. Resultados e lacunas pertencem a [FULL_VALIDATION.md](../Documentacao/FULL_VALIDATION.md). Não reutilizar resultados Pineapple antigos como evidência da regra híbrida.

O perfil `pineapple-cash` permite complementar somente as fixtures e o lote cash, sem repetir MTT/rede. Cada continuação deve receber teto reduzido conforme o orçamento restante; um resultado parcial não substitui as evidências das outras camadas.

A revisão visual v2 usa `editorial/timeline.py`: tempos de entrada/revelação/pausa, elegibilidade para extras e contabilidade animada em centavos. `test_narration.py` também verifica essas invariantes e a revisão de mídia. Renderer Academy v5/editorial v4; os três `sourceSnapshot` históricos permanecem imutáveis.
