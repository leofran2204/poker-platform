# Deploy em VPS — Poker Platform

**Objetivo:** subir a stack Docker (Postgres + Redis + API Axum + Frontend/Caddy) em VPS Ubuntu.
**Status do produto:** staging / demo. **Sem certificação de produção.** A demo vigente está na Hostinger, com DePix reconciliado conforme o [STATUS](../Documentacao/STATUS_OPERACIONAL.md). Novos laboratórios usam mock por padrão; preservar a configuração autorizada do servidor existente. Mesas têm dono único por processo.

**Domínio do produto:** [`zerotiltpoker.net`](https://zerotiltpoker.net)  
**Host público (staging/demo):** `zerotiltpoker.net` (apex) — Caddy + Let's Encrypt + reverse_proxy da API no mesmo host.

> **Sem cartão / sem VPS?** Use a demo em casa com HTTPS: [`DEPLOY_HOME_CLOUDFLARE.md`](DEPLOY_HOME_CLOUDFLARE.md) (Cloudflare Tunnel + Origin CA). Este guia é para **VPS 24/7** (Hetzner ou Ubuntu em provedor BR com PIX).

> Fonte de stack: `docker-compose.yml`, Dockerfiles API/Frontend, `Caddyfile` (LE), `.env.staging.example`.

---

## 1. Capacidade e provedor

O procedimento vale para uma VPS Ubuntu x86_64 com Docker. A instalação vigente está na Hostinger; o nome histórico deste arquivo foi preservado para manter os links. Conferir RAM, disco livre e uso atual antes do build Rust; imagens podem ser construídas em CI quando a VPS não comportar o compilador. Preços, planos e cotas devem ser consultados no provedor ao contratar.

A API permanece com uma réplica, responsável pelos atores das mesas.

---

## 2. Ordem dos containers (igual ao compose)

```text
1. postgres     (volume postgres_data; healthcheck pg_isready)
2. redis        (volume redis_data; healthcheck PING)
3. poker_api    (build Motor-Rust + API-Axum; healthcheck curl http://127.0.0.1:3000/health — readiness Postgres+Redis)
4. poker_frontend (Caddy + SPA React/Vite; healthcheck wget :80; depends_on API healthy; publica 80/443 HTTPS)
```

Portas públicas na VPS: **apenas 22, 80, 443** (tráfego de produto em **HTTPS**).  
Postgres e Redis ficam em `127.0.0.1` no compose (não exponha na internet).  
Env de referência: `.env.staging.example` ou `../.env.production.example` (nunca commitar `.env` real).

---

## 3. Checklist pré-voo

- [ ] Acesso administrativo à VPS e capacidade disponível para build
- [ ] Domínio **zerotiltpoker.net** com registro **A** (e opcional **www**) para o IP da VM
- [ ] Repositório no GitHub (já: `poker-platform`)
- [ ] Segredos gerados (nunca commitar `.env`):
  - `JWT_SECRET` ≥ 32 bytes aleatórios
  - `EMAIL_CODE_PEPPER` e `KYC_DATA_PEPPER` ≥ 32 bytes, aleatórios, exclusivos e diferentes do JWT
  - `POSTGRES_PASSWORD` forte
  - `PIX_PROVIDER=mock` / `PIX_MODE=mock` em um laboratório novo
  - Não misturar chaves de teste e produção; na demo existente, conservar o DePix autorizado, seus limites e reconciliação
- [ ] `CORS_ORIGINS=https://zerotiltpoker.net` (só HTTPS; a API **rejeita** origem sem HTTPS)

---

## 4. Passo a passo na VPS

### 4.1 Criar a Cloud Server

1. Hetzner Console → **New project** → **Add server**
2. Location: Ashburn ou DE  
3. Image: **Ubuntu 24.04**  
4. Type: capacidade de CPU, memória e disco dimensionada para os serviços e o build
5. SSH key: cole sua chave pública  
6. Firewall (criar e anexar):

| Direção | Porta | Origem |
|---------|-------|--------|
| In | 22/tcp | seu IP (ideal) ou 0.0.0.0/0 |
| In | 80/tcp | 0.0.0.0/0 |
| In | 443/tcp | 0.0.0.0/0 |
| Out | any | any |

7. Create → anote o **IPv4**

### 4.2 DNS (zerotiltpoker.net)

No provedor onde registrou o domínio (registro A no painel DNS):

```text
A      @      →   IP_DA_VPS          # zerotiltpoker.net
A      www    →   IP_DA_VPS          # opcional: www.zerotiltpoker.net
```

Se o painel não aceitar `@`, use o hostname nu `zerotiltpoker.net` ou o registro de apex que o registrar oferecer.

TTL baixo (300s) na primeira vez. Espere propagar:

```bash
dig +short zerotiltpoker.net A
# deve imprimir o IPv4 da Hetzner
```

**Ordem importante:** DNS apontando **antes** (ou junto) do `docker compose up` do frontend — o Caddy só emite Let's Encrypt se a 80/443 do host responder no nome público.

### 4.3 Preparar o servidor

```bash
ssh root@IP_DA_VPS

apt update && apt upgrade -y
apt install -y ca-certificates curl git ufw

# Docker oficial
curl -fsSL https://get.docker.com | sh
systemctl enable --now docker

# Firewall host (além do Hetzner)
ufw allow OpenSSH
ufw allow 80/tcp
ufw allow 443/tcp
ufw --force enable

# Usuário deploy (opcional mas recomendado)
adduser deploy
usermod -aG docker deploy
# copie sua authorized_keys para /home/deploy/.ssh/
```

### 4.4 Clonar o projeto

```bash
# como deploy ou root
cd /opt
git clone https://github.com/leofran2204/poker-platform.git
cd poker-platform
```

### 4.5 Arquivo `.env` de staging

O `docker-compose.yml` já interpola `POSTGRES_*`, `DATABASE_URL`, `JWT_SECRET`, `EMAIL_CODE_PEPPER`, `KYC_DATA_PEPPER`, `CORS_ORIGINS`, `DOMAIN_NAME`, PIX e rake a partir do `.env`. Não é necessário editar o YAML na VPS.

```bash
cd /opt/poker-platform/Infraestrutura-Docker
cp .env.staging.example .env
# gere senhas/JWT (nunca use os placeholders TROCAR_*):
#   openssl rand -base64 48
# edite POSTGRES_PASSWORD, DATABASE_URL, JWT_SECRET, EMAIL_CODE_PEPPER,
# KYC_DATA_PEPPER e PIX_WEBHOOK_SECRET; use valores independentes
chmod 600 .env
```

Template versionado: **`.env.staging.example`** (já com `DOMAIN_NAME=zerotiltpoker.net` e CORS HTTPS).

**Invariantes:**

- `DATABASE_URL` deve usar o **mesmo** user/senha/db de `POSTGRES_*`.
- `CORS_ORIGINS` = origem HTTPS completa (`https://zerotiltpoker.net`).
- `DOMAIN_NAME=zerotiltpoker.net` **sem** `https://`.
- Se servir também `www`, inclua no CORS: `https://zerotiltpoker.net,https://www.zerotiltpoker.net` e considere redirect www→apex no Caddy (opcional).
- PIX: laboratório novo inicia em `mock`. O deploy de código não troca o provedor, as chaves, os limites ou o modo da demo existente.

### 4.6 Caddy / domínio

O `Caddyfile` do repo já usa `{$DOMAIN_NAME:localhost}`. O serviço `poker_frontend` recebe `DOMAIN_NAME` do compose — **não** é preciso colar o domínio no arquivo se o `.env` estiver certo.

Caddy pede certificado Let's Encrypt sozinho nas portas 80/443 quando `DOMAIN_NAME` é um hostname público com DNS apontando para a VPS.

### 4.7 Subir a stack (ordem correta)

```bash
cd /opt/poker-platform/Infraestrutura-Docker

# 1) Build (API Rust + Frontend React/Vite — demora na 1ª vez; use 8 GB de RAM)
docker compose build

# 2) Sobe dependências e apps
docker compose up -d

# 3) Logs
docker compose ps
docker compose logs -f poker_api
```

Ordem automática do Compose: **postgres → redis → api → frontend**.

### 4.8 Validar

```bash
curl -fsS https://zerotiltpoker.net/health
curl -fsS https://zerotiltpoker.net/caddy-health
curl -fsS https://zerotiltpoker.net/api/lobby/tables
```

Navegador: `https://zerotiltpoker.net/login`

Smoke de mesa: register/login → lobby → join → WS (all-in pode demorar no preflop por Monte Carlo de equity).

---

## 5. Backups e manutenção

| Ação | Como |
|------|------|
| Snapshot Hetzner | Console → Volume/Server → **Enable backups** ou snapshot semanal |
| Dump Postgres | `docker exec poker_postgres pg_dump -U poker_user poker_db > backup.sql` |
| Atualizar código | Backup → `REBUILD_API=1 bash scripts/vps-redeploy-frontend.sh` na raiz → [validação](DEPLOYMENT_VALIDATION.md) |
| Logs | `docker compose logs -f --tail=200` |
| Disco | `df -h`; limpar `docker system prune` com cuidado |

---

### Troca para `brazilian_pineapple_pot_before_call_v2`

A regra, os treinos e o filme devem entrar juntos. Antes da troca da API, confirmar ausência de mãos Pineapple abertas e de torneios Pineapple em andamento. Nesta publicação, usar uma janela completamente ociosa: zero `cash_game_seats` com status `ACTIVE`, zero torneios `running`, nenhuma inscrição em torneio agendado e nenhum snapshot Redis `poker:table:state:*` com mão aberta ou jogadores. Contagem do lobby sozinha não é evidência suficiente.

Preparar as imagens antes da janela. Bloquear temporariamente novas entradas de jogo no proxy (inscrições cash/MTT, início administrativo e WebSockets), mantendo leitura e callbacks financeiros; repetir as consultas após drenar requisições. Somente com os critérios atendidos, parar a API antiga, repetir a conferência estável e criar os containers novos. Se houver jogo em andamento, retirar o bloqueio e adiar a troca. Não cancelar torneios, expulsar jogadores nem alterar saldos para obter uma janela.

Guardar dump PostgreSQL validado por `pg_restore --list`, volumes Caddy verificados e tags das imagens anteriores; preservar `.env`. Conferir o SHA aprovado antes de construir e antes de subir, as 60 migrations/checksums, os 90 arquivos de mídia, saúde dos quatro serviços, `/api/academy/play` na versão 3, ante nominal/pago/pagador, versão da regra e os capítulos do filme publicado. Restaurar a configuração normal do proxy ao terminar ou abortar.

## 6. O que **não** fazer neste deploy

- `ENVIRONMENT=production` com JWT ou peppers fracos/reutilizados (a API recusa iniciar)
- Trocar configuração financeira ou ativar agentes/fechar ciclos como efeito colateral do deploy
- Expor Postgres/Redis na internet
- Esperar multi-pod de mesas (ainda **1 dono por processo**)
- Usar free tier Oracle **como se fosse** o mesmo que Hetzner (capacidade e ARM mudam o jogo)

---

## 7. Custos operacionais

Registrar os custos reais de VPS, backups, domínio e tráfego. As estimativas antigas de planos Hetzner foram retiradas por não descreverem a instalação vigente.

---

## 8. Próximos passos opcionais

1. CI no GitHub Actions: build de imagens e push para registry; na VPS só `pull` + `up` (VM menor possível).  
2. Firewall Hetzner + fail2ban no SSH.  
3. Monitoring simples (`htop`, `docker stats`, uptime do `/health`).  
4. Quando for “produção de verdade”: ownership de mesa, PIX autorizado, TLS e secrets em vault, multi-AZ — **fora** deste guia.

---

## 9. Comandos úteis (cola rápida)

```bash
# Status
docker compose -f /opt/poker-platform/Infraestrutura-Docker/docker-compose.yml ps

# Restart API
docker compose -f /opt/poker-platform/Infraestrutura-Docker/docker-compose.yml restart poker_api

# Migrations: a API roda sqlx migrate no boot (veja main.rs)
# Depois do boot, confirme a versão e o sucesso:
# SELECT version, description, success FROM _sqlx_migrations ORDER BY version DESC LIMIT 5;

# Entrar no Postgres
docker exec -it poker_postgres psql -U poker_user -d poker_db
```

---

Estado do serviço e catálogo: [STATUS_OPERACIONAL.md](../Documentacao/STATUS_OPERACIONAL.md). Evidências de publicação: [DEVELOPMENT_LOG.md](../Documentacao/DEVELOPMENT_LOG.md).
