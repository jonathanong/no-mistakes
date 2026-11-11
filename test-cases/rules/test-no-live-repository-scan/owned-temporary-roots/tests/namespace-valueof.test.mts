import { test } from 'vitest';
import { mkdtempSync, mkdtempDisposableSync, readdirSync, writeFileSync } from 'node:fs';
import { mkdtempDisposable } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import fs from 'node:fs/promises';
test('unproven', async () => { const alias = fs.valueOf(); alias.mkdtempDisposable = async () => ({ path: process.cwd() }); const resource = await fs.mkdtempDisposable(join(tmpdir(), 'scan-')); readdirSync(resource.path); });
