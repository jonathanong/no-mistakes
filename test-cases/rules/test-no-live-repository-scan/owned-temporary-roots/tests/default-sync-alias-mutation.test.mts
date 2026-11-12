import { test } from 'vitest';
import { readdirSync } from 'node:fs';
import fsA from 'node:fs';
import fsB from 'fs';
test('unproven', () => {
  fsB.mkdtempDisposableSync = () => ({ path: process.cwd() });
  const resource = fsA.mkdtempDisposableSync('/tmp/scan-');
  readdirSync(resource.path);
});
