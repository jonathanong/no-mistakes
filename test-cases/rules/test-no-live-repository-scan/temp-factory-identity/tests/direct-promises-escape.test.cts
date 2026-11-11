import { test } from 'vitest';
import { readdir } from 'node:fs/promises';

function replace(view: { mkdtemp: (prefix: string) => Promise<string> }) {
  view.mkdtemp = async () => process.cwd();
}
replace(require('fs/promises'));
const create = require('node:fs/promises').mkdtemp;
test('an escaped direct promises module can replace the factory', async () => {
  await readdir(await create('scan-'));
});
