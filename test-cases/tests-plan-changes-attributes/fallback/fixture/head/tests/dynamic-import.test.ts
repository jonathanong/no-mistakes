import { expect, test } from "vitest";

declare function loadFixture(id: string): { size: number; label: string };

test("dynamic import of loader", async () => {
  const id = "opaque";
  expect(loadFixture("opaque").label).toBe("sample");
  const mod = await import("./loader-stub");
  expect(mod.loadFixture(id).size).toBe(64);
});
