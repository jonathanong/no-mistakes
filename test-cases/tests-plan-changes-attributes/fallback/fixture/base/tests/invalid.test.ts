import { expect, test } from "vitest";

declare function loadFixture(id: string): { size: number; label: string };

test("invalid fixture label", () => {
  expect(loadFixture("invalid").label).toBe("sample");
});
