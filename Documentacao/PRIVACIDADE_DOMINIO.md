# Privacidade do domínio e do site

## WHOIS / RDAP (`zerotiltpoker.net`)

- Registrador: **Hostinger**
- Consulta pública (RDAP) em 2026-08-29: **titular mascarado** (nome vazio; contato só via formulário Hostinger)
- Confirme no painel Hostinger → Domínios → **WHOIS Privacy / Domain Privacy** = **ON**
- Se estiver OFF: ative e aguarde propagação RDAP (minutos a horas)

Não dá para esconder 100%: datas de registro, nameservers e IP do VPS continuam públicos.

## O que o site já faz

- `robots.txt` bloqueia `/admin`, `/wallet`, login/registro, `/api/`
- Caddy envia `X-Robots-Tag: noindex` nessas rotas
- SPA marca meta `noindex` em Carteira e Admin
- Chave PIX na Carteira fica **mascarada** até “Mostrar chave” / “Copiar”
- Nome do recebedor PIX permanece completo (necessário para o jogador conferir no banco)

## Limites honestos

| Vetor | Status |
|-------|--------|
| WHOIS público | Em geral privado (Hostinger) |
| Nome PIX na Carteira | Visível a quem está logado e pede fichas |
| Certificate Transparency | Domínio listado em logs CT |
| IP do servidor | Descobrível por DNS |

## Hostinger — checklist

1. Domínios → `zerotiltpoker.net` → Privacy / WHOIS Protection **ativado**
2. Contato do domínio: e-mail de alias, não o pessoal se possível
3. Não publicar CPF/telefone/endereço em páginas públicas

<!-- DOCUMENTATION_SYNC:START -->
> **Estado operacional sincronizado (2026-09-10):** S23 — cancela inscrição com reembolso total + admin agenda e cria torneios + textos da TournamentPage **Sem certificação de produção; o código rejeita PIX em modo production. Deploy público: VPS Hostinger (demo/staging) com domínio zerotiltpoker.net. Staging/demo apenas; não alegar Launch Ready de produção.** Stack Docker local 4/4 healthy e VPS Hostinger 4/4 healthy. Migrations 001–049 na VPS (045 convite/fila, 046 ledger estrutura, 047 bots, 048 3 mesas + fee ledger, 049 total_fees). PM duas carteiras R$150 sem rebuy (ilimitado com saldo). Motor 1848 lib (fee 15%, seating 3 mesas, run-out all-in) + API 43 lib + ator MTT 2 testes integração PASS. VPS: 1º MTT fim a fim (freeroll 6 inscritos, 3 mesas, 5 mãos assinadas, campeão + payout GTD). Bots lag_v2 em MTT (12 inscritos, 43 mãos assinadas, zero erros). Lobby GET /api/lobby/tables lista mesas OPEN mesmo lotadas com X-max sempre. MTT: inscrição + 3 mesas + gameplay WS ao vivo (mesmo protocolo do cash) + rebalance/consolidação FT + payouts; gameplay_ready=true. Health público OK. Diário de mãos + replay no frontend (2026-09-10): grava suas mãos no navegador (suas cartas, board, pote, resultado), replay passo a passo, download TXT/JSON, painel do vencedor sem botão (só as 5 cartas saltam, some sozinho em 7s). Ritmo de digestão no frontend (2026-09-10): board com stagger de 220ms por carta + painel de resultado fixo do showdown (vencedor, mão e cartas reveladas, sem auto-fechar, sobrevive à mão seguinte). Bots da casa desligados na VPS (stop oficial, reembolso) a pedido. Ritual do crupiê no frontend (2026-09-10): banner de embaralhamento + cartas distribuídas por assento a partir do dealer, versos para os oponentes, stagger no board. Migration 053 (2026-09-10): cash Texas 9-max NL 0,75/1,50 frente 15000 em PM e Real + torneios Texas R$25 em PM e Real (buy-in 2500, stack 15000, 1 reentrada 2500/25000, agenda 21:30 SP, auto-start 5). Bots externos de estratégia validados no local em 2026-09-10: bot/strategy (ranges cash 6-max/9-max, MTT ChipEV/ICM/PKO, avaliador próprio 5-7 cartas, push/fold FT, pot odds) com tsc limpo + 13/13 selftest offline; scripts/strategy-bots.mjs jogou mesa real PM NL 0,25 (3 bots, 5-6 mãos cada, 5 mãos no hand_history, decisões por ranges/odds). Tabelas ICM versionadas em bot/strategy/icm/tables (geradas de final_table.ts/bubble.ts via generate.mjs). Recebedor manual: Leofran, chave 6eefcd53-686e-42d4-a062-03751336251c (PLAY_MONEY_PIX_KEY). Saque: informar chave Pix própria, recebimento em até 24h. A VPS permanece no padrão seguro PIX mock. DePix existe somente em Sandbox não produtivo, com chave sk_test_, allowlist de depositante, idempotência, HMAC com janela temporal, deduplicação de eventos e crédito apenas em checkout.completed. O CPF/CNPJ é encaminhado ao provedor sem persistência local. Depósito manual continua como fallback; não há saque automático. Mesas com dono único por processo (cash TableActor + torneio TournamentActor, mesmo protocolo); settlement assinado (HMAC) na liquidação; halt de mesa MTT auditável (MTT_TABLE_HALTED).
>
> Fonte canônica: [`STATUS_OPERACIONAL.json`](STATUS_OPERACIONAL.json). Verificação: `cargo run --bin documentation-sync -- --check`.
<!-- DOCUMENTATION_SYNC:END -->
