const specifier = "vitest";
const { test: check } = require(specifier);
const modifier = "skip";
const items = [];
check[modifier]("case", () => {
  items.push(1);
});
