const specifier = "vitest";
const { test } = require(specifier);
const bag = [];
test("case", () => {
  bag.push(1);
});
