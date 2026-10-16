const { join } = require('node:path');
const { writeFileSync } = require('node:fs');
const api = require('/tmp/no-mistakes-2076-prospective2087/packages/no-mistakes/index.js');
(async () => {
  const checkout = '/tmp/no-mistakes-2076-prospective2087';
  const result = [];
  for (const [relative, entry] of [
    ['test-cases/rules/query-reached-per-item/executor-paths-fixture', 'entry.mts'],
    ['fixtures/benchmarks/parameter-annotation-cache/cold', 'entry.ts'],
    ['fixtures/benchmarks/parameter-annotation-cache/scale', 'entry.ts'],
  ]) {
    const root = join(checkout, relative);
    const reports = [{ type: 'effects', kind: 'postgres', entry }, { type: 'check' }];
    result.push(await api.analyzeProject({ root, reports }));
  }
  writeFileSync(process.argv[2], JSON.stringify(result));
})().catch(error => { console.error(error); process.exitCode = 1; });
