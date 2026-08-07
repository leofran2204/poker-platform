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
> **Estado operacional sincronizado (2026-08-07):** S12 — Auth MFA + supply-chain CI; ações legais na mesa; settle pós-disconnect; liquidação de mão assinada (migração 017); smoke live 10 usuários/100 mãos com settlement verificado na VPS demo; branch codex/security-supply-chain fechada e documentada. **Sem certificação de produção; o código rejeita PIX em modo production. Deploy público: VPS Hostinger (demo/staging) com domínio zerotiltpoker.net. Staging/demo apenas; não alegar Launch Ready de produção.** VPS stack healthy (postgres, redis, api, frontend/Caddy). Migrations 001–017 aplicadas (017 hand settlement audit). Smoke live scripts/live-e2e-ten-users.mjs: run 202608070833 PASS (10 reg/100 mãos); run 202608070920 PASS com settlementsVerified=2 (assinatura + winner + payouts+rake=pote por mesa). Simulação motor 100k mãos release OK. Segundo lote sintético zte2e202608070920* removido; lote original zte2e202608070833* preservado (10 contas demo). Suíte histórica motor/API + gates supply-chain (Dependabot, audit, SBOM/Trivy workflows). Mock é o padrão. O único adaptador externo é o Asaas Sandbox, restrito por PIX_ALLOWED_DEPOSITOR_IDS; Mercado Pago e PIX de produção permanecem desabilitados. Nenhum depósito com dinheiro real foi habilitado. Mesas continuam com dono único por processo; uma guarda persistente pausa a mesa após falha entre início e liquidação da mão, exigindo revisão/abort administrativo antes da reabertura. Liquidação de mão agora persiste settlement assinado (HMAC) e a API verifica assinatura no replay; históricos legados sem assinatura permanecem legíveis como não verificados.
>
> Fonte canônica: [`STATUS_OPERACIONAL.json`](STATUS_OPERACIONAL.json). Verificação: `cargo run --bin documentation-sync -- --check`.
<!-- DOCUMENTATION_SYNC:END -->
