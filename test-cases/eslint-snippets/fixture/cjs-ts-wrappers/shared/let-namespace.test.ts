const specifier = "vitest";
let { runner } = require(specifier);
const items = [];
runner.test("case", () => {
  items.push(1);
});
