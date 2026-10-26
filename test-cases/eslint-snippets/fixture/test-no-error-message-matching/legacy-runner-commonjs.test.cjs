// Keep CommonJS-global runner aliases covered in both ESLint and type-aware Oxlint.
const test = customRunner;
const run = test;
run("module alias", () => error.message === "expected");
