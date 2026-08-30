# Diretrizes do Projeto Poker_Project

- **Caminho Principal do Projeto**: O caminho oficial para arquivos e comandos do projeto é sempre `c:/Users/leofr/Projetos/Poker_Project` (NUNCA utilizar o diretório do OneDrive).
- **Compilação e Testes Rust via WSL2**: Compilações, verificações e testes dos módulos Rust (`Motor-Rust`, `API-Axum` e demais crates) devem ser executados prioritariamente na distribuição `Ubuntu` do WSL2. Dentro do WSL, usar o projeto em `/mnt/c/Users/leofr/Projetos/Poker_Project` e definir `CARGO_TARGET_DIR` em disco persistente do Linux (ex.: `$HOME/poker-build/motor-target`), evitando gerar artefatos no filesystem compartilhado (`/mnt/c/.../target`) e eliminando a dependência de `dlltool.exe`/`cl.exe` do Windows. Evitar `/tmp` quando for tmpfs com pouco espaço. A compilação nativa no Windows deve ser usada somente como alternativa quando o WSL estiver indisponível.
- **Ordem Financeira: Rake antes do Loss Deflator**: O Loss Deflator deve ser calculado e aplicado **somente após** a retirada do rake do main pot e de todos os side pots. A base do Loss Deflator é sempre o valor líquido dos potes pós-rake; ele nunca pode ser calculado sobre o valor bruto pré-rake. A ordem obrigatória é: calcular potes → retirar rake → aplicar Loss Deflator sobre os potes líquidos → concluir os pagamentos.
- **Sincronização Obrigatória de Documentação**: Sempre que qualquer arquivo da pasta `Documentacao/` ou de acompanhamento for modificado/atualizado, é **OBRIGATÓRIO** atualizar e sincronizar **TODOS** os demais arquivos da pasta (`DASHBOARD.md`, `QUALITY.md`, `CRONOGRAMA.md`, `DEVELOPMENT_LOG.md`, `TESTING_GOALS.md`, `README.md`, `BUSINESS_RULES.md`, `ARQUITETURA_E_APIS.md`, `guia_aprendizado.md`, etc.) simultaneamente, garantindo zero divergências de datas, métricas de testes, versões ou status.

## Caminho operacional validado no Codex Desktop (Windows)

Esta seção é a memória operacional obrigatória para evitar repetir diagnósticos de sandbox, WSL e Node. Consulte-a antes de trocar ferramentas ou improvisar outro fluxo.

### Regra de decisão rápida

1. Rust/Cargo: executar no WSL2 Ubuntu, com `CARGO_TARGET_DIR` no filesystem Linux.
2. Frontend: usar o `node.exe` fornecido por `codex_app__load_workspace_dependencies` e chamar os arquivos JavaScript do npm/TypeScript/Vite diretamente.
3. Edição: usar `apply_patch`; se a atualização direta de um arquivo existente falhar com `helper_unknown_error`, usar o patch temporário descrito abaixo.
4. Permissão: se WSL, `.git/index.lock` ou SSH falharem com `E_ACCESSDENIED`/sandbox, repetir exatamente o mesmo comando com elevação estreita. Não mudar a estratégia técnica por causa da sandbox.

### Rust/Cargo pelo WSL2

Comando-base, ajustando o crate e o subcomando:

```powershell
wsl -d Ubuntu -- bash -lc 'cd /mnt/c/Users/leofr/Projetos/Poker_Project && export CARGO_TARGET_DIR=$HOME/poker-build/root-target && cargo test'
wsl -d Ubuntu -- bash -lc 'cd /mnt/c/Users/leofr/Projetos/Poker_Project/API-Axum && export CARGO_TARGET_DIR=$HOME/poker-build/api-target && cargo clippy --all-targets -- -D warnings'
```

- Não gerar `target` em `/mnt/c`; isso evita lentidão e dependências indevidas de `dlltool.exe`/`cl.exe` do Windows.
- Se o WSL estiver bloqueado pela sandbox, repetir o mesmo comando com aprovação elevada. Só usar Rust nativo do Windows se o WSL estiver realmente indisponível.

### Node 24, npm e build do frontend

Nunca confiar em `node`, `npm`, `npm.cmd` ou `pnpm` encontrados no `PATH` do Windows. O `npm.cmd` do sistema pode estar acoplado ao Node 18 mesmo após alterar o `PATH`. O procedimento confiável é:

1. Consultar `codex_app__load_workspace_dependencies` no início da tarefa e copiar o caminho retornado para `node.exe` (Node 24 ou a versão mais nova empacotada).
2. Invocar o CLI do npm explicitamente com esse executável:

```powershell
$node = '<caminho-retornado>\node\bin\node.exe'
$npmCli = 'C:\Program Files\nodejs\node_modules\npm\bin\npm-cli.js'
& $node $npmCli ci --prefix Frontend-Web
Push-Location Frontend-Web
& $node node_modules\typescript\bin\tsc -b
& $node node_modules\vite\bin\vite.js build
Pop-Location
```

- O lockfile canônico do frontend é `Frontend-Web/package-lock.json`; não usar `pnpm` nesse diretório.
- Se uma tentativa anterior com pnpm criar `pnpm-lock.yaml` ou contaminar `node_modules`, remover somente esse arquivo gerado e refazer `npm ci` com o Node empacotado.
- Para auditoria sem atualização forçada: `& $node $npmCli audit --prefix Frontend-Web`. Não usar `--force` sem revisar mudanças de versão principal.

### Contorno padronizado do `apply_patch` no Windows

Arquivos novos devem ser criados diretamente por `apply_patch`. Se `apply_patch` falhar ao atualizar um arquivo existente com `helper_unknown_error`:

1. Criar por `apply_patch` um script PowerShell temporário na raiz do repositório.
2. No script, ler e alterar apenas os arquivos-alvo com `[System.IO.File]::ReadAllText()` e gravar UTF-8 sem BOM com `[System.IO.File]::WriteAllText()`.
3. Executar o script e fazer ele se autoexcluir no fim.
4. Conferir imediatamente com `git diff --check`, `git diff -- <arquivos>` e `git status --short`.

Esse contorno preserva a exigência de iniciar alterações por `apply_patch`, evita edições manuais frágeis e impede arquivos auxiliares esquecidos no repositório. Respeitar LF conforme `.gitattributes`.

### Git, SSH e sandbox

- Falha ao criar `.git/index.lock`, acessar a chave SSH ou iniciar o WSL normalmente é restrição da sandbox, não defeito do código.
- Repetir o comando necessário com elevação estreita e justificativa específica; nunca solicitar uma regra ampla para PowerShell/Python e nunca apagar locks ou chaves sem verificar o alvo.
- Para operações remotas, usar a chave já configurada e manter `StrictHostKeyChecking=accept-new`; não imprimir tokens, senhas ou chaves nos logs.
- Antes de migration/deploy de banco, criar backup verificável; depois conferir versão da migration, saúde dos containers, logs recentes e invariantes de dados.

### Sequência mínima antes de concluir uma entrega

1. `git diff --check` e revisão do diff.
2. Frontend: `npm ci`, `tsc -b` e build Vite usando o Node empacotado.
3. Rust: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` e testes relevantes via WSL com `CARGO_TARGET_DIR` Linux.
4. Se houver deploy: backup, migration, health checks internos e públicos, logs e consulta dos invariantes alterados.
5. Confirmar `git status --short` limpo depois de commit e push.
