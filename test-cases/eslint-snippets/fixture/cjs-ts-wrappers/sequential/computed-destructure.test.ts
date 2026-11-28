const key = "test";
const { [key]: test } = require("vitest");
test.sequential("hidden", () => {});
