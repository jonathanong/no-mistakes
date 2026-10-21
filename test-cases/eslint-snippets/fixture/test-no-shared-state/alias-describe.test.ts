import * as v from "vitest";

const items = new Map();
const suite = v.describe;

suite("A", () => {
  v.beforeEach(() => {
    items.clear();
  });
});

v.test("outside", () => {
  items.set("a", 1);
});
