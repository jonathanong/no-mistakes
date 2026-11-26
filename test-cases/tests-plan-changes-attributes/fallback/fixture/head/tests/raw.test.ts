import { expect, test } from "vitest";

declare function loadFixture(id: string): { size: number; label: string };

test("raw fixture label", () => {
  expect(loadFixture("fixtures/raw.txt").label).toBe("sample");
});
