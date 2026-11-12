import { test } from 'vitest';
import { mkdtempSync, mkdtempDisposableSync, readdirSync, writeFileSync } from 'node:fs';
import { mkdtempDisposable } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import * as fs from 'fixture-fs';
test('unproven', () => { const resource = fs.mkdtempDisposableSync('scan-'); readdirSync(resource.path); });
