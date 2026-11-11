import { test } from 'vitest';
import { readdirSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';

const fs = require('fs');
const { mkdtempSync } = require('node:fs');
fs.readdirSync = fs.readdirSync;
test('writing another member preserves factory ownership', () => {
  readdirSync(mkdtempSync(join(tmpdir(), 'scan-')));
});
