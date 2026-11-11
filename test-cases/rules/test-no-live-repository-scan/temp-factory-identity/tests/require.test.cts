import { test } from 'vitest';
import { readdirSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';

const fs = require('node:fs');
const { mkdtempSync: createSync } = require('fs');
const wrong = require('../wrong-fs.mjs');

test('Node require factories create owned roots', () => {
  readdirSync(fs.mkdtempSync(join(tmpdir(), 'scan-')));
  readdirSync(createSync(join(tmpdir(), 'scan-')));
});
test('other require modules do not create owned roots', () => {
  readdirSync(wrong.mkdtempSync(join(tmpdir(), 'scan-')));
});
