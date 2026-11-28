const moduleName = "vitest";
const api = require(moduleName).vi;
const { mock } = api;
mock("./dep");
