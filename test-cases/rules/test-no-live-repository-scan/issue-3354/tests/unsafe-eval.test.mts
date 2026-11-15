import { mkdtempSync, readdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { it } from 'vitest';

it('keeps roots unresolved after direct eval', () => {
  const root = mkdtempSync(join(tmpdir(), 'case-'));
  eval('root');
  readdirSync(root);
});
