import { test } from 'vitest';
import { mkdtempSync, mkdtempDisposableSync, readdirSync, writeFileSync } from 'node:fs';
import { mkdtempDisposable } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
test('unproven', () => { const resource = mkdtempDisposable(join(tmpdir(), 'scan-')); readdirSync(resource.path); });
