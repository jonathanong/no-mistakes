const exact = require("vitest").expect;
const { test } = require("vitest");
const values = [42];
test("reads", () => {
  const err = new Error("boom");
  exact.apply(err.message, values).toBe(42);
  exact.apply(null, [err.message]).toBe(42);
  exact.apply(null, [...[err.message]]).toBe("x");
  exact.call(null, err.message, ...values).toBe("x");
  exact.apply(null, [err.message], ...values).toBe("x");
});
