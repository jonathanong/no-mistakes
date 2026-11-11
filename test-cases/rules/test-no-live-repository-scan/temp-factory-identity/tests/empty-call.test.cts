import { test } from 'vitest';
import { readdirSync } from 'node:fs';

const fs = require('fs');
fs();
test('calling the module as a function leaves its factories unresolved', () => {
  readdirSync(fs.mkdtempSync('scan-'));
});
