import { type default as vt } from "vitest";

let count = 0;
vt.test("one", () => {
  count += 1;
});
vt.test("two", () => {
  count += 1;
});
