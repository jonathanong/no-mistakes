import { test } from 'vitest';
import { readdir } from 'node:fs/promises';

const fs = require('fs');
fs.promises = { mkdtemp: async () => process.cwd() };
test('replacing the promises view invalidates its factory', async () => {
  await readdir(await fs.promises.mkdtemp('scan-'));
});
