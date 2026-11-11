import { test } from 'vitest';
import { readdirSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';

function replaceFactory(view: { mkdtempSync: (prefix: string) => string }) {
  view.mkdtempSync = () => process.cwd();
}
replaceFactory(require('fs'));
const create = require('node:fs').mkdtempSync;
test('an escaped direct require view can replace the factory', () => {
  readdirSync(create(join(tmpdir(), 'scan-')));
});
