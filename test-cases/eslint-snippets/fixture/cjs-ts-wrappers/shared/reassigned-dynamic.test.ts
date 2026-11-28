const specifier = "vitest";
let { test: check } = require(specifier);
check = localHelper;
const items = [];
check("case", () => {
  items.push(1);
});
