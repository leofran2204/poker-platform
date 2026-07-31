# Certificados de origem (Cloudflare Origin CA)

Coloque aqui (não vão para o Git):

| Arquivo | Conteúdo |
|---------|----------|
| `origin.pem` | Certificado Origin CA (Cloudflare) |
| `origin-key.pem` | Chave privada |

## Como gerar (HTTPS de ponta a ponta)

1. [dash.cloudflare.com](https://dash.cloudflare.com) → domínio **zerotiltpoker.net**
2. **SSL/TLS** → **Origin Server** → **Create Certificate**
3. Hostnames: `zerotiltpoker.net` e `www.zerotiltpoker.net` (e `localhost` se quiser testar local com o mesmo cert — opcional)
4. Validade: 15 years (padrão ok)
5. Copie o **Origin Certificate** → salve como `origin.pem`
6. Copie a **Private Key** → salve como `origin-key.pem`
7. No painel: **SSL/TLS** → overview → modo **Full (strict)**

No Windows (PowerShell), na pasta `Infraestrutura-Docker/certs`:

```powershell
# Cole o certificado e a chave nos arquivos (ou salve pelo editor)
notepad origin.pem
notepad origin-key.pem
```

Sem esses arquivos o container `poker_frontend` com `Caddyfile.tunnel` **não sobe**.
