import * as pw from "@playwright/test";
const suffix = randomSuffix();
// A const loop binding has no initializer and is not a known test import.
for (const runner of runners) {
  runner.beforeAll(() => use(suffix));
}
