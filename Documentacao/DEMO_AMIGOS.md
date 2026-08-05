# Demo com amigos — zerotiltpoker.net

Guia curto para convidar dezenas de pessoas a testar e mandar feedback.

## O que cada amigo precisa fazer

1. Abrir **https://zerotiltpoker.net** (HTTPS público Let's Encrypt)
2. **Registrar** (username 3–30 chars, e-mail válido, senha forte + confirmação; ex. `PokerDemo1`)
3. **Verificar e-mail** — código de 6 dígitos enviado via Resend (inbox + spam); tela `/verify-email`
4. Ir ao **Lobby**
5. Clicar **Entrar** em uma mesa (buy-in mínimo automático)
6. Jogar e anotar bugs / sensações

## O que a conta ganha

| Item | Valor |
|------|--------|
| Saldo inicial | **R$ 1.000,00** (100 000 centavos) play-money |
| PIX real | **Não** — mock only |
| Mesas | Várias demo NL2 / NL5 / NL10 / NL25 (9 assentos cada) |

## Mensagem pronta para WhatsApp / Discord

```text
Teste do Zero Tilt Poker (demo HTTPS, fichas virtuais):

https://zerotiltpoker.net

1) Crie conta (senha tipo PokerDemo1 — maiúscula + minúscula + número; confirme a senha)
2) Abra o e-mail e digite o código de 6 dígitos (olhe o spam)
3) Lobby → Entrar numa mesa
4) Me diga: o que travou, o que gostou, se a mesa abriu ok

É play-money, sem dinheiro real. Site no ar só enquanto a demo estiver ligada.
```

## Do seu lado (anfitrião)

1. **Rebuild e subir** após este pacote de demo (API + frontend WASM):

```powershell
cd C:\Users\leofr\Projetos\Poker_Project\Infraestrutura-Docker
docker compose -f docker-compose.yml -f docker-compose.tunnel.yml up -d --build
# + cloudflared tunnel run …
```

A migration `013_demo_public_tables.sql` roda no boot da API e cria as mesas se ainda não existirem.

2. PC ligado + tunnel/Docker estáveis.
3. Peça feedback estruturado: registro, lobby, join, lag, mobile, crashes.

## Limites honestos (avise os amigos)

- Demo em casa: se o PC ou o tunnel cair, o site some.
- Até ~9 jogadores por mesa; várias mesas no lobby.
- Rate limit de auth ~30 req/min por IP (rede compartilhada pode “engasgar” cadastros em massa no mesmo Wi‑Fi).
- Não é produção nem jogo com dinheiro real.

## Checklist rápido “está pronto?”

```text
[ ] https://zerotiltpoker.net/caddy-health → OK
[ ] Registrar conta nova no browser
[ ] Lobby lista mesas Demo NL*
[ ] Entrar na mesa (saldo R$1000)
[ ] Segundo browser/perfil entra na mesma mesa
```

<!-- DOCUMENTATION_SYNC:START -->
> **Estado operacional sincronizado (2026-08-01):** S10 — B2B SaaS Multi-Tenant White-Label (clubs, rake split 15/85, agentes/rakeback, dashboard admin via HTTPS, lobby MTT); domínio público zerotiltpoker.net; demo em casa com Cloudflare Tunnel e HTTPS de ponta a ponta; VPS/Hetzner opcional. **Sem certificação de produção; o código rejeita PIX em modo production. Deploy público previsto: demo residencial (Cloudflare Tunnel) ou VPS opcional — não multi-AZ. Staging/demo apenas; não alegar Launch Ready de produção.** Infra e docs de deploy (compose com healthchecks Postgres/Redis/API/Caddy, Caddyfile.tunnel HTTPS, DEPLOY_HOME_CLOUDFLARE, DEPLOY_HETZNER, .env.staging/.env.tunnel/.env.production.example) versionados no working tree. Suíte histórica reportada: ~2.051 testes (1.904 Motor Rust + 115 Dioxus + 32 API); full-validation e smoke em https://zerotiltpoker.net ainda pendem de execução. Migration 014 (clubs, memberships, club_agents, club_id em mesas/torneios) e endpoints admin B2B implementados localmente. Mock é o padrão. O único adaptador externo é o Asaas Sandbox, restrito por PIX_ALLOWED_DEPOSITOR_IDS; Mercado Pago e PIX de produção permanecem desabilitados. Nenhum depósito com dinheiro real foi habilitado. Mesas continuam com dono único por processo; uma guarda persistente pausa a mesa após falha entre início e liquidação da mão, exigindo revisão/abort administrativo antes da reabertura.
>
> Fonte canônica: [STATUS_OPERACIONAL.json](STATUS_OPERACIONAL.json). Verificação: cargo run --bin documentation-sync -- --check.
<!-- DOCUMENTATION_SYNC:END -->
