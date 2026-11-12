import { test } from 'vitest';
import { readdir } from 'node:fs/promises';
import fsA from 'node:fs/promises';
import fsB from 'fs/promises';
declare function mutate(value: unknown): void;
test('unproven', async () => {
  mutate(fsB);
  const resource = await fsA.mkdtempDisposable('/tmp/scan-');
  await readdir(resource.path);
});
