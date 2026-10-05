// A narrowly scoped, expiring build-only exception; see Documentacao/QUALITY.md.
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

export const advisory = 'https://github.com/advisories/GHSA-vfj7-8cjw-p6xm';
const expires = Date.parse('2026-10-19T00:00:00Z');
const buildPackages = new Set(['braces', 'chokidar', 'micromatch', 'fast-glob', 'tailwindcss']);

export function assess(report, lock, { production = false, now = Date.now(), content = [] } = {}) {
  if (report?.auditReportVersion !== 2 || report.error || !report.vulnerabilities || !report.metadata?.vulnerabilities) {
    throw new Error('Missing or invalid npm audit report');
  }
  const findings = report.vulnerabilities;
  const allowed = (name, visiting = new Set()) => {
    const finding = findings[name];
    if (!finding || production || now >= expires || !buildPackages.has(name) || visiting.has(name)) return false;
    if (finding.severity !== 'high' || !finding.nodes?.length || !finding.via?.length) return false;
    if (!finding.nodes.every(node => lock.packages?.[node]?.dev === true)) return false;
    if (lock.packages?.['node_modules/braces']?.version !== '3.0.3') return false;
    // Only reviewed, repository-controlled source globs reach braces in this build.
    if (JSON.stringify(content) !== JSON.stringify(['./index.html', './src/**/*.{js,ts,jsx,tsx}'])) return false;
    const next = new Set([...visiting, name]);
    return finding.via.every(via => typeof via === 'string' ? allowed(via, next) :
      name === 'braces' && via.name === 'braces' && via.url === advisory && via.severity === 'high');
  };
  const blocked = [], excepted = [];
  for (const [name, finding] of Object.entries(findings)) {
    if (!['high', 'critical'].includes(finding.severity)) continue;
    (allowed(name) ? excepted : blocked).push(name);
  }
  if (blocked.length) throw new Error(`High/critical vulnerabilities blocked: ${blocked.join(', ')}`);
  return excepted;
}

async function main() {
  const frontend = fileURLToPath(new URL('../Frontend-Web/', import.meta.url));
  const lock = JSON.parse(fs.readFileSync(path.join(frontend, 'package-lock.json'), 'utf8'));
  const { default: config } = await import(pathToFileURL(path.join(frontend, 'tailwind.config.js')));
  const cli = process.env.npm_execpath;
  if (!cli) throw new Error('Run through npm run audit:security');
  for (const production of [true, false]) {
    const audit = spawnSync(process.execPath, [cli, 'audit', '--json', ...(production ? ['--omit=dev'] : [])], {
      cwd: frontend, encoding: 'utf8', timeout: 120000, windowsHide: true,
    });
    if (audit.error || ![0, 1].includes(audit.status)) throw new Error('npm audit failed to run');
    const excepted = assess(JSON.parse(audit.stdout), lock, { production, content: config.content });
    console.log(`${production ? 'Production' : 'Complete'} dependency audit passed; build-only exceptions: ${excepted.join(', ') || 'none'}`);
    if (excepted.length) console.log(`${advisory} expires 2026-10-19; no patched braces release on 2026-10-05.`);
  }
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().catch(error => { console.error(error.message); process.exitCode = 1; });
}
