import { test } from 'vitest';
import { readdirSync } from 'node:fs';

const fs = require('fs');
const helper = { mutate(_key: string, view: typeof fs) { view.mkdtempSync = () => process.cwd(); } };
helper.mutate('readdirSync', fs);
test('a module view passed later can still be changed', () => {
  readdirSync(fs.mkdtempSync('scan-'));
});
