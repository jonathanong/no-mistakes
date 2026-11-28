require("vitest").expect.mock("./dep");
require("vitest").mock("./dep");
require("@jest/globals").mock("./dep");
const { expect: assertion } = require("vitest");
assertion.mock("./dep");
const vt = require("vitest");
vt.expect.mock("./dep");
