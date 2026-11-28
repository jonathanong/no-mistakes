const source = "vitest";
const { test } = require(source);
const rows = [[1]];
test.sequential.each(rows)("case", () => {});
