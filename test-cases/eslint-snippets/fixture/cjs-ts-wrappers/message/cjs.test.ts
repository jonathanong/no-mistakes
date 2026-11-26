import vt = require("vitest");
const { test } = require("vitest");
const { expect: e } = require("@playwright/test");
test("reads", () => {
  const err = new Error("boom");
  vt.expect(err.message).toBe("boom");
  e(err.message).toBe("boom");
  if (err.message === "boom") return;
});
