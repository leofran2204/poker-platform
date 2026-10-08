# Guia de Validação de Deploy & Orquestração

Procedimento de validação da plataforma em **Docker Compose**, **demo residencial (Cloudflare Tunnel HTTPS)** e **Kubernetes** (1 réplica).

**Domínio demo:** `zerotiltpoker.net`  
**Não é certificação de produção.** Na demo VPS, o DePix está reconciliado conforme o `STATUS_OPERACIONAL.md`; laboratório e tunnel usam o modo configurado no `.env` e permanecem mock por padrão.

Última publicação conferida: **08/10/2026, 13:48:04 UTC**, frontend `75971a0c`: 30 vídeos regravados com guias fonéticas para os termos ingleses, legendas/capítulos sincronizados e URLs de mídia versionadas. API mantida na imagem de `4c9541a3` (06/10); API, PostgreSQL e Redis conservaram containers, imagens e horário de início. Quatro serviços saudáveis, 60 migrations e 90 arquivos de mídia íntegros. Ambos os workflows aprovados antes da troca, backup validado e `.env` preservado. [Evidências e limite da revisão auditiva](../Documentacao/QUALITY.md#pronúncia-dos-vídeos--08102026). Documentação posterior não exige reconstruir essas imagens.

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

Antes de subir a API em `ENVIRONMENT=production`, configure `JWT_SECRET`, `EMAIL_CODE_PEPPER` e `KYC_DATA_PEPPER` com valores aleatórios independentes de pelo menos 32 bytes. Depois do boot, confirme `Migrations applied` nos logs e valide `_sqlx_migrations`; a release S26 exige a migration `060` com `success = true` e checksum SHA-384 idêntico ao arquivo versionado.

### Publicação S26 — Academy e Agente ZT

1. Conferir branch/commit, alterações locais na VPS e numeração/checksums das migrations. Não alterar SQL já aplicado.
2. Criar e conferir backup PostgreSQL e dos volumes Caddy fora da árvore versionada, com acesso restrito, antes do primeiro boot da API nova. Preservar o `.env` do servidor.
3. Executar `REBUILD_API=1 bash scripts/vps-redeploy-frontend.sh` a partir da raiz do clone. O script exige fast-forward, ajusta os dois volumes Caddy para uid 10001 (migração do frontend antigo root) e exige containers saudáveis e resposta da API por HTTPS. O ajuste usa um container transitório; o servidor permanece sem privilégios extras.
4. Conferir os quatro containers, `/health`, `/caddy-health`, catálogo público e migration 060. O boot não ativa agentes, não fecha ciclos e não faz backfill de vínculos.
5. Conferir `/curso`, a aula histórica `m0l2`, vídeo/legenda/poster atuais e `POST /api/academy/play` com cenário didático. Comparar o SHA-256 da mídia com o manifesto editorial; o laboratório não movimenta carteiras.
6. Registrar commit, horário e resultado em `DEVELOPMENT_LOG.md` e `QUALITY.md`. Falha em API, migração ou conteúdo servido impede declarar deploy concluído.

Recuar a imagem requer avaliar compatibilidade com o schema expandido. Restaurar backup elimina dados posteriores ao dump; não executar restauração automática.

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
