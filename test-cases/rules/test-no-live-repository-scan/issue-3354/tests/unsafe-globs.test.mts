import { globSync, mkdtempSync, readdirSync } from 'node:fs';
import { globSync as fsGlobSync } from 'fs';
import { globSync as fastGlobSync } from 'fast-glob';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { it } from 'vitest';

declare const unknownPattern: string;
declare const opaqueOptions: { cwd: string };

it('reports globs without a bounded pattern and cwd', () => {
  const root = mkdtempSync(join(tmpdir(), 'case-'));
  globSync('**/*.test.mts', { cwd: process.cwd() });
  fsGlobSync('**/*.test.mts');
  fastGlobSync('**/*.test.mts');
  globSync(unknownPattern, { cwd: root });
  globSync('/outside/**/*.test.mts', { cwd: root });
  globSync('../**/*.test.mts', { cwd: root });
  globSync('**/*.test.mts', opaqueOptions);
  globSync('**/*.test.mts', { cwd: root, ...opaqueOptions });
  globSync('**/*.test.mts', { cwd: root, cwd: process.cwd() });
  readdirSync('./src', { cwd: root });
  import.meta.glob('**/*.test.mts', { cwd: root });
  import.meta.globEager('**/*.test.mts', { cwd: root });
});
