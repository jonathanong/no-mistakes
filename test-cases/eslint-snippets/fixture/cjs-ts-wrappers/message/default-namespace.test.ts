import vt from "vitest";
vt.test("reads", () => {
  const err = new Error("boom");
  if (err.message === "boom") return;
});
