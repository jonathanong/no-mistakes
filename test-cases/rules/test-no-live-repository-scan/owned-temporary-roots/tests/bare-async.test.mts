import { test } from 'vitest';
import { mkdtempSync, mkdtempDisposableSync, readdirSync, writeFileSync } from 'node:fs';
import { mkdtempDisposable } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { mkdtempDisposable as createDirectory } from 'fs/promises';
test('owned', async () => { const resource = await createDirectory(join(tmpdir(), 'scan-')); readdirSync(resource.path); });
