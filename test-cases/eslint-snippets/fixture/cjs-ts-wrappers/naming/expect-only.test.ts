const { expect: e } = require("vitest");
e.mock("./dep");
const assertion = (require("vitest") as typeof import("vitest")).expect;
assertion.mock("./dep");
const plain = require("vitest").expect;
plain.mock("./dep");

import * as vt from "vitest";
const name = "vi";
const { [name as string]: api } = vt;
api.mock("./dep");
