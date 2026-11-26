const { expect: e } = require("vitest");
e.mock("./dep", () => ({ kept: 1 }));
import vt = require("vitest");
vt.vi.mock("./dep", () => ({ replaced: 1 }));
