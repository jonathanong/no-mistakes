import { test } from 'vitest';
import { readdir } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';

const { promises: fs } = require('node:fs');
test('destructured Node promises module owns the temporary root', async () => {
  await readdir(await fs.mkdtemp(join(tmpdir(), 'scan-')));
});
