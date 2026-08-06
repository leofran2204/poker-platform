# E-mail real de verificação — Resend

Com `EMAIL_PROVIDER=resend`, o código de 6 dígitos vai para a **caixa de entrada** do jogador (não só nos logs).

**Estado (2026-08-05):** domínio `zerotiltpoker.net` **verified** no Resend (região `sa-east-1`); DNS na **Hostinger**; API na VPS com `email_provider=resend`.

> Caixa webmail `admin@zerotiltpoker.net` (envio/recebimento humano) **não** é o mesmo caminho que o Resend. O app usa a **API Resend**; SMTP da caixa Hostinger **não** está implementado (`EMAIL_PROVIDER=smtp` ainda não existe).

## 1. Conta Resend

1. Crie conta em [https://resend.com](https://resend.com)
2. **API Keys** → Create API Key → copie `re_...`
3. **(Recomendado para amigos)** Domains → Add `zerotiltpoker.net` (região ex.: São Paulo `sa-east-1`)
   - Configure DNS (DKIM / SPF / MX de envio) que o painel mostrar
   - Aguarde status **Verified**
4. Sem domínio verificado: só pode usar `onboarding@resend.dev` e o Resend **só entrega no e-mail da sua conta Resend** (não no Gmail de amigos).

### DNS típico (Hostinger / zone editor)

Valores **exatos** vêm do painel Resend (use Copy). Referência:

| Tipo | Nome (host) | Conteúdo (exemplo) |
|------|-------------|--------------------|
| **TXT** (DKIM) | `resend._domainkey` | `p=MIGf...` (inteiro; sem cortar) |
| **MX** (envio) | `send` | `feedback-smtp.sa-east-1.amazonses.com` — prioridade **10** (campo separado) |
| **TXT** (SPF) | `send` | `v=spf1 include:amazonses.com ~all` (**com espaços**) |

**Erros comuns Hostinger:**

- MX com espaço no host (`sa- east-1`) → “MX record content is not valid”
- SPF sem espaços (`v=spf1include:...`) → SPF inválido; Resend pode falhar
- Nome `resend._domainkey.zerotiltpoker.net` quando o painel já completa o domínio → use só `resend._domainkey`
- **Não** apague o MX da **raiz** (`@`) do webmail `admin@` — o MX do Resend é só no subdomínio `send`

Conferência no PC:

```powershell
nslookup -type=TXT resend._domainkey.zerotiltpoker.net
nslookup -type=TXT send.zerotiltpoker.net
nslookup -type=MX send.zerotiltpoker.net
```

Status `failed` no Resend costuma ser “All required records are missing” — corrija DNS e **Restart verification** (ou delete/recreate o domínio no Resend).

## 2. Configurar a VPS (`.env`)

```bash
cd /opt/poker-platform/Infraestrutura-Docker
nano .env
```

Adicione/ajuste:

```env
REQUIRE_EMAIL_VERIFICATION=true
EMAIL_PROVIDER=resend
RESEND_API_KEY=re_xxxxxxxx
# Com domínio verificado:
EMAIL_FROM=Zero Tilt Poker <noreply@zerotiltpoker.net>
# Só teste próprio (sem domínio):
# EMAIL_FROM=Zero Tilt Poker <onboarding@resend.dev>
```

## 3. Subir a API com o código Resend

```bash
cd /opt/poker-platform
git pull origin master
cd Infraestrutura-Docker
export DOCKER_BUILDKIT=1
docker compose build poker_api
docker compose up -d poker_api
docker logs poker_api 2>&1 | grep -iE 'Auth policy|email|resend' | tail -10
```

Esperado no boot:

```text
Auth policy loaded require_email_verification=true email_provider=resend
```

Confirme envs no container (não cole a chave em chats):

```bash
docker exec poker_api printenv | grep -E 'EMAIL_|RESEND|REQUIRE_EMAIL'
```

## 4. Testar

1. Registrar em https://zerotiltpoker.net com um e-mail real  
2. Abrir inbox + spam  
3. Código de 6 dígitos no e-mail Zero Tilt  
4. Se falhar, logs:  
   `docker logs poker_api 2>&1 | grep -i resend | tail -20`  
   (em falha, o sistema ainda cai no **log** com o código)

## Segurança

- Nunca commite `RESEND_API_KEY`  
- `chmod 600 .env`  
- Em staging/demo, use domínio verificado + `EMAIL_FROM` do seu domínio  
