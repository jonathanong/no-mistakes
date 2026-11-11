import { test } from 'vitest';
import { readdirSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';

const key = 'mkdtempSync';
require('fs')[key] = () => process.cwd();
const createSync = require('node:fs').mkdtempSync;
test('an unknown direct require member write invalidates the factory', () => {
  readdirSync(createSync(join(tmpdir(), 'scan-')));
});
