import { test, vi } from 'vitest';
import * as fs from 'node:fs';
import { readdir } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';

vi.spyOn(fs.promises, 'readdir');
test('spying on a separate promises member keeps the Node factories', async () => {
  fs.readdirSync(fs.mkdtempSync(join(tmpdir(), 'scan-')));
  await readdir(await fs.promises.mkdtemp(join(tmpdir(), 'scan-')));
});
