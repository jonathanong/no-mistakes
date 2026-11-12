import { test } from 'vitest';
import { readdirSync } from 'node:fs';
globalThis.mkdtempDisposableSync = () => ({ path: process.cwd() });
test('unproven', () => {
  readdirSync(mkdtempDisposableSync('/tmp/scan-').path);
});
