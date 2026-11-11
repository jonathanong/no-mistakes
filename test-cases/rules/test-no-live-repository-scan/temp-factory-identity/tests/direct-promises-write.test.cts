import { test } from 'vitest';
import { readdir } from 'node:fs/promises';

require('fs/promises').mkdtemp = async () => process.cwd();
const create = require('node:fs/promises').mkdtemp;
test('a direct promises write invalidates the bound factory', async () => {
  await readdir(await create('scan-'));
});
