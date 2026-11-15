import { mkdtempSync } from 'node:fs';
import { globSync } from 'fast-glob';
import fastGlob from 'fast-glob';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { it } from 'vitest';

it('uses an owned cwd for fast-glob patterns', () => {
  const root = mkdtempSync(join(tmpdir(), 'case-'));
  globSync('**/*.test.mts', { cwd: root });
  fastGlob.sync('**/*.test.mts', { cwd: root });
});
