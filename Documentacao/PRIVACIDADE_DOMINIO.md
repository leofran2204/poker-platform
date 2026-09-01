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
> **Estado operacional sincronizado (2026-09-01):** S20 — Big Blind Ante 26 níveis nos torneios + potes laterais com ante morto; cash permanece sem ante; catálogo cash canônico NLHE 0,25/0,25, Hold'em Short Deck 0,25/0,50 e Omaha Short Deck 0,50/0,50 (Play Money e Jogo Real). **Sem certificação de produção; o código rejeita PIX em modo production. Deploy público: VPS Hostinger (demo/staging) com domínio zerotiltpoker.net. Staging/demo apenas; não alegar Launch Ready de produção.** Stack Docker local 4/4 healthy e VPS Hostinger 4/4 healthy (poker_api/poker_frontend/poker_postgres/poker_redis); migrations 001–032 aplicadas (BBA). Gate S20: cargo fmt, Clippy estrito (Motor + API), 1828 testes Motor-Rust (incl. 3 BBA) + 35 testes API-Axum + TypeScript tsc + Vite build — todos sem falhas; VPS validado com 6 torneios 26 níveis ante=big_blind (invalid_BBA=0), backup verificável, rebuild 4m13s e health público OK. Mantidas evidências de stress Short Deck e catálogo cash. A VPS permanece no padrão seguro PIX mock. DePix existe somente em Sandbox não produtivo, com chave sk_test_, allowlist de depositante, idempotência, HMAC com janela temporal, deduplicação de eventos e crédito apenas em checkout.completed. O CPF/CNPJ é encaminhado ao provedor sem persistência local. Depósito manual continua como fallback; não há saque automático. Mesas com dono único por processo; settlement assinado (HMAC) na liquidação.
>
> Fonte canônica: [`STATUS_OPERACIONAL.json`](STATUS_OPERACIONAL.json). Verificação: `cargo run --bin documentation-sync -- --check`.
<!-- DOCUMENTATION_SYNC:END -->
