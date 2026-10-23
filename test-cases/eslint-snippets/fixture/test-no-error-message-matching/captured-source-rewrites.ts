import { expect, test } from "vitest";
let error = { message: "first", code: "FIRST" };
// Preserve each original capture point instead of rereading the source at assertion time.
const code = error.code;
error = { message: "second", code: "SECOND" };
expect(code).toBe("FIRST");
const captured = error.code;
error.message = "third";
expect(captured).toBe("SECOND");
test("snapshot", () => {
  const local = error.code;
  error.message = "fourth";
  local === "SECOND";
});
