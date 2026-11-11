import { test } from 'vitest';
import { readdirSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';

const fs = require('fs');
fs['mkdtemp\u0053ync'] = () => process.cwd();
const { mkdtempSync: createTemp } = require('node:fs');

test('a computed factory write invalidates a separate binding', () => {
  readdirSync(createTemp(join(tmpdir(), 'scan-')));
});
