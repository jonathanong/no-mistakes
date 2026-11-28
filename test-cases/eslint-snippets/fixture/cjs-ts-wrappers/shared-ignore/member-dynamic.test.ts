const spec = "vitest";
const check = require(spec).test;
const items = [];
check("case", () => {
  items.push(1);
});
check.skip("skipped", () => {
  items.push(1);
});
