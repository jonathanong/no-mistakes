import { expect, test } from "vitest";

declare function loadFixture(id: string): { size: number; label: string };

test("opaque fields", () => {
  const keys = [];
  for (const key in loadFixture("opaque")) {
    keys.push(key);
  }
  expect(keys).toContain("label");
});
