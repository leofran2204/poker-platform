# Privacidade do domínio e do site

## WHOIS / RDAP (`zerotiltpoker.net`)

- Registrador: **Hostinger**
- Consulta pública (RDAP) em 2026-08-29: **titular mascarado** (nome vazio; contato só via formulário Hostinger)
- Confirme no painel Hostinger → Domínios → **WHOIS Privacy / Domain Privacy** = **ON**
- Se estiver OFF: ative e aguarde propagação RDAP (minutos a horas)

Não dá para esconder 100%: datas de registro, nameservers e IP do VPS continuam públicos.

## O que o site já faz

- `robots.txt` bloqueia `/admin`, `/wallet`, login/registro, `/api/`
- Caddy envia `X-Robots-Tag: noindex` nessas rotas
- SPA marca meta `noindex` em Carteira e Admin
- Chave PIX na Carteira fica **mascarada** até “Mostrar chave” / “Copiar”
- Nome do recebedor PIX permanece completo (necessário para o jogador conferir no banco)

## Limites honestos

| Vetor | Status |
|-------|--------|
| WHOIS público | Em geral privado (Hostinger) |
| Nome PIX na Carteira | Visível a quem está logado e pede fichas |
| Certificate Transparency | Domínio listado em logs CT |
| IP do servidor | Descobrível por DNS |

## Hostinger — checklist

1. Domínios → `zerotiltpoker.net` → Privacy / WHOIS Protection **ativado**
2. Contato do domínio: e-mail de alias, não o pessoal se possível
3. Não publicar CPF/telefone/endereço em páginas públicas
