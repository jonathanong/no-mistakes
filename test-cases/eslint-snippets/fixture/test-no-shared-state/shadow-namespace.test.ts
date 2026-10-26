import * as v from "vitest";

const items = new Map();

v.describe("suite", () => {
  const v = { beforeEach: () => {} }; // This local hook never invokes its callback.
  v.beforeEach(() => {
    items.clear();
  });
});

v.test("outside", () => {
  items.set("a", 1);
});
