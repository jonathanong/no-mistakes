import { expect, test } from "vitest";

declare function loadFixture(id: string): { size: number; label: string };
declare function require(specifier: string): { loadFixture: typeof loadFixture };

test("require of loader", () => {
  const id = "opaque";
  expect(loadFixture("opaque").label).toBe("sample");
  const mod = require("./loader-stub");
  expect(mod.loadFixture(id).size).toBe(64);
});
