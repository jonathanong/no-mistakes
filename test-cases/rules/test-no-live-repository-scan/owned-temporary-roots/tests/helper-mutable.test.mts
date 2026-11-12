import { test } from 'vitest';
import { mkdtempSync, mkdtempDisposableSync, readdirSync, writeFileSync } from 'node:fs';
import { mkdtempDisposable } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
function workspace() { let root = mkdtempSync(join(tmpdir(), 'scan-')); root = process.cwd(); return root; }
test('live', () => { readdirSync(workspace()); });
