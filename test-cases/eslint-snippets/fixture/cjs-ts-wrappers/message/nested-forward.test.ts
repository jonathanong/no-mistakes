import vt from "vitest";
const other = (message) => message;
vt.test("reads", () => {
  const err = new Error("boom");
  vt.expect.call.call(other, err.message, 42).toBe(42);
  vt.expect.call(null, err.message).toBe("x");
});
