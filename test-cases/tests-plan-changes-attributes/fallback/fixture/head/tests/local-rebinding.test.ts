import { expect, test } from "vitest";

declare function loadFixture(id: string): { size: number; label: string };

test("local re-binding of loader", () => {
  expect(loadFixture("opaque").label).toBe("sample");
  function inner(loadFixture: (id: string) => { size: number }) {
    return loadFixture("other").size;
  }
  expect(inner).toBeDefined();
});
