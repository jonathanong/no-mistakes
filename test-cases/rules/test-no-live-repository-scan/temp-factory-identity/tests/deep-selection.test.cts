import { test } from 'vitest';
import { readdirSync } from 'node:fs';

require('fs').other = { deep: { more: { mkdtempSync: () => process.cwd() } } };
const create = require('fs').other.deep.more.mkdtempSync;
test('a deep result is not the Node factory export', () => {
  readdirSync(create('scan-'));
});
