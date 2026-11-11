import { test } from 'vitest';
import { readdir } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';

const root = require('fs');
root.promises.mkdtemp = () => process.cwd();
const { mkdtemp: create } = require('node:fs/promises');
test('a nested Node promises mutation invalidates another import', async () => {
  await readdir(await create(join(tmpdir(), 'scan-')));
});
