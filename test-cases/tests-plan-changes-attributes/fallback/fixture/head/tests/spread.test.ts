import { expect, test } from "vitest";

declare function loadFixture(id: string): { size: number; label: string };

test("opaque copy", () => {
  const copy = { ...loadFixture("opaque") };
  expect(copy).toHaveProperty("label");
});
