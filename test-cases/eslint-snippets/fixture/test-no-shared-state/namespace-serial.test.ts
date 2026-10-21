import * as v from "vitest";

const items = new Map<string, number>();

v.describe.configure({ mode: "serial" });
v.beforeAll(() => {
  items.clear();
});
v.test("a", () => {
  items.set("a", 1);
});
