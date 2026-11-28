const spec = "vitest";
let runner = require(spec);
const check = runner.test;
const items = [];
check("case", () => {
  items.push(1);
});
let namespace = require(spec);
const { test: fromLet } = namespace;
fromLet("from let", () => {
  items.push(1);
});
