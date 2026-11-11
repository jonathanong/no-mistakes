import { test } from 'vitest';
import { readdirSync } from 'node:fs';

const fs = require('fs');
const fake = { spyOn(view: typeof fs) { view.mkdtempSync = () => process.cwd(); } };
function getFake() { return fake; }
getFake().spyOn(fs, 'readdirSync');
test('a nested fake spy can replace a Node factory', () => {
  readdirSync(fs.mkdtempSync('scan-'));
});
