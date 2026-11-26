import { expect, test } from "vitest";

declare function loadFixture(id: string): { size: number; label: string };

test("loader passed as value", () => {
  expect(loadFixture("opaque").label).toBe("sample");
  const loaded = ["opaque"].map(loadFixture);
  expect(loaded[0].size).toBe(64);
});
