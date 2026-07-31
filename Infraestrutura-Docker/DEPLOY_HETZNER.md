# Deploy Hetzner Cloud — Poker Platform

**Objetivo:** subir a stack Docker (Postgres + Redis + API Axum + Frontend/Caddy) em VPS **robusto e barato**.  
**Status do produto:** staging / demo. **Sem certificação de produção.** PIX continua mock/sandbox; mesas com dono único por processo.

> Fonte de stack: `Infraestrutura-Docker/docker-compose.yml`, `API-Axum/Dockerfile`, `Frontend-Dioxus/Dockerfile`, `Caddyfile`, `.env.example`.

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
- [ ] Domínio (ex.: `poker.seudominio.com`) apontando **A** para o IP da VM
- [ ] Repositório no GitHub (já: `poker-platform`)
- [ ] Segredos gerados (nunca commitar `.env`):
  - `JWT_SECRET` ≥ 32 bytes aleatórios
  - `POSTGRES_PASSWORD` forte
  - `PIX_PROVIDER=mock` / `PIX_MODE=mock` (padrão seguro)
- [ ] `CORS_ORIGINS=https://seu-dominio.com` (só HTTPS; a API **rejeita** origem sem HTTPS)

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

### 4.2 DNS

No provedor do domínio:

```text
A   poker.seudominio.com   →   IP_DA_VPS
```

TTL baixo (300s) na primeira vez. Espere propagar (`dig poker.seudominio.com`).

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
cp .env.example .env
# edite senhas, JWT, domínio e CORS — depois:
chmod 600 .env
```

Exemplo mínimo de staging (troque todos os `TROCAR_*`):

```bash
cat > .env <<'EOF'
DOMAIN_NAME=poker.seudominio.com

HOST=0.0.0.0
PORT=3000
ENVIRONMENT=development

POSTGRES_USER=poker_user
POSTGRES_PASSWORD=TROCAR_SENHA_FORTE_AQUI
POSTGRES_DB=poker_db
DATABASE_URL=postgres://poker_user:TROCAR_SENHA_FORTE_AQUI@postgres:5432/poker_db

REDIS_URL=redis://redis:6379
TRUST_PROXY_HEADERS=true

JWT_SECRET=TROCAR_MINIMO_32_BYTES_ALEATORIOS________________
CORS_ORIGINS=https://poker.seudominio.com

PIX_PROVIDER=mock
PIX_MODE=mock
PIX_WEBHOOK_SECRET=TROCAR_WEBHOOK_SECRET

DEFAULT_BIG_BLIND_CENTS=2000
DEFAULT_RAKE_BASIS_POINTS=500
DEFAULT_RAKE_CAP_CENTS=10000
LOSS_DEFLATOR_ENABLED=true
EOF

chmod 600 .env
```

**Invariantes:**

- `DATABASE_URL` deve usar o **mesmo** user/senha/db de `POSTGRES_*`.
- `CORS_ORIGINS` = origem HTTPS completa (`https://` + `DOMAIN_NAME`).
- `DOMAIN_NAME` **sem** `https://` (só o host que o Caddy escuta).
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
curl -fsS https://poker.seudominio.com/health
curl -fsS https://poker.seudominio.com/caddy-health   # se exposto no Caddyfile
curl -fsS https://poker.seudominio.com/api/lobby/tables
```

Navegador: `https://poker.seudominio.com/login`

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

**Resumo:** compre **Hetzner ~8 GB**, Ubuntu, Docker, clone o repo, `.env` com HTTPS no `CORS_ORIGINS`, `compose build && up -d`, DNS no domínio. Custo típico **~€10–16/mês** com backup. Ideal para **staging/demo**; não é selo de produção.
