# Estrategia do bot Zero Tilt

Baselines simplificados e auditaveis para cash 6-max/9-max, MTT ChipEV/ICM/PKO,
pos-flop heuristico, ICM e BRM. Base: `poker.txt` + pesquisa (GTO Wizard, BBZ,
RangeConverter, GTOLab, PokerCoaching, ThinkGTO).

## O que e / nao e

- E: ponto de partida deterministico para o bot (`decideAction(state)`).
- NAO e: saida exata de solver. Ranges percentuais usam ordem de forca
  simplificada; tabelas ICM sao baselines. Para producao, gerar lookups via
  ICMIZER/HRC/GTO Wizard e substituir os arquivos correspondentes.

## Uso

```ts
import { decideAction } from './strategy/index.js';

const out = decideAction({
  gameType: 'cash_6max',
  street: 'preflop',
  position: 'BTN',
  stackBB: 100,
  effectiveStackBB: 100,
  potBB: 1.5,
  betToCallBB: 0,
  myHand: ['As', 'Ks'],
  board: [],
  actionHistory: [],
  opponents: [],
});
// { action: 'raise', sizeBB: 2.5, reasoning: 'RFI BTN 6-max' }
```

Nos scripts de carga (`scripts/*.mjs`), substituir o bloco `Math.random()`
por uma chamada a `decideAction()` com o `GameState` montado a partir do
`table_state` do WebSocket.

## Checagem

```bash
cd bot
npm install
npm run check
```
