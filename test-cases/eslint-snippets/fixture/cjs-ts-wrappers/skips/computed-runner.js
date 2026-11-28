require("vitest")["test"].skip("x", () => {});
const run = require("vitest")["test"];
run.skip("bound", () => {});
