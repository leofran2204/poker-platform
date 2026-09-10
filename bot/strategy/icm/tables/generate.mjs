// Gera bot/strategy/icm/tables/*.json a partir do codigo-fonte TS.
// Fonte unica: as tabelas versionadas vivem em final_table.ts / bubble.ts;
// este script so serializa (notacao + 169 maos + combos aproximados).
// Para trocar por saidas de solver (ICMIZER/HRC/GTO Wizard), substitua os
// arquivos .ts de origem e rode de novo: `node strategy/icm/tables/generate.mjs`
// a partir de bot/ (depois de `tsc`).

import { writeFileSync, mkdirSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
mkdirSync(here, { recursive: true });

const dist = await import('../../../dist/strategy/index.js');
const { SHOVE_LATE, SHOVE_EARLY, CALL_VS_SHOVE, CALL_VS_SHOVE_DEEP, parseRange, countCombosHands } = dist;

function entry(notation) {
  const range = parseRange(notation);
  return {
    notation,
    hands169: [...range].sort(),
    combos1326: countCombosHands(range),
  };
}

const pushfold = { _meta: meta('push/fold mesa final 9-handed, baseline ICM medio') };
for (const depth of Object.keys(SHOVE_LATE).map(Number).sort((a, b) => a - b)) {
  pushfold[`${depth}bb`] = {
    late_BTN_SB_CO: entry(SHOVE_LATE[depth]),
    early: entry(SHOVE_EARLY[depth]),
  };
}
pushfold.call_vs_shove = {
  short: entry(CALL_VS_SHOVE),
  deep15bbPlus: entry(CALL_VS_SHOVE_DEEP),
};

const bubble = {
  _meta: meta('bolha: encolhimento do range de open por pressao ICM'),
  tightening: {
    midStack: 0.55,
    chipLeader: 1.15,
    shortStack: 0.9,
    other: 0.8,
    outOfBubble_gt10_from_money: 1.0,
  },
  example_MP: dist.BUBBLE_OPEN_EXAMPLE ?? null,
};

function meta(desc) {
  return {
    description: desc,
    source: 'bot/strategy/icm/final_table.ts + bubble.ts (baselines simplificados, nao solver)',
    generated_at: new Date().toISOString(),
    solver_replacement: 'Substitua os .ts de origem por lookups ICMIZER/HRC/GTO Wizard e rode generate.mjs de novo.',
  };
}

writeFileSync(join(here, 'pushfold_ft.json'), JSON.stringify(pushfold, null, 2) + '\n');
writeFileSync(join(here, 'bubble.json'), JSON.stringify(bubble, null, 2) + '\n');

// Versao Markdown (convencao do repo: .md sempre que possivel).
// O .json fica como lookup de maquina para o futuro porte ao bots.rs.
const md = [];
md.push('# Tabelas ICM — Mesa Final (baseline, nao solver)');
md.push('');
md.push(`Fonte: bot/strategy/icm/final_table.ts + bubble.ts. Gerado em ${new Date().toISOString().slice(0, 10)}.`);
md.push('Para trocar por saidas ICMIZER/HRC/GTO Wizard: substitua os .ts de origem e rode generate.mjs de novo.');
md.push('');
md.push('## Push/fold por stack (9-handed, ICM medio)');
md.push('');
md.push('| Stack | Late (BTN/SB/CO) | Early | Combos late (1326) |');
md.push('|---|---|---|---|');
for (const depth of Object.keys(SHOVE_LATE).map(Number).sort((a, b) => a - b)) {
  md.push(`| ${depth}bb | \`${SHOVE_LATE[depth]}\` | \`${SHOVE_EARLY[depth]}\` | ${pushfold[`${depth}bb`].late_BTN_SB_CO.combos1326} |`);
}
md.push('');
md.push('## Call vs shove (ICM)');
md.push('');
md.push(`- Curto: \`${CALL_VS_SHOVE}\``);
md.push(`- Profundo (15bb+): \`${CALL_VS_SHOVE_DEEP}\``);
md.push('');
md.push('## Bolha: encolhimento do open');
md.push('');
md.push('| Situacao | Fator (x range ChipEV) |');
md.push('|---|---|');
md.push(`| Mid stack | ${bubble.tightening.midStack} |`);
md.push(`| Chip leader | ${bubble.tightening.chipLeader} |`);
md.push(`| Short stack | ${bubble.tightening.shortStack} |`);
md.push(`| Outros | ${bubble.tightening.other} |`);
md.push(`| Longe da bolha (10+ do dinheiro) | ${bubble.tightening.outOfBubble_gt10_from_money} |`);
md.push('');
if (bubble.example_MP?.note) md.push(`Exemplo MP: ${bubble.example_MP.note}`);
md.push('');
writeFileSync(join(here, 'pushfold_ft.md'), md.join('\n'));
console.log('OK tables/pushfold_ft.json + tables/bubble.json + tables/pushfold_ft.md');
