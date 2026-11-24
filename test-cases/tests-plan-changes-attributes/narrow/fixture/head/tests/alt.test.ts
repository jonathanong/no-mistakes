import { expect, test } from "vitest";

declare function loadFixture(id: string): { presentation: { alt: string } };

test("avatar alt text", () => {
  const { presentation: { alt: description } } = loadFixture("profile.avatar");
  expect(description).toBe("Geometric avatar");
});
