const specifier = "vitest";
const key = "runner";
let { runner } = require(specifier);
runner = function local() {};
const items = [];
runner.test("reassigned", () => {
  items.push(1);
});
let { [key]: named } = require(specifier);
named.test("computed", () => {
  items.push(1);
});
