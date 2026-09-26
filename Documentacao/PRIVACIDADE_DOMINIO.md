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

## Dados de conta, KYC e suporte

- Data de nascimento e nome legal são usados para maioridade e análise manual da conta.
- O CPF informado no KYC não é persistido em claro: o banco guarda HMAC com `KYC_DATA_PEPPER` exclusivo e somente os quatro últimos dígitos para conferência operacional.
- Patrocinadores da rede de dois níveis não recebem CPF, e-mail, data de nascimento, nome legal nem conteúdo de chamados.
- Chamados de suporte ficam vinculados à conta e não devem conter senha, códigos de verificação ou MFA.
- A proteção técnica descrita aqui não substitui política de retenção, base legal, canal do titular e revisão jurídica LGPD antes de operação certificada.

## Limites honestos

| Vetor | Status |
|-------|--------|
| WHOIS público | Em geral privado (Hostinger) |
| Nome PIX na Carteira | Visível a quem está logado e pede fichas |
| CPF de KYC no banco | Somente HMAC + últimos 4 dígitos; sem texto integral |
| Nome legal e nascimento | Restritos ao fluxo autenticado de KYC/admin |
| Certificate Transparency | Domínio listado em logs CT |
| IP do servidor | Descobrível por DNS |

## Hostinger — checklist

1. Domínios → `zerotiltpoker.net` → Privacy / WHOIS Protection **ativado**
2. Contato do domínio: e-mail de alias, não o pessoal se possível
3. Não publicar CPF/telefone/endereço em páginas públicas

<!-- DOCUMENTATION_SYNC:START -->
> **S25** (2026-09-26) — demo `zerotiltpoker.net` · sem certificação de produção · PIX automático ligado (DePix reconciliado).
> Fatos (catálogo, carteiras, limites): [`STATUS_OPERACIONAL.md`](STATUS_OPERACIONAL.md).
<!-- DOCUMENTATION_SYNC:END -->
