const run = require("vitest").test;
run("reads", () => {
  const err = new Error("boom");
  if (err.message === "boom") return;
});
