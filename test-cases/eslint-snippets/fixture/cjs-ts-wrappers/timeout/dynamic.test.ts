const spec = "vitest";
const vt = require(spec);
vt.test("slow", () => {}, 999_999);
