import { test } from 'vitest';
import { promises as fs } from 'node:fs';
import { promises as wrong } from '../wrong-fs.mjs';
import { readdir } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';

test('named Node promises module owns the temporary root', async () => {
  await readdir(await fs.mkdtemp(join(tmpdir(), 'scan-')));
});
test('a similarly named local module is not owned', async () => {
  await readdir(await wrong.mkdtemp(join(tmpdir(), 'scan-')));
});
