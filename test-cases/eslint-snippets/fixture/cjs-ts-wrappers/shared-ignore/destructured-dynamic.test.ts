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
const run = check;
run("aliased", () => {
  bag.push(1);
});
const mid = check;
const again = mid;
again.skip("chained", () => {
  bag.push(1);
});
const look = other;
look("not a test", () => {
  bag.push(1);
});
const namespace = require(specifier);
const { test: fromNamespace } = namespace;
fromNamespace("from namespace", () => {
  bag.push(1);
});
