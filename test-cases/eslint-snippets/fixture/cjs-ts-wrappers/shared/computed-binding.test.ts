const moduleName = "vitest";
const { ["runner"]: runner } = require(moduleName);
const items = [];
runner.test("case", () => {
  items.push(1);
});
