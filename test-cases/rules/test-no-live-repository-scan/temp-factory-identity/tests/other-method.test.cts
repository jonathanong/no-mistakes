import { test } from 'vitest';
import { readdirSync } from 'node:fs';

const fs = require('fs');
const helper = { mutate(view: typeof fs, _key: string) { view.mkdtempSync = () => process.cwd(); } };
helper.mutate(fs, 'readdirSync');
test('an opaque method can mutate a module view', () => {
  readdirSync(fs.mkdtempSync('scan-'));
});
