const spec = "vitest";
let vt = require(spec);
const items = [];
vt.test("a", () => {
  items.push(1);
});
