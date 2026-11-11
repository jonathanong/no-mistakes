import { test } from 'vitest';
import { mkdtempSync, mkdtempDisposableSync, readdirSync, writeFileSync } from 'node:fs';
import { mkdtempDisposable } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { mkdtempDisposableSync as createDirectory } from 'fs';
test('owned', () => { const resource = createDirectory(join(tmpdir(), 'scan-')); readdirSync(resource.path); });
