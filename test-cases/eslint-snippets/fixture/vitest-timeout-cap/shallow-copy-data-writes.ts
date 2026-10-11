import { defineConfig } from 'vitest/config';
import { test } from 'vitest';

const plain = { timeout: 1 };
const plainCopy = { ...plain };
plainCopy.timeout = 30_001;
test('plain data', () => {}, plain);

const deleted = { test: { testTimeout: 1 } };
const deletedCopy = { ...deleted, set deadline(value: number) { this.test.testTimeout = value; } };
delete deletedCopy.deadline;
defineConfig(deleted);

const restored = { test: { testTimeout: 1 } };
const restoredCopy = {
  ...restored,
  __proto__: { set deadline(value: number) { this.test.testTimeout = value; } },
  deadline: 0,
};
restoredCopy.deadline++;
defineConfig(restored);

const getter = { test: { testTimeout: 1 }, get deadline() { return 0; } };
const getterCopy = { ...getter };
// Object spread creates data descriptors even when the source has an accessor.
getterCopy.deadline = 30_001;
defineConfig(getter);

const wrapped = { timeout: 1 };
const box = { copy: { ...wrapped } };
const alias = box.copy;
alias.timeout = 30_001;
test('wrapped data', () => {}, wrapped);

const array = { timeout: 1 };
const arrayCopy = [{ ...array }];
arrayCopy[0].timeout = 30_001;
test('array wrapped data', () => {}, array);

const proto = { timeout: 1 };
const protoCopy = { ...proto, __proto__: {} };
delete protoCopy.__proto__;
test('deleting prototype key', () => {}, proto);

const asserted = { timeout: 1 };
const assertedCopy = { ...asserted };
(assertedCopy.timeout as number) = 30_001;
test('asserted own data', () => {}, asserted);

const nonnull = { timeout: 1 };
const nonnullCopy = { ...nonnull };
nonnullCopy.timeout! = 30_001;
test('nonnull own data', () => {}, nonnull);

const wrappedDelete = { timeout: 1 };
const wrappedDeleteCopy = { ...wrappedDelete };
delete (wrappedDeleteCopy.timeout as number);
test('asserted deletion', () => {}, wrappedDelete);
