const spec = "vitest";
const vt = require(spec);
const { expect: check } = require(spec);
const { test } = require("vitest");
test("reads", () => {
  const err = new Error("boom");
  vt.expect(err.message).toBe("boom");
  check.call(null, err.message).toBe("x");
  check.apply(null, [err.message]).toBe("x");
});
