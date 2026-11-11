import { test } from 'vitest';
import { mkdtempSync, mkdtempDisposableSync, readdirSync, writeFileSync } from 'node:fs';
import { mkdtempDisposable } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import * as fs from 'node:fs/promises';
test('owned', async () => { await using resource = await fs.mkdtempDisposable(join(tmpdir(), 'scan-')); readdirSync(resource.path); });
