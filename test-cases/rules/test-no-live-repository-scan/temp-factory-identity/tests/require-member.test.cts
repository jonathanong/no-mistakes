import { test } from 'vitest';
import { readdirSync } from 'node:fs';
import { readdir } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';

const createSync = require('node:fs').mkdtempSync;
const createAsync = require('fs').promises.mkdtemp;
const promises = require('node:fs').promises;
const fromPromiseModule = require('fs/promises').mkdtemp;
const fake = require('../wrong-fs.mjs').mkdtempSync;
const fakePromises = require('../wrong-fs.mjs').promises.mkdtemp;
const unknown = require('node:fs').other.mkdtempSync;
const invalidSync = require('node:fs/promises').mkdtempSync;
const invalidNested = require('fs').promises.mkdtempSync;

test('static Node require member chains create owned roots', async () => {
  readdirSync(createSync(join(tmpdir(), 'scan-')));
  await readdir(await createAsync(join(tmpdir(), 'scan-')));
  await readdir(await promises.mkdtemp(join(tmpdir(), 'scan-')));
  await readdir(await fromPromiseModule(join(tmpdir(), 'scan-')));
});
test('other modules, unknown members, and sync promises remain unproven', async () => {
  readdirSync(fake(join(tmpdir(), 'scan-')));
  await readdir(await fakePromises(join(tmpdir(), 'scan-')));
  readdirSync(unknown(join(tmpdir(), 'scan-')));
  readdirSync(invalidSync(join(tmpdir(), 'scan-')));
  readdirSync(invalidNested(join(tmpdir(), 'scan-')));
});
