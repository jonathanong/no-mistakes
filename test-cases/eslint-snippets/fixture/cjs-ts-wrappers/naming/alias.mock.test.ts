const moduleName = "vitest";
const vt = require(moduleName);
const mock = vt.vi.mock;
mock("./dep");
