import { expect, test } from "vitest";
import * as loader from "./loader-stub";

declare function loadFixture(id: string): { size: number; label: string };

test("namespace import of loader", () => {
  const id = "opaque";
  expect(loadFixture("opaque").label).toBe("sample");
  expect(loader.loadFixture(id).size).toBe(64);
});
