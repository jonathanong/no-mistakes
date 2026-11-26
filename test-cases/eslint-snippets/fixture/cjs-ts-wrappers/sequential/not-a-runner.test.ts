import vitest from "vitest";
vitest.sequential("default import is not a runner", () => {});
const { expect: e } = require("vitest");
e.sequential("expect is not a runner", () => {});
