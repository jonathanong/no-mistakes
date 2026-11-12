import { test } from 'vitest';
import { mkdtempSync, mkdtempDisposableSync, readdirSync, writeFileSync } from 'node:fs';
import { mkdtempDisposable } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { mutate } from 'fixture-mutator';
test('unproven', () => { const resource = mkdtempDisposableSync(join(tmpdir(), 'scan-')); mutate(resource); readdirSync(resource.path); });
