# Render — não é o caminho canônico

A demo pública roda em **VPS + Caddy** ou **casa + Cloudflare Tunnel**.

- VPS: [`DEPLOY_HETZNER.md`](DEPLOY_HETZNER.md)
- Casa: [`DEPLOY_HOME_CLOUDFLARE.md`](DEPLOY_HOME_CLOUDFLARE.md)
- Índice: [`DEPLOYMENT_VALIDATION.md`](DEPLOYMENT_VALIDATION.md)

`render.yaml` neste diretório é residual. Não use este arquivo para escolher LLM nem para implantar Node.js — a API é `API-Axum/Dockerfile`, porta 3000.
