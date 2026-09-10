# Documentação — Zero Tilt Poker

Demo: [zerotiltpoker.net](https://zerotiltpoker.net). Contrato de agentes: [`../AGENTS.md`](../AGENTS.md). Estado: [`STATUS_OPERACIONAL.md`](STATUS_OPERACIONAL.md).

## Sincronização de fatos

Fonte máquina: [`STATUS_OPERACIONAL.json`](STATUS_OPERACIONAL.json) (schema v2). Humanos leem o Markdown gerado. Não coloque parágrafo no JSON.

```bash
cargo run --bin documentation-sync -- --write
cargo run --bin documentation-sync -- --check
```

O `--check` é obrigatório na CI. Prosa histórica, legal e didática não é reescrita pelo bin. **Não** atualize todos os arquivos da pasta quando um fato muda — só o JSON + `--write`, e o dono se a prosa única mudou.

## Índice (um assunto → um arquivo)

| Documento | Propósito |
|-----------|-----------|
| [`../AGENTS.md`](../AGENTS.md) | Contrato para qualquer LLM |
| `CLAUDE.md` / `GEMINI.md` / `.github/copilot-instructions.md` | Só ganchos → `AGENTS.md` |
| [`../.hermes.md`](../.hermes.md) | Piloto Hermes (observar; não é engenharia) |
| `STATUS_OPERACIONAL.md` / `.json` | Fatos: ciclo, catálogo, carteiras, PIX, limites |
| `QUALITY.md` | Gates de CI / DoD |
| `BUSINESS_RULES.md` | Regras de pôquer e dinheiro |
| `ARQUITETURA_E_APIS.md` | Contratos REST/WS |
| [`../Arquitetura-Motor/ARQUITETURA_MOTOR.md`](../Arquitetura-Motor/ARQUITETURA_MOTOR.md) | Arquitetura motor/stack |
| `DASHBOARD.md` | Sprint, backlog, fases |
| `CRONOGRAMA.md` | Redirect → DASHBOARD |
| `DEVELOPMENT_LOG.md` | História (não prevalece) |
| `DEMO_AMIGOS.md` | Convite operacional |
| `FULL_VALIDATION.md` | Carga autorizada |
| `TESTING_GOALS.md` | Redirect → FULL_VALIDATION / historico |
| `guia_aprendizado.md` | Como aprender o código |
| `RELATORIO_AMIGOS_SOCIO.md` | Pitch 4 min |
| `RELATORIO_PARCEIROS_ZEROTILT.md` | Due diligence |
| `PLANO_GO_TO_MARKET_REDE_2_NIVEIS.md` | GTM rede 2 níveis |
| `CURSO_ESTRATEGIA_POKER.md` | Curso de jogo (produto) |
| `LOSS_DEFLATOR_EXEMPLOS.md` | Exemplos do deflator |
| `SECURITY_UPGRADES_EXEMPLOS.md` | Exemplos JWT / provably fair |
| `SIMULACAO_RITUAL.md` | Snapshot 2026-09-02 |
| `SIMULACAO_ESTRUTURA_18_12.md` | Ata 18/12 |
| `TERMOS_DE_USO_E_SERVICO.md` / `PRIVACIDADE_DOMINIO.md` | Legal |
| `historico/` | Enciclopédia QUALITY, cronograma detalhado, metas Fase 2 |

Deploy: [`../Infraestrutura-Docker/DEPLOYMENT_VALIDATION.md`](../Infraestrutura-Docker/DEPLOYMENT_VALIDATION.md).

<!-- DOCUMENTATION_SYNC:START -->
> **S24** (2026-09-10) — demo `zerotiltpoker.net` · sem certificação de produção · PIX automático desligado.
> Fatos (catálogo, carteiras, limites): [`STATUS_OPERACIONAL.md`](STATUS_OPERACIONAL.md).
<!-- DOCUMENTATION_SYNC:END -->
