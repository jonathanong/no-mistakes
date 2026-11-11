import { test } from 'vitest';
import { mkdtempSync, mkdtempDisposableSync, readdirSync, writeFileSync } from 'node:fs';
import { mkdtempDisposable } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
function workspace(files: Record<string, string>) { const root = mkdtempSync(join(tmpdir(), 'scan-')); for (const [name, text] of Object.entries(files)) writeFileSync(join(root, name), text); return root; }
test('owned', () => { readdirSync(workspace({ 'package.json': '{}' })); });
