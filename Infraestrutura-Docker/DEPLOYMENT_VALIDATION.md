# Guia de Validação de Deploy & Orquestração

Procedimento de validação da plataforma em **Docker Compose**, **demo residencial (Cloudflare Tunnel HTTPS)** e **Kubernetes** (1 réplica).

**Domínio demo:** `zerotiltpoker.net`  
**Não é certificação de produção.** Na demo VPS, o DePix está reconciliado conforme o `STATUS_OPERACIONAL.md`; laboratório e tunnel usam o modo configurado no `.env` e permanecem mock por padrão.

| Caminho | Guia | HTTPS |
|---------|------|--------|
| Casa + Tunnel | [DEPLOY_HOME_CLOUDFLARE.md](DEPLOY_HOME_CLOUDFLARE.md) | E2E (CF + Origin CA) |
| VPS | [DEPLOY_HETZNER.md](DEPLOY_HETZNER.md) | Caddy + Let's Encrypt |
| Lab | `docker compose up` | localhost / self-signed |

---

## 🐋 1. Orquestração em Docker Compose

O arquivo `docker-compose.yml` orquestra:
- **`postgres`**: PostgreSQL 15
- **`redis`**: Redis 7
- **`poker_api`**: API Axum / motor
- **`poker_frontend`**: Caddy + SPA TypeScript (`Frontend-Web` / Vite build)

> Kafka/Zookeeper **não** fazem parte do compose atual.

### Lab local
```bash
cd Infraestrutura-Docker
cp .env.example .env
docker compose up --build -d
```

### Demo casa (HTTPS tunnel)
```bash
cp .env.tunnel.example .env
# certs/origin.pem + origin-key.pem (ver certs/README.md)
docker compose -f docker-compose.yml -f docker-compose.tunnel.yml up --build -d
# + cloudflared tunnel run (ver DEPLOY_HOME_CLOUDFLARE.md)
```

### Validação de saúde
```bash
docker compose ps
docker compose logs -f poker_api
curl -fsS https://zerotiltpoker.net/caddy-health   # com tunnel/VPS no ar
curl -fsS https://zerotiltpoker.net/health
```

Antes de subir a API em `ENVIRONMENT=production`, configure `JWT_SECRET`, `EMAIL_CODE_PEPPER` e `KYC_DATA_PEPPER` com valores aleatórios independentes de pelo menos 32 bytes. Depois do boot, confirme `Migrations applied` nos logs e valide `_sqlx_migrations`; a release de proteção do jogador exige a migration `058` com `success = true`.

---

## ☸️ 2. Manifesto Kubernetes de referência

O manifesto [`k8s-statefulset.yaml`](k8s-statefulset.yaml) serve para validação com **uma réplica**. Afinidade por IP não implementa ownership distribuído de mesa e não autoriza escalar a API horizontalmente. Antes de usar múltiplas réplicas, é obrigatório definir roteamento/ownership dos atores e validar settlement HMAC entre processos.

```bash
# Criar namespace de laboratório
kubectl create namespace poker-platform

# Aplicar o StatefulSet e o Service com ClientIP affinity
kubectl apply -f k8s-statefulset.yaml

# Verificar pods do StatefulSet em execução
kubectl get statefulsets -n poker-platform
kubectl get pods -n poker-platform -o wide
```

---

## 🔒 3. Hardening de containers

O Compose aplica controles diferentes conforme a função do container:

- **API:** usuário `10001:10001`, filesystem read-only, `/tmp` em `tmpfs`, `cap_drop: ALL` e `no-new-privileges`.
- **Frontend/Caddy:** filesystem read-only, diretórios graváveis em `tmpfs`, `cap_drop: ALL`, apenas `NET_BIND_SERVICE` readicionada e `no-new-privileges`.
- **PostgreSQL/Redis:** imagens oficiais fixadas por digest, portas publicadas somente em loopback, volumes persistentes e `no-new-privileges`.

Esses controles reduzem a superfície de ataque, mas não constituem certificação OWASP ou auditoria externa.

---

## 📊 4. Evidência de capacidade

Não existe SLA de produção nem benchmark de release certificado neste repositório. Resultados históricos não devem ser apresentados como capacidade vigente.

Para gerar evidência reproduzível, use `Documentacao/FULL_VALIDATION.md` e `scripts/full-validation.*` somente com autorização explícita. Registre ambiente, commit, duração, carga e resultado; integridade financeira deve ser confirmada por testes/consultas, não por porcentagem declarativa.
