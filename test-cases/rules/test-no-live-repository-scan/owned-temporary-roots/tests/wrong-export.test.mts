import { test } from 'vitest';
import { mkdtempSync, mkdtempDisposableSync, readdirSync, writeFileSync } from 'node:fs';
import { mkdtempDisposable } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { readFileSync as createDirectory } from 'node:fs';
test('unproven', () => { const resource = createDirectory('source'); readdirSync(resource.path); });
