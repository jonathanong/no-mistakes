const spec = "vitest";
const vt = require(spec);
const items = [];
vt.test("dynamic", () => { items.push(1); });
