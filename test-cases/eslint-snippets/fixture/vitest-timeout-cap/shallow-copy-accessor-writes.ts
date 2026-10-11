import { defineConfig } from 'vitest/config';
import { test } from 'vitest';

const own = { test: { testTimeout: 1 } };
const ownCopy = { ...own, set deadline(value: number) { this.test.testTimeout = value; } };
ownCopy.deadline = 30_001;
defineConfig(own);

const inherited = { test: { testTimeout: 1 } };
const inheritedCopy = {
  ...inherited,
  __proto__: { set deadline(value: number) { this.test.testTimeout = value; } },
};
inheritedCopy.deadline = 30_001;
defineConfig(inherited);

const updated = { test: { testTimeout: 1 } };
const updatedCopy = {
  ...updated,
  get deadline() { return 30_000; },
  set deadline(value: number) { this.test.testTimeout = value; },
};
updatedCopy.deadline++;
defineConfig(updated);

const compound = { test: { testTimeout: 1 } };
const compoundCopy = { ...compound, set deadline(value: number) { this.test.testTimeout = value; } };
compoundCopy.deadline += 30_001;
defineConfig(compound);

const key = process.env.DEADLINE;
const computed = { test: { testTimeout: 1 } };
const computedCopy = { ...computed, set [key](value: number) { this.test.testTimeout = value; } };
computedCopy.deadline = 30_001;
defineConfig(computed);

const absent = { timeout: 1 };
const absentCopy = { ...absent };
// A missing own property can invoke an inherited setter.
absentCopy.deadline = 30_001;
test('unproved inherited setter', () => {}, absent);

const wrapped = { test: { testTimeout: 1 } };
const box = { copy: { ...wrapped, set deadline(value: number) { this.test.testTimeout = value; } } };
const alias = box.copy;
alias.deadline = 30_001;
defineConfig(wrapped);

const removed = { test: { testTimeout: 1 } };
const removedCopy = {
  ...removed,
  __proto__: { set deadline(value: number) { this.test.testTimeout = value; } },
  deadline: 0,
};
delete removedCopy.deadline;
removedCopy.deadline = 30_001;
defineConfig(removed);

const aliasRemoved = { test: { testTimeout: 1 } };
const aliasRemovedCopy = {
  ...aliasRemoved,
  __proto__: { set deadline(value: number) { this.test.testTimeout = value; } },
  deadline: 0,
};
const removingAlias = aliasRemovedCopy;
delete removingAlias.deadline;
aliasRemovedCopy.deadline = 30_001;
defineConfig(aliasRemoved);

const assertedSetter = { test: { testTimeout: 1 } };
const assertedSetterCopy = {
  ...assertedSetter,
  set deadline(value: number) { this.test.testTimeout = value; },
};
(assertedSetterCopy.deadline as number) = 30_001;
defineConfig(assertedSetter);

const nonnullSetter = { test: { testTimeout: 1 } };
const nonnullSetterCopy = {
  ...nonnullSetter,
  set deadline(value: number) { this.test.testTimeout = value; },
};
nonnullSetterCopy.deadline! = 30_001;
defineConfig(nonnullSetter);

const overwritten = { timeout: 1 };
let overwrittenCopy = { ...overwritten };
(overwrittenCopy as { timeout: number }) = { timeout: 30_001 };
overwrittenCopy.timeout = 30_001;
test('overwritten copy', () => {}, overwritten);
