import { test } from 'vitest';
import { mkdtempSync, mkdtempDisposableSync, readdirSync, writeFileSync } from 'node:fs';
import { mkdtempDisposable } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import fs from 'node:fs';
test('unproven', () => { delete fs.mkdtempDisposableSync; const resource = fs.mkdtempDisposableSync(join(tmpdir(), 'scan-')); readdirSync(resource.path); });
