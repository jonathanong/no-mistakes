const { group: { test } } = require("vitest");
test.sequential("nested name", () => {});
const { ...it } = require("vitest");
it.sequential("rest name", () => {});
