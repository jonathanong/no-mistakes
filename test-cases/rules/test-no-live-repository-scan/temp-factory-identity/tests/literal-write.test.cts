import { test } from 'vitest';
import { readdirSync } from 'node:fs';

require('fs')['mkdtempSync'] = () => process.cwd();
const create = require('node:fs').mkdtempSync;
test('a computed literal write invalidates the bound factory', () => {
  readdirSync(create('scan-'));
});
