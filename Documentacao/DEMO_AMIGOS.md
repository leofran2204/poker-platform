# Demo com amigos — zerotiltpoker.net

Guia curto para convidar dezenas de pessoas a testar e mandar feedback.

## O que cada amigo precisa fazer

1. Abrir **https://zerotiltpoker.net** (HTTPS público Let's Encrypt)
2. Ver o contador **“X online”** no topo (e o banner na home) — só conta quem **está logado** com heartbeat recente (~90s)
3. **Registrar** (username 3–30 chars, e-mail válido, senha forte + confirmação; ex. `PokerDemo1`)
4. **Verificar e-mail** — código de 6 dígitos enviado via Resend (inbox + spam); tela `/verify-email`
5. Ir ao **Lobby** e combinar a **mesma mesa** com pelo menos **2 pessoas**
6. Clicar **Entrar** (buy-in mínimo automático)
7. Jogar e anotar bugs / sensações

> **Importante:** 1 pessoa sozinha no site **não** inicia mão. Precisa de **≥ 2 assentos ocupados na mesma mesa**.

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

1. **Stack demo** — VPS Hostinger já hospeda `zerotiltpoker.net` (API + `Frontend-Web` + Caddy). Em casa (alternativa):

```powershell
cd C:\Users\leofr\Projetos\Poker_Project\Infraestrutura-Docker
docker compose -f docker-compose.yml -f docker-compose.tunnel.yml up -d --build
# + cloudflared tunnel run …
```

A migration `013_demo_public_tables.sql` roda no boot da API e cria as mesas se ainda não existirem.

2. Confirme o contador online e o health: `https://zerotiltpoker.net/api/health` e `/api/presence/online`.
3. Peça feedback estruturado: registro, e-mail, contador online, lobby, join, lag, mobile, crashes.

## Limites honestos (avise os amigos)

- Demo em casa: se o PC ou o tunnel cair, o site some.
- Até ~9 jogadores por mesa; várias mesas no lobby.
- Rate limit de auth ~30 req/min por IP (rede compartilhada pode “engasgar” cadastros em massa no mesmo Wi‑Fi).
- Não é produção nem jogo com dinheiro real.

## Checklist rápido “está pronto?”

```text
[ ] https://zerotiltpoker.net/api/health → OK
[ ] https://zerotiltpoker.net/api/presence/online → JSON online_count
[ ] Badge “N online” visível no header
[ ] Registrar conta nova no browser
[ ] Após login, contador sobe (heartbeat)
[ ] Lobby lista mesas Demo NL*
[ ] Dois perfis entram na MESMA mesa (saldo R$1000)
[ ] Mão inicia com ≥ 2 assentos
```

<!-- DOCUMENTATION_SYNC:START -->
> **Estado operacional sincronizado (2026-08-26):** S13+ — Presença online + home NewsTips (notícias/dicas); S12 (MFA, settlements 017, smoke 10×100); demo VPS zerotiltpoker.net (play-money, mín. 2 na mesma mesa). Repo canônico: Projetos/Poker_Project (não OneDrive). **Sem certificação de produção; o código rejeita PIX em modo production. Deploy público: VPS Hostinger (demo/staging) com domínio zerotiltpoker.net. Staging/demo apenas; não alegar Launch Ready de produção.** VPS stack healthy (postgres, redis, api, frontend/Caddy). Migrations 001–017. Presence API no ar: GET /api/presence/online e POST /api/presence/heartbeat (TTL 90s, Redis). Smoke live 10×100 PASS (0833 jornada; 0920 settlementsVerified=2). Frontend badge/hero online deployados. Mock é o padrão. Asaas Sandbox restrito por PIX_ALLOWED_DEPOSITOR_IDS; Mercado Pago e PIX de produção desabilitados. Nenhum depósito com dinheiro real. Mesas com dono único por processo; guarda de recovery entre início e liquidação. Settlement assinado (HMAC) na liquidação; API verifica no replay.
>
> Fonte canônica: [`STATUS_OPERACIONAL.json`](STATUS_OPERACIONAL.json). Verificação: `cargo run --bin documentation-sync -- --check`.
<!-- DOCUMENTATION_SYNC:END -->
