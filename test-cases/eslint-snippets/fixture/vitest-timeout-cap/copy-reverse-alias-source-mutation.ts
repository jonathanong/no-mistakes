import { defineConfig } from "vitest/config";

const valueOfShared = {
  testTimeout: 1,
  valueOf() {
    this.testTimeout = 30_001;
    return 0;
  },
};
const valueOfSource = { test: valueOfShared, counter: 2 };
const valueOfCopy = { ...valueOfSource };
const valueOfAlias = valueOfCopy;
valueOfCopy.counter = valueOfShared;
valueOfAlias.counter++;

const primitiveShared = {
  testTimeout: 1,
  [Symbol.toPrimitive]() {
    this.testTimeout = 60_000;
    return 0;
  },
};
const primitiveSource = { test: primitiveShared, counter: 2 };
const primitiveCopy = { ...primitiveSource };
const primitiveAlias = primitiveCopy;
primitiveCopy.counter = primitiveShared;
primitiveAlias.counter -= 1;

// These primitive snapshots verify real source mutation without exporting its identity.
export const observedTimeouts = [
  defineConfig(valueOfSource).test.testTimeout,
  defineConfig(primitiveSource).test.testTimeout,
];
