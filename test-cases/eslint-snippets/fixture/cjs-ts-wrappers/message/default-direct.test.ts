import vt from "vitest";
vt("reads", () => {
  const err = new Error("boom");
  if (err.message === "boom") return;
});
