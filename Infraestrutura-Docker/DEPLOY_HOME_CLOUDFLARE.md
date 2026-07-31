# Demo em casa + Cloudflare Tunnel — **HTTPS** (zerotiltpoker.net)

**Objetivo:** `https://zerotiltpoker.net` no browser, com **TLS de ponta a ponta**.  
**Sem VPS.** PC ligado + Docker + `cloudflared`.

---

## HTTPS: o que é seguro (e o que não é “HTTP na cara do usuário”)

| Trecho | Protocolo | Quem protege |
|--------|-----------|--------------|
| **Seu browser → Cloudflare** | **HTTPS** (cadeado) | Certificado público Free da Cloudflare |
| **Cloudflare → cloudflared** | Canal **cifrado** do Tunnel | Cloudflare One / cloudflared |
| **cloudflared → Caddy no PC** | **HTTPS** na porta 443 | **Origin CA** (este guia) |

Ou seja: o visitante **sempre** acessa por **HTTPS**.  
O modo antigo “Flexible + origem HTTP” também mostrava cadeado no browser, mas a última milha no PC era HTTP. **Este guia usa origem HTTPS** — mais alinhado ao que você pediu.

```text
Browser --HTTPS--> Cloudflare --túnel cifrado--> cloudflared --HTTPS--> Caddy :443
                         ▲                                    ▲
                    cert público                         Origin CA
```

---

## Arquivos

| Arquivo | Função |
|---------|--------|
| `Caddyfile.tunnel` | Caddy com **TLS** + rotas API/WS/SPA |
| `Caddyfile.tunnel-http-only` | Fallback HTTP na origem (não recomendado) |
| `docker-compose.tunnel.yml` | Monta Caddyfile + `certs/` |
| `certs/origin.pem` + `origin-key.pem` | Você gera no painel (não commitados) |
| `.env.tunnel.example` | `CORS_ORIGINS=https://zerotiltpoker.net` |

---

## 1. Domínio na Cloudflare

1. Conta em [dash.cloudflare.com](https://dash.cloudflare.com)  
2. **Add site** → `zerotiltpoker.net` (plano Free)  
3. No **registrador**, coloque os **nameservers** que a Cloudflare mostrar  
4. Espere o site ficar **Active**

---

## 2. Certificado de origem (HTTPS no PC)

1. Domínio → **SSL/TLS** → **Origin Server** → **Create Certificate**  
2. Hostnames: `zerotiltpoker.net`, `*.zerotiltpoker.net` (ou `www.zerotiltpoker.net`)  
3. Crie e baixe/cole:

```text
Infraestrutura-Docker/certs/origin.pem       ← certificado
Infraestrutura-Docker/certs/origin-key.pem   ← chave privada
```

4. **SSL/TLS** → Overview → modo **Full (strict)**

Detalhes: `certs/README.md`.

---

## 3. Stack Docker no PC

Docker Desktop instalado. PowerShell:

```powershell
cd C:\Users\leofr\Projetos\Poker_Project\Infraestrutura-Docker

copy .env.tunnel.example .env
# Edite JWT_SECRET (e senhas). CORS já é https://zerotiltpoker.net

# Confirme que os certs existem:
dir certs\origin.pem, certs\origin-key.pem

docker compose -f docker-compose.yml -f docker-compose.tunnel.yml up -d --build
```

Teste local (pode avisar de certificado — Origin CA não é “público” no browser local):

```powershell
curl -k https://127.0.0.1/caddy-health
```

---

## 4. Cloudflare Tunnel

### Instalar

```powershell
winget install Cloudflare.cloudflared
```

### Login + tunnel

```powershell
cloudflared tunnel login
cloudflared tunnel create zerotilt-poker
```

### `%USERPROFILE%\.cloudflared\config.yml`

```yaml
tunnel: TROCAR_UUID
credentials-file: C:\Users\SEU_USUARIO\.cloudflared\TROCAR_UUID.json

ingress:
  - hostname: zerotiltpoker.net
    service: https://127.0.0.1:443
    originRequest:
      # Origin CA da Cloudflare é confiável no ecossistema CF;
      # se o cloudflared reclamar do cert local, use noTLSVerify: true
      # (o tráfego browser→CF continua HTTPS; origem ainda é TLS).
      noTLSVerify: true
  - hostname: www.zerotiltpoker.net
    service: https://127.0.0.1:443
    originRequest:
      noTLSVerify: true
  - service: http_status:404
```

> `noTLSVerify: true` evita dor de cabeça com hostname `127.0.0.1` vs cert emitido para `zerotiltpoker.net`. A conexão local **ainda é TLS**; só desliga a verificação do nome no cliente do tunnel. Para máxima rigidez depois, aponte o service para `https://zerotiltpoker.net:443` com hosts file — opcional.

### DNS

```powershell
cloudflared tunnel route dns zerotilt-poker zerotiltpoker.net
cloudflared tunnel route dns zerotilt-poker www.zerotiltpoker.net
```

### Rodar

```powershell
cloudflared tunnel run zerotilt-poker
```

---

## 5. Validar (HTTPS de verdade)

No browser (cadeado):

- https://zerotiltpoker.net/caddy-health  
- https://zerotiltpoker.net/login  

```powershell
curl -fsS https://zerotiltpoker.net/caddy-health
```

---

## 6. Checklist de segurança (demo)

- [x] Usuário final só fala **HTTPS** com a Cloudflare  
- [x] Origem no PC em **HTTPS** (Origin CA)  
- [x] Tunnel cifrado (não precisa abrir 80/443 no roteador)  
- [ ] `JWT_SECRET` forte no `.env`  
- [ ] PIX **mock**  
- [ ] PC sem suspensão enquanto a demo estiver no ar  
- [ ] Não commitar `certs/*.pem`  

Isso **não** substitui VPS/produção (IP residencial, PC desliga, etc.).

---

## Problemas comuns

| Sintoma | O que fazer |
|---------|-------------|
| Caddy não sobe | Faltam `certs/origin.pem` e `origin-key.pem` |
| 502 no domínio | Docker parado ou `cloudflared` parado |
| Cert error no browser | Domínio não Active / SSL mode errado (use Full strict) |
| CORS | `CORS_ORIGINS=https://zerotiltpoker.net` |
| WS falha | Confirme wss via mesmo host; tunnel up |

---

## Resumo

**Sim: tem que ser HTTPS para o usuário — e com este guia é.**  
Cadeado no browser + TLS na origem + tunnel cifrado.  
Guia completo de cliques: este arquivo + `certs/README.md`.
