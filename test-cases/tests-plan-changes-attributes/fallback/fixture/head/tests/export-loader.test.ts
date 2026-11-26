import { expect, test } from "vitest";

declare function loadFixture(id: string): { size: number; label: string };

test("exported loader", () => {
  expect(loadFixture("opaque").label).toBe("sample");
});

export { loadFixture };
