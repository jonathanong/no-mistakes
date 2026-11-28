import { test } from "vitest";
test("reads", () => {
  const err = new Error("boom");
  test.expect(err.message).toBe("boom");
});
