import { expect, test } from "vitest";

declare function loadFixture(id: string): { encoding: { format: string } };

test("avatar encoding", () => {
  const avatar = loadFixture("profile.avatar");
  const copy = avatar;
  expect(copy.encoding).toMatchObject({ format: "webp" });
});
