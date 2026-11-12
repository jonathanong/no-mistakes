import { test } from 'vitest';
import { readdir } from 'node:fs/promises';
import fsA from 'node:fs/promises';
import fsB from 'fs/promises';
test('owned', async () => {
  const first = await fsA.mkdtempDisposable('/tmp/scan-');
  const second = await fsB.mkdtempDisposable('/tmp/scan-');
  await readdir(first.path);
  await readdir(second.path);
});
