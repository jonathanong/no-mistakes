const spec = "vitest";
const vt = require(spec);
const items = [];
vt.test("dynamic", () => { items.push(1); });
vt.test.skip("skipped", () => { items.push(1); });
vt.test.each([1])("row", () => { items.push(1); });
