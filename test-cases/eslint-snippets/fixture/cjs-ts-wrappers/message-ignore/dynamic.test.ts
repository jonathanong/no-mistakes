const spec = "vitest";
const vt = require(spec);
const { test } = require("vitest");
test("reads", () => {
  const err = new Error("boom");
  vt.expect(err.message).toBe("boom");
});
