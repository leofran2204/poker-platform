# Deploy Hetzner Cloud — Poker Platform

**Objetivo:** subir a stack Docker (Postgres + Redis + API Axum + Frontend/Caddy) em VPS **robusto e barato**.  
**Status do produto:** staging / demo. **Sem certificação de produção.** PIX continua mock/sandbox; mesas com dono único por processo.

**Domínio do produto:** [`zerotiltpoker.net`](https://zerotiltpoker.net)  
**Host público (staging/demo):** `zerotiltpoker.net` (apex) — Caddy + Let's Encrypt + reverse_proxy da API no mesmo host.

> **Sem cartão / sem VPS?** Use a demo em casa com HTTPS: [`DEPLOY_HOME_CLOUDFLARE.md`](DEPLOY_HOME_CLOUDFLARE.md) (Cloudflare Tunnel + Origin CA). Este guia é para **VPS 24/7** (Hetzner ou Ubuntu em provedor BR com PIX).

> Fonte de stack: `docker-compose.yml`, Dockerfiles API/Frontend, `Caddyfile` (LE), `.env.staging.example`.

---

## 1. Tamanho da VM e custo estimado

### Recomendado (staging confortável)

| Item | Valor |
|------|--------|
| **Provedor** | [Hetzner Cloud](https://www.hetzner.com/cloud) |
| **Plano alvo** | **CX32** ou **CPX31/CPX32** (confira nomes no console; busque **~4 vCPU · 8 GB RAM · ≥80 GB**) |
| **Região** | **Ashburn (US-East)** se latência for BR/US; **Falkenstein/Nuremberg (DE)** se preferir UE/preço |
| **SO** | **Ubuntu 24.04 LTS** (ou 22.04) x86_64 |
| **Custo VM** | ordem de grandeza **€6–12 / mês** (verifique preço atual no painel) |
| **Snapshot backup** | ~20% do volume (ex.: +€1–3/mês) — **recomendo ativar** |
| **IP flutuante** | opcional; se não usar, o IP da VM muda se recriar a instância |
| **Total estimado** | **~€8–15 / mês** (~R$ 50–90, câmbio variável) |

### Mínimo viável (apertado)

| Item | Valor |
|------|--------|
| Plano | **CX22 / CPX22** (~2 vCPU · **4 GB** · 40 GB) |
| Custo | ~**€4–8 / mês** |
| Risco | build Docker da API/Rust pode **estourar RAM**; prefira build em CI ou máquina local e só puxe imagens |

### O que **não** cabe bem

- 1–2 GB de RAM (estilo free tier mínimo) com Postgres + Redis + API + build na mesma máquina.
- Multireplica K8s: o projeto ainda é **1 processo dono das mesas** (`k8s-statefulset` = 1 réplica).

---

## 2. Ordem dos containers (igual ao compose)

```text
1. postgres     (volume postgres_data)
2. redis        (volume redis_data)
3. poker_api    (build Motor-Rust + API-Axum; depende de postgres+redis healthy)
4. poker_frontend (Caddy + WASM estático; publica 80/443; reverse_proxy → poker_api)
```

Portas públicas na VPS: **apenas 22, 80, 443**.  
Postgres e Redis ficam em `127.0.0.1` no compose (não exponha na internet).

---

## 3. Checklist pré-voo

- [ ] Conta Hetzner + cartão
- [ ] Domínio **zerotiltpoker.net** com registro **A** (e opcional **www**) para o IP da VM
- [ ] Repositório no GitHub (já: `poker-platform`)
- [ ] Segredos gerados (nunca commitar `.env`):
  - `JWT_SECRET` ≥ 32 bytes aleatórios
  - `POSTGRES_PASSWORD` forte
  - `PIX_PROVIDER=mock` / `PIX_MODE=mock` (padrão seguro)
- [ ] `CORS_ORIGINS=https://zerotiltpoker.net` (só HTTPS; a API **rejeita** origem sem HTTPS)

---

## 4. Passo a passo na VPS

### 4.1 Criar a Cloud Server

1. Hetzner Console → **New project** → **Add server**
2. Location: Ashburn ou DE  
3. Image: **Ubuntu 24.04**  
4. Type: **CX32 / ~8 GB** (recomendado)  
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

O `docker-compose.yml` já interpola `POSTGRES_*`, `DATABASE_URL`, `JWT_SECRET`, `CORS_ORIGINS`, `DOMAIN_NAME`, PIX e rake a partir do `.env`. Não é necessário editar o YAML na VPS.

```bash
cd /opt/poker-platform/Infraestrutura-Docker
cp .env.staging.example .env
# gere senhas/JWT (nunca use os placeholders TROCAR_*):
#   openssl rand -base64 48
# edite POSTGRES_PASSWORD, DATABASE_URL, JWT_SECRET, PIX_WEBHOOK_SECRET
chmod 600 .env
```

Template versionado: **`.env.staging.example`** (já com `DOMAIN_NAME=zerotiltpoker.net` e CORS HTTPS).

**Invariantes:**

- `DATABASE_URL` deve usar o **mesmo** user/senha/db de `POSTGRES_*`.
- `CORS_ORIGINS` = origem HTTPS completa (`https://zerotiltpoker.net`).
- `DOMAIN_NAME=zerotiltpoker.net` **sem** `https://`.
- Se servir também `www`, inclua no CORS: `https://zerotiltpoker.net,https://www.zerotiltpoker.net` e considere redirect www→apex no Caddy (opcional).
- PIX: mantenha `mock`; não use `PIX_MODE=production`.

### 4.6 Caddy / domínio

O `Caddyfile` do repo já usa `{$DOMAIN_NAME:localhost}`. O serviço `poker_frontend` recebe `DOMAIN_NAME` do compose — **não** é preciso colar o domínio no arquivo se o `.env` estiver certo.

Caddy pede certificado Let's Encrypt sozinho nas portas 80/443 quando `DOMAIN_NAME` é um hostname público com DNS apontando para a VPS.

### 4.7 Subir a stack (ordem correta)

```bash
cd /opt/poker-platform/Infraestrutura-Docker

# 1) Build (API Rust + Frontend WASM — demora na 1ª vez; use 8 GB de RAM)
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
| Atualizar código | `git pull` → `docker compose build` → `docker compose up -d` |
| Logs | `docker compose logs -f --tail=200` |
| Disco | `df -h`; limpar `docker system prune` com cuidado |

---

## 6. O que **não** fazer neste deploy

- `ENVIRONMENT=production` com `JWT_SECRET` fraco (a API recusa segredos conhecidos em production)
- `PIX_MODE=production` (código bloqueia PIX real)
- Expor Postgres/Redis na internet
- Esperar multi-pod de mesas (ainda **1 dono por processo**)
- Usar free tier Oracle **como se fosse** o mesmo que Hetzner (capacidade e ARM mudam o jogo)

---

## 7. Custo mensal resumido (ordem de grandeza)

| Item | € / mês |
|------|---------|
| VM ~8 GB | 8–12 |
| Backups | 1–3 |
| Domínio (anual/12) | ~1 |
| **Total** | **~€10–16** |

Sem tráfego massivo o transfer da Hetzner costuma bastar; confira a cota do plano.

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

# Entrar no Postgres
docker exec -it poker_postgres psql -U poker_user -d poker_db
```

---

**Resumo:** compre **Hetzner ~8 GB**, Ubuntu, Docker, clone o repo, DNS **A** de `zerotiltpoker.net` → IP da VPS, `.env` a partir de `.env.staging.example`, `compose build && up -d`. Custo típico **~€10–16/mês** com backup. Ideal para **staging/demo**; não é selo de produção.
