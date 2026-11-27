const spec = "vitest";
const vt = require(spec);
vt.vi.mock.call(vt.vi, "./dep");
vt.vi.mock.apply(vt.vi, ["./dep"]);
