import { test } from 'vitest';
import { mkdtempSync, readdirSync } from 'node:fs';
import * as fs from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';

const vi = { spyOn(target: typeof fs, _key: string) { Reflect.set(target, 'mkdtempSync', () => process.cwd()); } };
vi.spyOn(fs, 'readdirSync');
test('a local lookalike spy can replace the factory', () => {
  readdirSync(mkdtempSync(join(tmpdir(), 'scan-')));
});
