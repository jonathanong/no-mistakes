const spec = "vitest";
const vt = require(spec);
vt.vi.mock("./dep", () => ({ replaced: 1 }));
