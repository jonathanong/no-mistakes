import { expect, test } from "vitest";

declare function loadFixture(id: string): { size: number; label: string };

test("aliased loader", () => {
  const id = "opaque";
  const get = loadFixture;
  expect(loadFixture("opaque").label).toBe("sample");
  expect(get(id).size).toBe(64);
});
