# Zero Tilt Poker — contrato para qualquer LLM

Se este arquivo **não** estiver no seu contexto, leia `AGENTS.md` na raiz **agora**, depois `Documentacao/STATUS_OPERACIONAL.md`. Não edite código nem docs antes disso.

Este é o único texto de regras do projeto. Grok, OpenCode, Codex, Cursor e Copilot auto-carregam este arquivo. Claude Code usa `CLAUDE.md` (`@AGENTS.md`). Gemini CLI usa `GEMINI.md` (ponteiro). Hermes piloto usa `.hermes.md` (só observar — **não** injeta este arquivo). Não copie README nem catálogo para cá.

Caminho do repositório: `c:/Users/leofr/Projetos/Poker_Project` (nunca OneDrive).

## Mapa de verdades (um assunto → um arquivo)

| Assunto | Dono | Como alterar |
|---|---|---|
| Ciclo, catálogo cash/MTT, carteiras, PIX, presença, certificação, flags | `Documentacao/STATUS_OPERACIONAL.json` (máquina) → `STATUS_OPERACIONAL.md` (leitura, gerado) | Editar o JSON schema v2 + `cargo run --bin documentation-sync -- --write` e `--check`. Campo extra no JSON é erro. `certified: true` e PIX automático em production são recusados. |
| Regras de pôquer e dinheiro (rake, fee, 18/12, PM×Real, deflator) | `Documentacao/BUSINESS_RULES.md` | Editar só a regra. Stakes vigentes estão no STATUS. |
| Arquitetura motor/stack/pastas | `Arquitetura-Motor/ARQUITETURA_MOTOR.md` | |
| Contratos REST/WS/admin | `Documentacao/ARQUITETURA_E_APIS.md` | |
| Agora, backlog, DoD, fases | `Documentacao/DASHBOARD.md` | |
| História cronológica | `Documentacao/DEVELOPMENT_LOG.md` | Append; não prevalece sobre STATUS. |
| Gates de CI / qualidade | `Documentacao/QUALITY.md` | |
| Como aprender o código | `Documentacao/guia_aprendizado.md` | |
| Carga massiva autorizada | `Documentacao/FULL_VALIDATION.md` + `scripts/README.md` | |
| Convite demo | `Documentacao/DEMO_AMIGOS.md` | Tabelas cash/MTT geradas do JSON. |
| Pitch amigos / due diligence | `RELATORIO_AMIGOS_SOCIO.md` / `RELATORIO_PARCEIROS_ZEROTILT.md` | Números = STATUS, não copiar stakes. |
| GTM rede 2 níveis | `Documentacao/PLANO_GO_TO_MARKET_REDE_2_NIVEIS.md` | |
| Curso de estratégia (produto) | `Documentacao/CURSO_ESTRATEGIA_POKER.md` | |
| Curso em vídeo (produto) | `ZeroTiltCurso/epNN-*/` (plan.md + script.py + narracao.mp3 + final) | Intermediários de render ignorados no `.gitignore`. |
| Exemplos Loss Deflator / JWT | `LOSS_DEFLATOR_EXEMPLOS.md` / `SECURITY_UPGRADES_EXEMPLOS.md` | |
| Deploy | `Infraestrutura-Docker/DEPLOY_HETZNER.md`, `DEPLOY_HOME_CLOUDFLARE.md`, índice `DEPLOYMENT_VALIDATION.md` | |
| UI Full Tilt | `Frontend-Web/README.md` | Catálogo = STATUS. |
| Legal | `TERMOS_DE_USO_E_SERVICO.md`, `PRIVACIDADE_DOMINIO.md` | Não misturar com ops. |
| Enciclopédia antiga / metas Fase 2 / cronograma detalhado | `Documentacao/historico/` | Snapshot. Não é vigente. |

Índice humano: `Documentacao/README.md`. Scripts: `scripts/README.md`.

## Não criar arquivo novo sem dono existente

Antes de adicionar `.md`, skill, agente opencode, script de ops ou “guia”:

1. Ler este mapa e os índices.
2. Se já há arquivo compatível, **editar ou estender esse**.
3. Arquivo novo só com justificativa explícita (assunto que nenhum dono cobre) e um único propósito.

Proibido: segundo STATUS, segundo QUALITY, segundo cronograma, segundo README de stack, segundo AGENTS.

Ganchos de descoberta (não são contrato): `CLAUDE.md` (`@AGENTS.md`), `GEMINI.md`, `.github/copilot-instructions.md`. Não copiar este arquivo para lá. `.hermes.md` é o piloto Hermes (só observar) e **substitui** este arquivo nessa ferramenta.

## Documentação — o que sincronizar

