import { defineConfig } from "vitest/config";
// Assertions and non-null wrappers are erased around an entire member update.
const numeric = { test: { testTimeout: 1 }, counter: 2 };
const numericCopy = { ...numeric };
(numericCopy.counter as number)++;
(numericCopy.counter as number) *= 2;
defineConfig(numeric);
const nullable = { test: { testTimeout: 1 }, counter: null };
const nullableCopy = { ...nullable };
nullableCopy.counter!++;
nullableCopy.counter! -= 1;
defineConfig(nullable);
// Erased wrappers must not hide object replacement before a later coercion.
const overwritten = { test: { testTimeout: 1 }, counter: 2 };
const overwrittenCopy = { ...overwritten };
(overwrittenCopy.counter as unknown) = {
  valueOf() {
    return 0;
  },
};
(overwrittenCopy.counter as unknown)++;
defineConfig(overwritten);
const overwrittenNonNull = { test: { testTimeout: 1 }, counter: 2 };
const overwrittenNonNullCopy = { ...overwrittenNonNull };
overwrittenNonNullCopy.counter! = {
  valueOf() {
    return 0;
  },
};
overwrittenNonNullCopy.counter!++;
defineConfig(overwrittenNonNull);
const shared = {
  test: {
    testTimeout: 1,
    valueOf() {
      this.testTimeout = 60_000;
      return 0;
    },
  },
};
const sharedCopy = { ...shared };
(sharedCopy.test as unknown)++;
defineConfig(shared);
