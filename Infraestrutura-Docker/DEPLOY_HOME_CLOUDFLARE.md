# Demo em casa + Cloudflare Tunnel — zerotiltpoker.net

**Objetivo:** expor a stack Docker do PC em `https://zerotiltpoker.net` **sem VPS e sem abrir porta no roteador**.  
**TLS:** termina na **Cloudflare**. Em casa o Caddy fala só **HTTP** (`Caddyfile.tunnel`).

> Staging/demo. PIX mock. PC precisa ficar ligado e com internet.

---

## Arquivos deste modo

| Arquivo | Função |
|---------|--------|
| `Caddyfile.tunnel` | Caddy **HTTP only**, host `zerotiltpoker.net` → API + WASM |
| `docker-compose.tunnel.yml` | Monta o Caddyfile e publica `127.0.0.1:80` |
| `.env.tunnel.example` | CORS `https://zerotiltpoker.net` |

Compose normal (`Caddyfile` + LE) continua para VPS. **Não misture** os dois modos no mesmo `up` sem querer.

---

## 1. Conta Cloudflare + domínio

1. Crie conta em [dash.cloudflare.com](https://dash.cloudflare.com)  
2. **Add a site** → `zerotiltpoker.net` (plano **Free** basta)  
3. Cloudflare mostra **2 nameservers** (ex.: `ada.ns.cloudflare.com`)  
4. No **registrador** do domínio, troque os nameservers para os da Cloudflare  
5. Aguarde o status do site ficar **Active** (minutos a horas)

---

## 2. Subir a stack no PC (Docker Desktop)

Pré-requisitos: [Docker Desktop](https://www.docker.com/products/docker-desktop/) no Windows, WSL2 ok.

No PowerShell:

```powershell
cd C:\Users\leofr\Projetos\Poker_Project\Infraestrutura-Docker

copy .env.tunnel.example .env
# Edite .env: JWT_SECRET e senhas (não use defaults em demo “pública”)

docker compose -f docker-compose.yml -f docker-compose.tunnel.yml up -d --build
```

A **primeira build** (Rust + WASM) pode demorar bastante e usar muita RAM.

Teste **só na máquina** (HTTP local):

```powershell
curl http://127.0.0.1/caddy-health
# OK
```

```powershell
docker compose -f docker-compose.yml -f docker-compose.tunnel.yml ps
docker compose -f docker-compose.yml -f docker-compose.tunnel.yml logs -f poker_api
```

---

## 3. Cloudflare Tunnel (`cloudflared`)

### 3.1 Instalar no Windows

- Download: https://developers.cloudflare.com/cloudflare-one/connections/connect-apps/install-and-setup/installation/  
- Ou: `winget install Cloudflare.cloudflared`

### 3.2 Login e criar tunnel

```powershell
cloudflared tunnel login
# abre o browser; autorize o domínio zerotiltpoker.net

cloudflared tunnel create zerotilt-poker
# anote o Tunnel ID (UUID)
```

### 3.3 Config do tunnel

Crie (ajuste o caminho do credencial que o `tunnel create` imprimir):

`%USERPROFILE%\.cloudflared\config.yml`

```yaml
tunnel: TROCAR_PELO_UUID_DO_TUNNEL
credentials-file: C:\Users\leofr\.cloudflared\TROCAR_PELO_UUID.json

ingress:
  - hostname: zerotiltpoker.net
    service: http://127.0.0.1:80
  - hostname: www.zerotiltpoker.net
    service: http://127.0.0.1:80
  - service: http_status:404
```

### 3.4 DNS do tunnel (CNAME)

```powershell
cloudflared tunnel route dns zerotilt-poker zerotiltpoker.net
cloudflared tunnel route dns zerotilt-poker www.zerotiltpoker.net
```

Isso cria CNAME no DNS da Cloudflare apontando para o tunnel.

### 3.5 Rodar o tunnel

```powershell
cloudflared tunnel run zerotilt-poker
```

Deixe essa janela aberta (ou instale como serviço Windows depois).

---

## 4. SSL/TLS no painel Cloudflare

**SSL/TLS** → overview:

- Modo recomendado com origem HTTP: **Flexible**  
  (browser ↔ Cloudflare = HTTPS; Cloudflare ↔ seu PC = HTTP na 127.0.0.1)

Se no futuro a origem tiver HTTPS válido, use **Full**.

---

## 5. Validar

```text
https://zerotiltpoker.net/caddy-health   → OK
https://zerotiltpoker.net/health         → API
https://zerotiltpoker.net/login          → frontend
```

CORS e WebSocket usam o mesmo host (`https://` / `wss://` via Cloudflare).

---

## 6. Dia a dia

| Ação | Comando |
|------|---------|
| Subir stack | `docker compose -f docker-compose.yml -f docker-compose.tunnel.yml up -d` |
| Parar stack | `docker compose -f docker-compose.yml -f docker-compose.tunnel.yml down` |
| Tunnel | `cloudflared tunnel run zerotilt-poker` |
| Logs API | `docker compose ... logs -f poker_api` |

**PC:** desative suspensão automática enquanto a demo estiver no ar.

---

## 7. Problemas comuns

| Sintoma | Causa provável |
|---------|----------------|
| 502 no domínio | Docker parado ou tunnel não está rodando |
| SSL error | Domínio ainda não Active na CF / modo SSL errado |
| CORS no browser | `CORS_ORIGINS` sem `https://zerotiltpoker.net` |
| Build OOM | Feche apps; 8 GB+ RAM; ou faça build com menos serviços |
| WS cai | Tunnel ok? Reinicie `cloudflared`; teste `/caddy-health` |

---

## 8. Segurança (demo em casa)

- Não use senhas default se o link for público  
- PIX permanece **mock**  
- Tunnel **não** abre 80/443 no roteador (bom)  
- Qualquer um com a URL pode bater na sua máquina se o tunnel estiver up — trate como **demo**, não produção  

---

## Resumo

```text
Browser --HTTPS--> Cloudflare --HTTP--> cloudflared --HTTP--> Caddy :80 --> API + WASM
                         (TLS aqui)              (Caddyfile.tunnel)
```
