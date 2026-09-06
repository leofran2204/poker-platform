---
name: zerotilt-monitor
description: Use quando precisar verificar a saúde da demo Zero Tilt Poker (zerotiltpoker.net): health da stack, catálogo do lobby, presença online e torneios agendados. Base do piloto de monitoramento Hermes.
---

# Zero Tilt Monitor

Procedimento de verificação somente-leitura da demo em `https://zerotiltpoker.net`.

## 1. Saúde da stack

`GET /api/health` → esperado: API + frontend/Caddy + PostgreSQL + Redis **4/4 healthy**.

## 2. Catálogo do lobby (S21)

`GET /api/lobby/tables` → esperado, cada um em Play Money e Jogo Real:

| Mesa | Blinds | Max |
|---|---|---|
| Texas Hold'em NL | 25/25 | 9 |
| Texas Hold'em Short Deck | 25/50 | 8 |
| Omaha 4 Cartas (SD) | 50/50 | 5 |
| Ultimate Pineapple | 50/50 | 6 |

Mesas `OPEN` listam mesmo lotadas, sempre com o max visível.

## 3. Torneios

MTT Texas 9-max · Freeroll FT Short Deck 8-max · Omaha 5-max · Pineapple 6-max.
Início agendado **21:30 America/Sao_Paulo**, auto-start com **5+** inscritos.

## 4. Presença

`GET /api/presence/online` → contador de autenticados com heartbeat recente (TTL 90s).

## 5. Classificação

- **Incidente (reportar):** API fora do ar, stack não-healthy, catálogo divergente, MTT 21:30 em risco.
- **Ruído (não reportar fora do digest):** flutuação momentânea de presença, mesa cheia, latência isolada.
- **Saudável:** responda apenas `[SILENT]`.

Fonte da verdade em caso de dúvida: `Documentacao/STATUS_OPERACIONAL.json` no repo.
