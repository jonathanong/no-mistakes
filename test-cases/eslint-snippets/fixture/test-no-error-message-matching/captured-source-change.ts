import { expect, test } from "vitest";
let error = { message: "first", code: "FIRST" };
const text = error.message;
// A captured message remains message-derived when the source binding changes.
error = { message: "second", code: "SECOND" };
expect(text).toBe("first");
const captured = error.message;
error.message = "third";
expect(captured).toMatch("second");
test("snapshot", () => {
  const local = error.message;
  error.message = "fourth";
  local.includes("third");
});
