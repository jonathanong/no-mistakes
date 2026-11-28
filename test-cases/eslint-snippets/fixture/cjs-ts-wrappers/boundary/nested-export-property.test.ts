import vt from "vitest";

const { expect } = require("vitest");
const { vi } = expect;
vi.mock("./dep");
require("vitest").expect.vi.mock("./dep");
const { expect: defaultExpect } = vt;
defaultExpect.mock("./dep");
