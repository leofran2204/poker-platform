# Validação Completa Autorizada

## Regra de execução

Esta rotina é **automática somente depois de autorização humana explícita**. Ela não é acionada por `push`, pull request ou por `cargo test` comum.

Para rodar localmente:

```powershell
.\scripts\full-validation.ps1 -Approved
.\scripts\full-validation.ps1 -Phase motor -Approved
```

```bash
FULL_VALIDATION_APPROVED=1 ./scripts/full-validation.sh
FULL_VALIDATION_APPROVED=1 ./scripts/full-validation.sh api
FULL_VALIDATION_APPROVED=1 ./scripts/full-validation.sh gateway
```

No GitHub Actions, use o workflow **Full Validation (Manual Authorization)** e escolha `approved: yes`. Os jobs de motor, API, frontend e gateway público executam em paralelo.

Cada execução gera um relatório TSV com `phase`, `status`, `duration_seconds` e
`workload`. Localmente ele fica em `artifacts/full-validation/` (ignorado pelo
Git); no GitHub Actions ele é publicado como artefato do job. Use
`FULL_VALIDATION_REPORT_DIR` para direcioná-lo a outro local.

No WSL, o lote do frontend troca explicitamente para a toolchain Linux, pois o
projeto mantém a toolchain GNU do Windows para o desenvolvimento nativo.
Para a fase `gateway` da versão Bash, habilite também a integração do Docker
Desktop com a distribuição WSL; no Windows, a versão PowerShell usa o Docker
Desktop diretamente.

PIX e seus testes de pagamento permanecem fora da rotina até nova autorização de escopo.

O perfil `all` também reconstrói a API e o Caddy e executa a fase `gateway`; os
quatro lotes ficam, portanto, vinculados à mesma autorização humana. Para rodar
somente a borda pública, use a fase `gateway`.

## Gateway público HTTPS/WSS

Depois de subir a stack Docker completa, execute a verificação E2E do ponto de
entrada Caddy. Ela comprova a negociação TLS local, HSTS, o redirecionamento
de HTTP para HTTPS, a API publicada por HTTPS e o handshake WSS. O cliente de
teste deliberadamente não envia ticket: receber `101 Switching Protocols` prova
o transporte WSS; o backend então encerra a sessão não autorizada.

```bash
docker compose -f Infraestrutura-Docker/docker-compose.yml up -d --build
PUBLIC_GATEWAY_INSECURE_LOCAL_CERT=1 bash ./scripts/verify-public-https.sh
```

`PUBLIC_GATEWAY_INSECURE_LOCAL_CERT=1` só é aceitável para o certificado local
gerado pelo Caddy. Em staging ou produção, deixe a variável ausente e forneça
uma origem HTTPS com certificado confiável em `PUBLIC_GATEWAY_URL`.

Para a fase API local, inicie PostgreSQL e Redis do projeto. Os scripts usam
por padrão a URL do `docker-compose` local (`localhost:5433`); exporte
`DATABASE_URL` antes do comando se desejar apontar para outro ambiente.

```powershell
docker compose -f .\Infraestrutura-Docker\docker-compose.yml up -d postgres redis
```

No perfil de carga da API, a feature transitiva `test-fast-bcrypt` usa custo 4
somente no binário de teste. O binário de produção continua usando bcrypt com
custo 12; os fuzzes exercitam validação, autorização e rate limiting, não a
calibração de custo do algoritmo.

## Piso de 100 cenários de carga

O perfil tem 100 cenários centrais distintos, além da suíte funcional normal:

| Área | Cenários | Carga |
|---|---:|---|
| Motor Rust | 79 | Monte Carlo, CSPRNG, fairness, fuzzing, side pots, torneios e invariantes financeiras |
| API HTTPS | 10 | Fuzz property-based de endpoints, com 2.000 casos por cenário |
| Frontend | 10 | Fuzz visual/de estado, 200.000 casos por cenário |
| WebSocket | 1 | 100 mesas, 9 jogadores e 1.000.800 mensagens |
| **Total central** | **100** | Execução somente autorizada |

Além desses 100, a mesma rotina executa os testes funcionais de API, os sete contratos PostgreSQL, segurança, reconexão, jitter, concorrência, desconexão, persistência PostgreSQL concorrente e stress de estado do frontend. Eles não são usados para inflar artificialmente a métrica de 100 cenários centrais.

No frontend, o alvo de biblioteca é canônico: os fuzzes e stresses são executados uma única vez por perfil, sem duplicação pelo binário da aplicação.

## Lotes para execução local

| Fase | Comando | Objetivo |
|---|---|---|
| Motor | `-Phase motor` / `motor` | 1.813 testes de rotina e 79 cenários de carga |
| API | `-Phase api` / `api` | HTTPS, WSS e segurança; requer PostgreSQL local ou `DATABASE_URL` apontando para o ambiente autorizado |
| Frontend | `-Phase frontend` / `frontend` | Testes funcionais, 10 fuzzes e dois stresses de estado |
| Gateway | `-Phase gateway` / `gateway` | Reconstrói API/Caddy e valida HTTPS, HSTS, redirecionamento e handshake WSS |

Os lotes podem ser executados separadamente para reduzir o tempo de espera. O resultado só é considerado completo quando todos terminarem com sucesso.

## Evidência de carga

Não use apenas a quantidade de funções de teste como critério. A evidência de
carga de cada execução deve registrar pelo menos: duração, orçamento de
iterações/mensagens e status. Os orçamentos atuais são 79 cenários massivos do
motor, 20.000 entradas property-based da API HTTPS, 1.000.800 mensagens WSS e
2.000.000 entradas de fuzz no frontend. Esse registro permite comparar uma
execução à outra sem reduzir a massividade a uma contagem nominal de testes.
