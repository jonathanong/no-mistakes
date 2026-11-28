const spec = "vitest";
const { expect: check } = require(spec);
const { test } = require("vitest");
const forwarded = [null, 42];
const values = [42];
test("reads", () => {
  const err = new Error("boom");
  check.call(...forwarded, err.message).toBe(42);
  check.apply(err.message, values).toBe(42);
  check.call(null, ...[err.message]).toBe("x");
});
