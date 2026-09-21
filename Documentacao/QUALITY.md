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

## Definition of Done

Uma mudança está pronta **localmente** quando:

1. Compila e clippy `-D warnings` no crate tocado (WSL).
2. Testes determinísticos relevantes passam (sem disparar full-validation).
3. Frontend tocado: `tsc` + lint + Vitest + build Vite com Node empacotado.
4. Fatos operacionais: JSON schema v2 + `documentation-sync --write` e `--check`. Prosa única só no arquivo dono (`AGENTS.md`).
5. Sem regressão óbvia no fluxo alterado.

Commit / push / deploy **não** fazem parte do DoD local — cada um exige ordem explícita (`AGENTS.md`).

## Onde não procurar qualidade

- Aprendizado / protocolo Mark → `guia_aprendizado.md`
- OKR, marketing, chaos, “plano Elon” → `historico/QUALITY_v5.4.md` (arquivo, não vigente)
- Catálogo e ciclo → STATUS
- Fases e backlog → DASHBOARD

<!-- DOCUMENTATION_SYNC:START -->
> **S24** (2026-09-21) — demo `zerotiltpoker.net` · sem certificação de produção · PIX automático ligado (DePix reconciliado).
> Fatos (catálogo, carteiras, limites): [`STATUS_OPERACIONAL.md`](STATUS_OPERACIONAL.md).
<!-- DOCUMENTATION_SYNC:END -->
