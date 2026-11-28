const spec = "vitest";
const vt = require(spec);
vt.test("slow", () => {}, 999_999);
const moduleName = spec;
require(moduleName).test.bind(null)("bound", () => {}, 999_999);
