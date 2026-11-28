const spec = "vitest";
const check = require(spec).test;
const items = [];
check("case", () => {
  items.push(1);
});
check.skip("skipped", () => {
  items.push(1);
});
const run = check;
run("aliased", () => {
  items.push(1);
});
const mid = check;
const again = mid;
again.skip("chained", () => {
  items.push(1);
});
