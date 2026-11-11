import { test } from 'vitest';
import { readdirSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';

const first = require('fs');
first.mkdtempSync = () => process.cwd();
const { mkdtempSync } = require('node:fs');

test('a CJS module mutation invalidates an aliased Node binding', () => {
  readdirSync(mkdtempSync(join(tmpdir(), 'scan-')));
});
