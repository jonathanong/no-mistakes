import { test } from 'vitest';
import fs, { mkdtempSync, mkdtempSync as createSync, readdirSync } from 'node:fs';
import * as nodeFs from 'fs';
import { mkdtemp as createAsync, readdir } from 'node:fs/promises';
import * as promises from 'fs/promises';
import { mkdtempSync as impossibleSync } from 'node:fs/promises';
import { mkdtempSync as fakeSync } from '../wrong-fs.mjs';
import * as fakeFs from '../wrong-fs.mjs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';


function shadowed() {
  const mkdtempSync = (..._args: unknown[]) => process.cwd();
  const root = mkdtempSync(join(tmpdir(), 'scan-'));
  return root;
}

function parameter(mkdtempSync: (prefix: string) => string) {
  return mkdtempSync(join(tmpdir(), 'scan-'));
}

test('local shadow is a live root', () => {
  readdirSync(shadowed());
});
test('parameter is not a Node factory', () => {
  readdirSync(parameter(() => process.cwd()));
});
test('wrong module is not a Node factory', () => {
  readdirSync(fakeSync(join(tmpdir(), 'scan-')));
  readdirSync(fakeFs.mkdtempSync(join(tmpdir(), 'scan-')));
  readdirSync(impossibleSync(join(tmpdir(), 'scan-')));
});
test('genuine Node imports create owned roots', async () => {
  readdirSync(mkdtempSync(join(tmpdir(), 'scan-')));
  readdirSync(createSync(join(tmpdir(), 'scan-')));
  readdirSync(nodeFs.mkdtempSync(join(tmpdir(), 'scan-')));
  readdirSync(fs.mkdtempSync(join(tmpdir(), 'scan-')));
  await readdir(await createAsync(join(tmpdir(), 'scan-')));
  await promises.readdir(await promises.mkdtemp(join(tmpdir(), 'scan-')));
});
