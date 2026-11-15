import { mkdtempSync, readdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path, { join } from 'node:path';
import { it } from 'vitest';

it('enumerates known children of an owned root', () => {
  const root = mkdtempSync(join(tmpdir(), 'case-'));
  for (const name of ['a', 'b'] as const) {
    const dir = path.join(root, name);
    readdirSync(dir);
  }
  for (const literalName of ['c', 'd']) {
    readdirSync(join(root, literalName));
  }
  for (const boundaryName of ['child-1', 'child-2', 'child-3', 'child-4', 'child-5', 'child-6', 'child-7', 'child-8', 'child-9', 'child-10', 'child-11', 'child-12', 'child-13', 'child-14', 'child-15', 'child-16'] as const) {
    readdirSync(join(root, boundaryName));
  }
  readdirSync(join(root, 'nested', 'child'));
});
