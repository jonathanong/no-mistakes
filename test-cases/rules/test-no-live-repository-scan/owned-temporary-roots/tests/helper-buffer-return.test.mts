import { test } from 'vitest';
import { mkdtempSync, mkdtempDisposableSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
function workspace() { for (const name of []) void name; return mkdtempSync(join(tmpdir(), 'scan-'), { encoding: 'buffer' }); }
test('mutable buffer root', () => { const root = workspace(); const alias = root; alias.fill(47); readdirSync(root); });
