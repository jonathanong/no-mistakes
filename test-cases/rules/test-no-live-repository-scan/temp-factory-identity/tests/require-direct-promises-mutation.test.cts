import { test } from 'vitest';
import { readdir } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';

require('node:fs').promises.mkdtemp = () => process.cwd();
const create = require('fs/promises').mkdtemp;
test('a direct require promises write invalidates another binding', async () => {
  await readdir(await create(join(tmpdir(), 'scan-')));
  await readdir(await require('fs').promises.mkdtemp(join(tmpdir(), 'scan-')));
});
