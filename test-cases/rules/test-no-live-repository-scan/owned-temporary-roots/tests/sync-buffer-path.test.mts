import { test } from 'vitest';
import { mkdtempSync, mkdtempDisposableSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
test('mutable buffer path', () => { const resource = mkdtempDisposableSync(join(tmpdir(), 'scan-'), { encoding: 'buffer' }); const root = resource.path; const alias = root; alias.fill(47); readdirSync(root); });
