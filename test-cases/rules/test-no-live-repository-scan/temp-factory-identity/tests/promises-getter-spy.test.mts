import { test, vi } from 'vitest';
import * as fs from 'node:fs';
import { readdir } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';

vi.spyOn(fs, 'promises', 'get').mockReturnValue({ mkdtemp: async () => process.cwd() });
test('a promises getter spy invalidates the nested factory', async () => {
  await readdir(await fs.promises.mkdtemp(join(tmpdir(), 'scan-')));
});
