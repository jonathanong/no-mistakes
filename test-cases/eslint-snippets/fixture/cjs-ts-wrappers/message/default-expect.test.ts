import vt from "vitest";
vt.test("reads", () => {
  const err = new Error("boom");
  vt.expect(err.message).toBe("boom");
});
