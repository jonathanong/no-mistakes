import { readdirSync, mkdtempSync as owned } from 'node:fs';
import * as fs from 'fs';
import * as promises from 'node:fs/promises';
import { promises as namedPromises } from 'node:fs';
import * as nodeFs from 'node:fs';
import * as wrongFs from './wrong-fs.mjs';
import { mkdtempSync as wrong } from './wrong-fs.mjs';
import { mkdtempSync as impossible } from 'node:fs/promises';
import promisesDefault from 'node:fs/promises';
import type { mkdtempSync as unusedType } from 'node:fs';
import type * as unusedTypes from 'node:fs';
import { type mkdtempSync as unusedNamedType } from 'node:fs';

function shadow(owned: (prefix: string) => string) {
  readdirSync(owned('shadow-'));
}
readdirSync(owned('case-'));
readdirSync(fs.mkdtempSync('case-'));
readdirSync(promises.mkdtemp('case-'));
readdirSync(namedPromises.mkdtemp('case-'));
readdirSync(nodeFs.promises.mkdtemp('case-'));
readdirSync(nodeFs.promises.mkdtempSync('case-'));
readdirSync(wrongFs.promises.mkdtemp('case-'));
readdirSync(promises.mkdtempSync('case-'));
readdirSync(promisesDefault.mkdtempSync('case-'));
readdirSync(wrong('case-'));
readdirSync(impossible('case-'));
