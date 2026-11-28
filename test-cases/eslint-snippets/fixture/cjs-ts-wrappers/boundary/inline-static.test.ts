require("vitest").vi.mock("./dep");
require("@jest/globals").jest.mock("./dep");
require("vitest").vitest.mock("./dep");
const api = require("vitest").vitest;
api.mock("./member");
const { vitest: renamed } = require("vitest");
renamed.mock("./pattern");
const vt = require("vitest");
vt.vitest.mock("./chain");