- Fato operacional mudou → JSON + `documentation-sync --write` + `--check`.
- Prosa única mudou → só o arquivo dono.
- **Não** reescrever todos os Markdowns da pasta a cada mudança. Isso gerou wiki no JSON e blockquote idêntico em 21 arquivos.

CI: `cargo run --locked --bin documentation-sync -- --check`.

## Limites do produto

- Sem certificação de produção. Sem PIX automático em production. Sem “Launch Ready”.
- Dinheiro em `u64` centavos inteiros. Ordem: potes → rake → Loss Deflator **só** sobre o líquido → pagamentos.
- Uma mesa = um processo (`TableActor` / `TournamentActor`). Settlement HMAC.
- Stack v4.0: Rust (motor + API) + TypeScript/React/Vite/Tailwind (`Frontend-Web/`). Dioxus é histórico git.

## Lei do repositório único (vale para qualquer LLM e qualquer harness)

Todo entregável mora dentro de `c:/Users/leofr/Projetos/Poker_Project`. Nada de
arquivo final em `Videos/`, `Desktop/`, `/tmp`, OneDrive ou pasta alheia: o que
nasce fora, move-se para dentro antes de concluir, e o original fora é apagado
após conferência (`diff` limpo). Intermediários de render/geração entram no
`.gitignore`; fontes + finais entram no git. Vale para vídeos (`ZeroTiltCurso/`),
scripts, relatórios e qualquer artefato pedido pelo dono.

## Git e publicação

Trabalho local (editar, testar, “prosseguir”) **não** autoriza `git commit`, `git push` nem deploy. Cada um exige ordem explícita e independente. Pedir commit não autoriza push.

## Migrations (disciplina anti-colisão, sessões paralelas)

- Antes de criar `API-Axum/migrations/NNN_*.sql`: `git pull` e conferir o maior NNN nos arquivos **e** em `_sqlx_migrations` (local + VPS). Número reutilizado ou migration aplicada e depois editada = boot travado com `VersionMismatch` (o sqlx confere checksum).
- Novas migrations: idempotentes (`IF NOT EXISTS`, `UPDATE` sem pré-condição destrutiva); mudança de catálogo com `INSERT` em `audit_logs`.
- Nunca commitar migration de outra sessão junto; conferir `git status` antes de `git add`.
- Pós-deploy: `SELECT version FROM _sqlx_migrations` + `docker logs poker_api` (“Migrations applied”).

## Ambiente Windows (esta máquina)

1. **Rust/Cargo:** WSL2 Ubuntu, projeto em `/mnt/c/Users/leofr/Projetos/Poker_Project`, `CARGO_TARGET_DIR` no disco Linux (ex. `$HOME/poker-build/root-target`, `api-target`, `motor-target`). Não gerar `target` em `/mnt/c`. Windows nativo só se o WSL estiver indisponível.
2. **Frontend:** não confiar em `node`/`npm` do PATH (pode ser Node 18). Usar o Node empacotado do harness, `Frontend-Web/package-lock.json`, cache `C:\tmp\poker-npm-cache`. Sem `pnpm` nesse diretório.
3. **Codex:** se `apply_patch` falhar com `helper_unknown_error` em arquivo existente, usar o contorno PowerShell UTF-8 sem BOM descrito no histórico deste contrato (criar script temporário que se autoexclui). Outros harnesses usam as ferramentas deles.
4. Sandbox `E_ACCESSDENIED` em WSL/git/SSH: repetir o **mesmo** comando com elevação estreita; não trocar a estratégia.

Antes de concluir entrega local: `git diff --check`; frontend `tsc` + Vite com Node empacotado se mexeu em `Frontend-Web`; Rust `fmt`/`clippy -D warnings`/testes relevantes no WSL.

## Papéis (só se o usuário pedir)

Fora disso, engenharia padrão.

1. Arquitetura — `ARQUITETURA_MOTOR.md` + `BUSINESS_RULES.md`; citar `arquivo:linha`.
2. Desenvolvimento — este arquivo (WSL, Node, rake→deflator, docs).
3. Segurança — centavos, HMAC, sem segredos em logs.
4. Negócio — `STATUS_OPERACIONAL.md` + `BUSINESS_RULES.md`. Nunca prometer o que o código rejeita.
5. Gestão — `DASHBOARD.md`; local ≠ commit ≠ push ≠ deploy.
6. Marketing de rede — somente `PLANO_GO_TO_MARKET_REDE_2_NIVEIS.md`.
7. Atendimento — `DEMO_AMIGOS.md` + STATUS; PT-BR; nunca pedir senha/código.

Opencode: `.opencode/agents/` roteia para `poker-*`. Hermes observa via `.hermes.md` (não altera código).
