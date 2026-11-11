import { test } from 'vitest';
import { readdir } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';

const key = 'mkdtemp';
require('fs').promises[key] = () => process.cwd();
const create = require('node:fs/promises').mkdtemp;
test('an unknown direct promises member write invalidates the factory', async () => {
  await readdir(await create(join(tmpdir(), 'scan-')));
});
