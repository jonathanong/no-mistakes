import { test } from 'vitest';
import { readdirSync } from 'node:fs';
import { readdir } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';

function shadowed(require: (name: string) => { mkdtempSync: (prefix: string) => string }) {
  return require('fs').mkdtempSync(join(tmpdir(), 'scan-'));
}
test('direct global Node require calls own their roots', async () => {
  readdirSync(require('fs').mkdtempSync(join(tmpdir(), 'scan-')));
  await readdir(await require('node:fs').promises.mkdtemp(join(tmpdir(), 'scan-')));
  await readdir(await require('fs/promises').mkdtemp(join(tmpdir(), 'scan-')));
});
test('wrong and shadowed require calls stay unproven', async () => {
  readdirSync(require('../wrong-fs.mjs').mkdtempSync(join(tmpdir(), 'scan-')));
  readdirSync(shadowed(() => ({ mkdtempSync: () => process.cwd() })));
  readdirSync(require('node:fs/promises').mkdtempSync(join(tmpdir(), 'scan-')));
  readdirSync(require('fs').other.mkdtempSync(join(tmpdir(), 'scan-')));
});
