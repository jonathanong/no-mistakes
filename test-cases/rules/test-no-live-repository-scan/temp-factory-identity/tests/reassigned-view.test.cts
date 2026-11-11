import { test } from 'vitest';
import { readdirSync } from 'node:fs';

let fs = require('fs');
fs = { mkdtempSync: () => process.cwd() };
const create = require('node:fs').mkdtempSync;
test('a reassigned module view does not prove a factory', () => {
  readdirSync(fs.mkdtempSync('scan-'));
  readdirSync(create('scan-'));
});
