import { test } from 'vitest';
import { mkdtempSync, readdirSync } from 'node:fs';
import * as mutableFs from 'fs';
import * as escapedFs from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';

Reflect.set(mutableFs, 'mkdtempSync', () => process.cwd());
const opaqueAlias = escapedFs;
void opaqueAlias;

test('a mutated or escaped Node module invalidates its factories', () => {
  readdirSync(mutableFs.mkdtempSync(join(tmpdir(), 'scan-')));
  readdirSync(escapedFs.mkdtempSync(join(tmpdir(), 'scan-')));
  readdirSync(mkdtempSync(join(tmpdir(), 'scan-')));
});
