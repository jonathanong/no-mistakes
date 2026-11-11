import { test, vi } from 'vitest';
import { mkdtempSync, readdirSync } from 'node:fs';
import * as fs from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';

vi.spyOn(fs, 'mkdtempSync').mockImplementation(() => process.cwd());
test('spying on the factory invalidates ownership', () => {
  readdirSync(mkdtempSync(join(tmpdir(), 'scan-')));
});
