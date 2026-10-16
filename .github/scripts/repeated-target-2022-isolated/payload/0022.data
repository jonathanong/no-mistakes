// Untimed full prepared-analysis CJS/ESM parity on reviewed physical inputs.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { pathToFileURL } from 'node:url';
const require = createRequire(import.meta.url);
const [packageRoot, fixtureRoot, reportsJson] = process.argv.slice(2);
const reports = JSON.parse(reportsJson);
assert.ok(Array.isArray(reports) && reports.length > 1);
const cjs = require(packageRoot + '/index.js');
const esm = await import(pathToFileURL(packageRoot + '/index.mjs').href);
const pending = cjs.analyzeProject({root:fixtureRoot, reports});
assert.ok(pending instanceof Promise, 'CJS prepared API must be async');
const cjsResult = await pending;
const esmPending = esm.analyzeProject({root:fixtureRoot, reports});
assert.ok(esmPending instanceof Promise, 'ESM prepared API must be async');
const esmResult = await esmPending;
assert.deepEqual(cjsResult, esmResult, 'CJS/ESM full prepared reports differ');
assert.equal(cjsResult.reports.length, reports.length);
for (const [index, report] of reports.entries()) {
  const options = {root: fixtureRoot};
  if (report.type === 'effects') {
    Object.assign(options, {kind: report.kind, entry: report.entry});
  }
  const cjsSingle = cjs[report.type](options);
  const esmSingle = esm[report.type](options);
  assert.ok(cjsSingle instanceof Promise, `CJS ${report.type} must be async`);
  assert.ok(esmSingle instanceof Promise, `ESM ${report.type} must be async`);
  assert.deepEqual(await cjsSingle, cjsResult.reports[index].result);
  assert.deepEqual(await esmSingle, cjsResult.reports[index].result);
}
process.stdout.write(JSON.stringify(cjsResult) + '\n');
