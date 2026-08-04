# Guia de Validação de Deploy & Orquestração

Procedimento de validação da plataforma em **Docker Compose**, **demo residencial (Cloudflare Tunnel HTTPS)** e **Kubernetes** (1 réplica).

**Domínio demo:** `zerotiltpoker.net`  
**Não é certificação de produção.** PIX permanece mock/sandbox.

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

---

## ☸️ 2. Orquestração em Kubernetes (StatefulSets & Session Affinity)

O manifesto [k8s-statefulset.yaml](file:///c:/Users/leofr/Projetos/Poker_Project/Infraestrutura-Docker/k8s-statefulset.yaml) assegura o isolamento de estado por mesa (*Sticky Session Affinity*):

```bash
# Criar namespace de produção
kubectl create namespace poker-platform

# Aplicar o StatefulSet e o Service com ClientIP affinity
kubectl apply -f k8s-statefulset.yaml

# Verificar pods do StatefulSet em execução
kubectl get statefulsets -n poker-platform
kubectl get pods -n poker-platform -o wide
```

---

## 🔒 3. Hardening de Segurança OWASP Container Security

Todos os containers da plataforma atendem aos requisitos estritos de segurança OWASP:
- **Usuário Sem Privilégios (`user: 10001:10001`)**: Nenhum container roda como `root`.
- **Sistema de Arquivos Read-Only (`read_only: true`)**: Impede alteração maliciosa de binários em tempo de execução.
- **Remoção de Linux Capabilities (`cap_drop: - ALL`)**: Elimina permissões administrativas do kernel.
- **Prevenção de Escalada de Privilégios (`no-new-privileges:true`)**: Bloqueia chamadas `setuid`.

---

## 📊 4. SLA de Produção & Latência Sub-Milissegundo

| Métrica | SLA de Produção | Desempenho Medido |
| :--- | :--- | :--- |
| **Avaliação de Mão 7-Cards** | $< 50 \, \mu s$ | **11,916 $\mu s$** |
| **Throughput WebSocket** | $> 50.000$ msgs/s | **376.891 msgs/s** |
| **Integridade do Ledger** | SHA-256 Chain | **100% Válido** |
