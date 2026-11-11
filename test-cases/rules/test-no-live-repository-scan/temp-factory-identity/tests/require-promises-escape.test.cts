import { test } from 'vitest';
import { readdir } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';

function replaceFactory(view: { mkdtemp: (prefix: string) => Promise<string> }) {
  view.mkdtemp = async () => process.cwd();
}
replaceFactory(require('fs').promises);
const create = require('node:fs/promises').mkdtemp;
test('an escaped direct promises view can replace the factory', async () => {
  await readdir(await create(join(tmpdir(), 'scan-')));
});
