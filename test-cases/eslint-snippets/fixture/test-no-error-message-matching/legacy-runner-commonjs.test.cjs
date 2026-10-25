const test = customRunner;
const run = test;
run("module alias", () => error.message === "expected");
