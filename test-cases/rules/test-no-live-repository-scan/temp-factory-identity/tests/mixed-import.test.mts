import { test } from 'vitest';
import { type Stats, mkdtempSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';

type StatValue = Stats;
void (0 as unknown as StatValue);
test('a type-only neighbor does not hide the runtime factory', () => {
  readdirSync(mkdtempSync(join(tmpdir(), 'scan-')));
});
