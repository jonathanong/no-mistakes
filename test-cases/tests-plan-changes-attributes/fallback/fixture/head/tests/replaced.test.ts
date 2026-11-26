import { expect, test } from "vitest";

declare function loadFixture(id: string): { size: number; label: string };

test("replaced fixture label", () => {
  expect(loadFixture("replaced").label).toBe("sample");
});
