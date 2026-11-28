const specifier = "vitest";
const { test: check, expect: other } = require(specifier);
const bag = [];
check("case", () => {
  bag.push(1);
});
check.skip("skipped", () => {
  bag.push(1);
});
other("not a test", () => {
  bag.push(1);
});
const { it: also } = require(specifier);
also("other", () => {
  bag.push(1);
});
