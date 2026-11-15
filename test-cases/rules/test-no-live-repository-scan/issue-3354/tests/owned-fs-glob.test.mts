import { globSync as nodeGlobSync, mkdtempSync } from 'node:fs';
import { globSync as fsGlobSync } from 'fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { it } from 'vitest';

declare const opaqueOptions: { cwd?: string };

it('uses an owned cwd for Node glob patterns', () => {
  const root = mkdtempSync(join(tmpdir(), 'case-'));
  nodeGlobSync('**/*.test.mts', { cwd: root });
  fsGlobSync('**/*.test.mts', { cwd: root });
  nodeGlobSync('**/*.test.mts', { ...opaqueOptions, cwd: root });
});
