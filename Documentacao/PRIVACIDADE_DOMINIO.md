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
> **Estado operacional sincronizado (2026-09-04):** S21 — Texas Hold’em rename + FT Short Deck 8-max + Omaha 5-max + Pineapple 6-max + Short Deck ranking trips>straight + torneio agendado 21:30 SP auto-start 5 + Pix Leofran + saque 24h + lobby max sempre + sim 100k/mesa + PM 150+150 sem rebuy (ilimitado com saldo, play money) **Sem certificação de produção; o código rejeita PIX em modo production. Deploy público: VPS Hostinger (demo/staging) com domínio zerotiltpoker.net. Staging/demo apenas; não alegar Launch Ready de produção.** Stack Docker local 4/4 healthy e VPS Hostinger 4/4 healthy. Migrations 001–044 na VPS (cash Texas SD 8-max + Omaha 5-max + Pineapple 6-max + Texas rename + FT 8 + scheduled 21:30 + PM 150+150 + restore catálogo + Dockerfile cache). PM duas carteiras R$150 sem rebuy (ilimitado com saldo). Motor short_deck_massive + tournament_to_champion PASS (Texas/Omaha 5/Pineapple 6 até 1 campeão; flush>FH e trips>straight). VPS 2h real 100 contas: 980 mãos R$135,11 rake, 4 campeões MTT. Simulado Motor-Rust/src/bin/simulated_100.rs 100k/mesa (400k total). Lobby GET /api/lobby/tables lista mesas OPEN mesmo lotadas com X-max sempre. MTT site: inscrição + horário agendado + popup FT; gameplay_ready=false (sem WS de torneio). Health público OK. Recebedor manual: Leofran, chave 6eefcd53-686e-42d4-a062-03751336251c (PLAY_MONEY_PIX_KEY). Saque: informar chave Pix própria, recebimento em até 24h. A VPS permanece no padrão seguro PIX mock. DePix existe somente em Sandbox não produtivo, com chave sk_test_, allowlist de depositante, idempotência, HMAC com janela temporal, deduplicação de eventos e crédito apenas em checkout.completed. O CPF/CNPJ é encaminhado ao provedor sem persistência local. Depósito manual continua como fallback; não há saque automático. Mesas com dono único por processo; settlement assinado (HMAC) na liquidação.
>
> Fonte canônica: [`STATUS_OPERACIONAL.json`](STATUS_OPERACIONAL.json). Verificação: `cargo run --bin documentation-sync -- --check`.
<!-- DOCUMENTATION_SYNC:END -->
