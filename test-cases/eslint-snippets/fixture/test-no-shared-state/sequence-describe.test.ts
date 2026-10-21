import * as v from "vitest";

const items = new Map();

(0, v.describe)("A", () => {
  v.beforeEach(() => {
    items.clear();
  });
});

v.test("outside", () => {
  items.set("a", 1);
});
