import { expect, test } from "vitest";
declare function loadFixture(id: string): unknown;
test("avatar", () => {
  expect(loadFixture("avatar")).toBeDefined();
});
