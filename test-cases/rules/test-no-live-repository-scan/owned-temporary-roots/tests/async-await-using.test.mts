import { test } from 'vitest';
import { mkdtempSync, mkdtempDisposableSync, readdirSync, writeFileSync } from 'node:fs';
import { mkdtempDisposable } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
test('owned', async () => { await using resource = await mkdtempDisposable(join(tmpdir(), 'scan-')); readdirSync(join(resource.path, 'lib')); });
