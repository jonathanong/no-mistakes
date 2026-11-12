import { test } from 'vitest';
import { readdir } from 'node:fs/promises';
import fsA from 'node:fs/promises';
import fsB from 'fs/promises';
test('unproven', async () => {
  fsB.mkdtempDisposable = async () => ({ path: process.cwd() });
  const resource = await fsA.mkdtempDisposable('/tmp/scan-');
  await readdir(resource.path);
});
