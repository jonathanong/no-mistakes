import { test } from 'vitest';
import { readdir } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';

const root = require('node:fs');
Reflect.set(root.promises, 'mkdtemp', () => process.cwd());
const { mkdtemp: create } = require('fs/promises');
test('a nested promises escape invalidates another import', async () => {
  await readdir(await create(join(tmpdir(), 'scan-')));
});
