import { test } from 'vitest';
import { readdirSync } from 'node:fs';
import fsA from 'node:fs';
import fsB from 'fs';
test('owned', () => {
  const first = fsA.mkdtempDisposableSync('/tmp/scan-');
  const second = fsB.mkdtempDisposableSync('/tmp/scan-');
  readdirSync(first.path);
  readdirSync(second.path);
});
