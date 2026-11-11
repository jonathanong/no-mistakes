import { test } from 'vitest';
import { readdirSync } from 'node:fs';

require('fs').makeFactory = () => ({ mkdtempSync: () => process.cwd() });
const mkdtempSync = require('fs').makeFactory().mkdtempSync;
test('a function result is not the Node factory export', () => {
  readdirSync(mkdtempSync('scan-'));
});
