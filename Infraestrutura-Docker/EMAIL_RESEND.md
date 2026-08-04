# E-mail real de verificação — Resend

Com `EMAIL_PROVIDER=resend`, o código de 6 dígitos vai para a **caixa de entrada** do jogador (não só nos logs).

## 1. Conta Resend

1. Crie conta em [https://resend.com](https://resend.com)
2. **API Keys** → Create API Key → copie `re_...`
3. **(Recomendado para amigos)** Domains → Add `zerotiltpoker.net`  
   - Configure DNS (SPF/DKIM) que o painel mostrar  
   - Aguarde status **Verified**
4. Sem domínio verificado: só pode usar `onboarding@resend.dev` e o Resend **só entrega no e-mail da sua conta Resend** (não no Gmail de amigos).

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
EMAIL_LOG_CODE_ALWAYS=false
```

## 3. Subir a API com o código Resend

```bash
cd /opt/poker-platform
git pull origin master
cd Infraestrutura-Docker
export DOCKER_BUILDKIT=1
docker compose build poker_api
docker compose up -d poker_api
docker logs poker_api 2>&1 | grep -i 'Auth policy\|email' | tail -10
```

## 4. Testar

1. Registrar no site com um e-mail real  
2. Abrir Gmail (e spam)  
3. Código de 6 dígitos no e-mail Zero Tilt  
4. Se falhar, logs:  
   `docker logs poker_api 2>&1 | grep -i resend | tail -20`  
   (em falha, o sistema ainda cai no **log** com o código)

## Segurança

- Nunca commite `RESEND_API_KEY`  
- `chmod 600 .env`  
- Em produção, use domínio verificado + `EMAIL_FROM` do seu domínio  
