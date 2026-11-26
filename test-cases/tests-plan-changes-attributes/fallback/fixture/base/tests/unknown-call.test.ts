import { expect, test } from "vitest";

declare function loadFixture(id: string): { size: number; label: string };
declare function inspect(value: unknown): boolean;

test("opaque inspection", () => {
  const avatar = loadFixture("opaque");
  expect(inspect(avatar)).toBe(true);
});
