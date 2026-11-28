const { expect: e } = require("vitest");
e.mock("./dep");
import vt = require("vitest");
vt.vi.mock("./dep");
const vt2 = require("vitest");
vt2.vi.mock("./other");
