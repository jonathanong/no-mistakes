import { test, vi } from 'vitest';
import { mkdtempSync, readdirSync } from 'node:fs';
import * as fs from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';

type FsModule = typeof fs;
void (0 as unknown as FsModule);
vi.spyOn(fs, 'readdirSync');
test('spying on another member does not replace the factory', () => {
  readdirSync(mkdtempSync(join(tmpdir(), 'scan-')));
});
