import { defineConfig } from "vitest/config";
import { test } from "vitest";
// Deletion through an erased wrapper can expose an inherited setter on the copy.
const asserted = { test: { testTimeout: 1 } };
const assertedCopy = {
  ...asserted,
  __proto__: { set deadline(value: number) { this.test.testTimeout = value; } },
  deadline: 0,
};
const assertedBox = ({ copy: assertedCopy } as const);
delete assertedBox.copy.deadline;
assertedCopy.deadline = 30_001;
defineConfig(asserted);
const satisfied = { test: { testTimeout: 1 } };
const satisfiedCopy = {
  ...satisfied,
  __proto__: { set deadline(value: number) { this.test.testTimeout = value; } },
  deadline: 0,
};
const satisfiedBox = { copy: (satisfiedCopy satisfies object) };
delete satisfiedBox.copy.deadline;
satisfiedCopy.deadline = 30_001;
defineConfig(satisfied);
const nonNull = { test: { testTimeout: 1 } };
const nonNullCopy = {
  ...nonNull,
  __proto__: { set deadline(value: number) { this.test.testTimeout = value; } },
  deadline: 0,
};
const nonNullBox = ([nonNullCopy] as const)!;
delete nonNullBox[0].deadline;
nonNullCopy.deadline = 30_001;
defineConfig(nonNull);
// Deletion itself does not invoke a setter; independent data writes stay safe.
const deleted = { timeout: 1 };
const deletedBox = ({ copy: { ...deleted } } as const);
delete deletedBox.copy.timeout;
test("asserted independent deletion", deleted, () => {});
const data = { timeout: 1 };
const dataBox = ({ copy: { ...data } } satisfies object);
dataBox.copy.timeout = 30_001;
test("asserted independent data write", data, () => {});
