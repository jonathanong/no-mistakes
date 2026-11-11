import { test } from 'vitest';
import { readdirSync } from 'node:fs';
import * as fs from 'node:fs';
import * as wrongFs from '../wrong-fs.mjs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';

test('the static Node promises receiver creates an owned root', async () => {
  readdirSync(await fs.promises.mkdtemp(join(tmpdir(), 'scan-')));
});
test('wrong modules and unavailable sync exports are unproven', async () => {
  readdirSync(await wrongFs.promises.mkdtemp(join(tmpdir(), 'scan-')));
  readdirSync(fs.promises.mkdtempSync(join(tmpdir(), 'scan-')));
});
