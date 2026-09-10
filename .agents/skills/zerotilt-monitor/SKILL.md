---
name: zerotilt-monitor
description: Use quando precisar verificar a saúde da demo Zero Tilt Poker (zerotiltpoker.net): health da stack, catálogo do lobby, presença online e torneios agendados. Base do piloto de monitoramento Hermes.
---

# Zero Tilt Monitor

Procedimento de verificação somente-leitura da demo em `https://zerotiltpoker.net`.

## 1. Saúde da stack

`GET /api/health` → esperado: API + frontend/Caddy + PostgreSQL + Redis **4/4 healthy**.

## 2. Catálogo do lobby

`GET /api/lobby/tables` → esperado, cada um em Play Money e Jogo Real. Conferir blinds/cap contra `Documentacao/STATUS_OPERACIONAL.md` (não memorizar stakes).

Mesas `OPEN` listam mesmo lotadas, sempre com o max visível.

## 3. Torneios

Eventos, horário e auto-start: iguais a `Documentacao/STATUS_OPERACIONAL.md`.

## 4. Presença

`GET /api/presence/online` → contador de autenticados com heartbeat recente (TTL 90s).

## 5. Classificação

- **Incidente (reportar):** API fora do ar, stack não-healthy, catálogo divergente, MTT 21:30 em risco.
- **Ruído (não reportar fora do digest):** flutuação momentânea de presença, mesa cheia, latência isolada.
- **Saudável:** responda apenas `[SILENT]`.

Fonte da verdade em caso de dúvida: `Documentacao/STATUS_OPERACIONAL.md` (humanos) / `STATUS_OPERACIONAL.json` (números).
