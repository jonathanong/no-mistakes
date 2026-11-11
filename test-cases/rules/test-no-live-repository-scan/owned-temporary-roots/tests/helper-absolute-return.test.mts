import { test } from 'vitest';
import { mkdtempSync, mkdtempDisposableSync, readdirSync, writeFileSync } from 'node:fs';
import { mkdtempDisposable } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { pathToFileURL } from 'node:url';
function workspace(flag: boolean) { if (flag) return new URL('/repository', pathToFileURL(mkdtempSync(join(tmpdir(), 'scan-')))); return mkdtempSync(join(tmpdir(), 'scan-')); }
test('unproven', () => { readdirSync(workspace(true)); });
