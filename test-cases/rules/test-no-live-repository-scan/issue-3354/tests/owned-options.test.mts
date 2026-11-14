import { mkdtempSync, readdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { it } from 'vitest';
import { scanMigrations } from '../src/owned-options.mts';

declare const baseOptions: { root?: string; recursive?: boolean };

it('keeps the final explicit owned root after a spread', () => {
  const root = mkdtempSync(join(tmpdir(), 'case-'));
  const options = { ...baseOptions, root };
  scanMigrations(options);
  const directOptions = { ...baseOptions, root };
  readdirSync(directOptions.root);
});
