# Demo com amigos — zerotiltpoker.net

Guia curto para convidar dezenas de pessoas a testar e mandar feedback.

## O que cada amigo precisa fazer

1. Abrir **https://zerotiltpoker.net**
2. **Registrar** (username 3–30 chars, email válido, senha forte: maiúscula + minúscula + número, ex. `PokerDemo1`)
3. Ir ao **Lobby**
4. Clicar **Entrar** em uma mesa (buy-in mínimo automático)
5. Jogar e anotar bugs / sensações

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

1) Crie conta (senha tipo PokerDemo1 — precisa letra maiúscula e número)
2) Lobby → Entrar numa mesa
3) Me diga: o que travou, o que gostou, se a mesa abriu ok

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
