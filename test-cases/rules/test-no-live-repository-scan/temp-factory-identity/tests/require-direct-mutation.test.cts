import { test } from 'vitest';
import { readdirSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';

require('fs').mkdtempSync = () => process.cwd();
const createSync = require('node:fs').mkdtempSync;
test('a direct require factory write invalidates a later binding', () => {
  readdirSync(createSync(join(tmpdir(), 'scan-')));
  readdirSync(require('fs').mkdtempSync(join(tmpdir(), 'scan-')));
});
