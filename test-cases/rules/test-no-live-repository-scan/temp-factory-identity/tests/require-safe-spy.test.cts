import { test, vi } from 'vitest';
import { readdirSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';

vi.spyOn(require('fs'), 'readdirSync');
const create = require('node:fs').mkdtempSync;
test('a genuine spy on another member leaves the factory intact', () => {
  readdirSync(create(join(tmpdir(), 'scan-')));
});
