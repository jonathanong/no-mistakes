const moduleName = "vitest";
const api = require(moduleName).vi;
let { mock } = api;
mock("./dep");
const { mock: loose } = require(moduleName);
loose("./dep");
