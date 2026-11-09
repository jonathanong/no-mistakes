import { owner } from './owner.mjs';
import { failedOwner } from './failed-owner.mjs';
import * as fs from 'node:fs';

const externalCallback = fs.promises.readFile;

Promise.resolve().finally(owner);

export function opaqueCallback() {
  return Promise.resolve().finally(externalCallback);
}

export function failedCallback() {
  return Promise.resolve().finally(failedOwner);
}

export function unknownOuter() {
  return Promise.resolve().finally(owner)();
}
