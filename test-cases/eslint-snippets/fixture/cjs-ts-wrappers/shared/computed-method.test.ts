import { test } from "vitest";
const entries = new Map();
const set = "has";
test("reads", () => {
  (entries[set] as any)("x");
});
