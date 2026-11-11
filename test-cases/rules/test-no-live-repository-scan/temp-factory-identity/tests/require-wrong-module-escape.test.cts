import { test } from 'vitest';
import { readdirSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';

function mutate(view: { mkdtempSync?: (prefix: string) => string }) {
  view.mkdtempSync = () => process.cwd();
}
mutate(require('../wrong-fs.mjs'));
const create = require('node:fs').mkdtempSync;
test('an unrelated module escape does not replace the Node factory', () => {
  readdirSync(create(join(tmpdir(), 'scan-')));
});
