import { expect, test } from "vitest";

import { identity as readFixture } from "../src/example";

test("unrelated fixture", () => {
  expect(readFixture("example.aab")).toBe("example.aab");
});
