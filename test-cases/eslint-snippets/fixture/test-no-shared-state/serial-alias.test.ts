import * as v from "vitest";

const items = new Map();
const suite = v.describe.serial;

suite("A", () => {
  v.beforeAll(() => {
    items.clear();
  });
  v.test("x", () => {
    items.set("a", 1);
  });
});
