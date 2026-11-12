import { test } from 'vitest';
import { readdirSync } from 'node:fs';
function getFs() {
  return { mkdtempDisposableSync: () => ({ path: process.cwd() }) };
}
test('unproven', () => {
  readdirSync(getFs().mkdtempDisposableSync('/tmp/scan-').path);
});
