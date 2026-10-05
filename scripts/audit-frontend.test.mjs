import { test } from 'node:test';
import assert from 'node:assert/strict';
import { assess, advisory } from './audit-frontend.mjs';
const options = { now: Date.parse('2026-10-05'), content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'] };
const lock = { packages: { 'node_modules/braces': { dev: true, version: '3.0.3' }, 'node_modules/tailwindcss': { dev: true } } };
const fixture = () => ({ auditReportVersion: 2, metadata: { vulnerabilities: { high: 2 } }, vulnerabilities: {
  braces: { severity: 'high', nodes: ['node_modules/braces'], via: [{ name: 'braces', url: advisory, severity: 'high' }] },
  tailwindcss: { severity: 'high', nodes: ['node_modules/tailwindcss'], via: ['braces'] },
} });
test('only the reviewed dev dependency chain is excepted', () => assert.deepEqual(assess(fixture(), lock, options), ['braces', 'tailwindcss']));
test('production and missing dev markers are blocked', () => {
  assert.throws(() => assess(fixture(), lock, { ...options, production: true }));
  assert.throws(() => assess(fixture(), { packages: {} }, options));
});
test('expired exception and changed source globs are blocked', () => {
  assert.throws(() => assess(fixture(), lock, { ...options, now: Date.parse('2026-10-19') }));
  assert.throws(() => assess(fixture(), lock, { ...options, content: ['user-input/**/*'] }));
});
test('new advisory and critical severity are blocked', () => {
  const report = fixture(); report.vulnerabilities.braces.via.push({ name: 'braces', url: 'https://github.com/advisories/new', severity: 'high' });
  assert.throws(() => assess(report, lock, options));
  const critical = fixture(); critical.vulnerabilities.braces.severity = 'critical';
  assert.throws(() => assess(critical, lock, options));
});
test('unknown dependency paths, cycles and invalid reports fail closed', () => {
  const unknown = fixture(); unknown.vulnerabilities.tailwindcss.via.push('missing');
  assert.throws(() => assess(unknown, lock, options));
  const cycle = fixture(); cycle.vulnerabilities.braces.via = ['tailwindcss'];
  assert.throws(() => assess(cycle, lock, options));
  for (const report of [{}, { error: 'network' }, null]) assert.throws(() => assess(report, lock, options));
});
