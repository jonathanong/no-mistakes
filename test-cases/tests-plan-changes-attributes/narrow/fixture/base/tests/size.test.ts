import { expect, test } from "vitest";

declare function loadFixture(id: string): { dimensions: { size: number } };

test("avatar size", () => {
  expect(loadFixture("profile.avatar").dimensions.size).toBe(256);
});
