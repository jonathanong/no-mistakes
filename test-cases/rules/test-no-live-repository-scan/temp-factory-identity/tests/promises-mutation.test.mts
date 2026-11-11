import { test } from 'vitest';
import { promises as fs } from 'node:fs';
import * as nodeFs from 'node:fs';
import { mkdtemp as create, readdir } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';

Reflect.set(fs, 'mkdtemp', () => process.cwd());
test('a mutated promises view invalidates the direct binding', async () => {
  await readdir(await fs.mkdtemp(join(tmpdir(), 'scan-')));
  await readdir(await create(join(tmpdir(), 'scan-')));
  await readdir(await nodeFs.promises.mkdtemp(join(tmpdir(), 'scan-')));
});
