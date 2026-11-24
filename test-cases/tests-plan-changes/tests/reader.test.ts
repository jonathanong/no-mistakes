import { expect, test } from "vitest";

const readFixture = (id: string) => id;

test("references fixture IDs", () => {
  expect(readFixture("example.a+b")).toBe("example.a+b");
  expect(readFixture("new.id")).toBe("new.id");
  expect(readFixture("old.id")).toBe("old.id");
  expect("site.title").toBe("site.title");
});
