import { test } from 'vitest';
import { mkdtempSync, mkdtempDisposableSync, readdirSync, writeFileSync } from 'node:fs';
import { mkdtempDisposable } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
function workspace(flag: boolean) { if (flag) return mkdtempSync(join(tmpdir(), 'scan-')); return process.cwd(); }
test('unproven', () => { readdirSync(workspace(true)); });
