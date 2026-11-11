import { test } from 'vitest';
import { mkdtempSync, mkdtempDisposableSync, readdirSync, writeFileSync } from 'node:fs';
import { mkdtempDisposable } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
function workspace() { const root = mkdtempSync(join(tmpdir(), 'scan-')); return root; }
test('owned', () => { readdirSync(workspace()); });
