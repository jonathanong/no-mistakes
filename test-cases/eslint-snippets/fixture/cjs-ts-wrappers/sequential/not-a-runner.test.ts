import vitest from "vitest";
vitest.sequential("default import is not a runner", () => {});
const { expect: e } = require("vitest");
e.sequential("expect is not a runner", () => {});
const check = require("vitest").expect;
check.sequential("member expect is not a runner", () => {});
