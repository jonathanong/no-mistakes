const moduleName = "vitest";
const sdk = require(moduleName);
const api = require(moduleName).vi;
api.mock("react");
const { vi: renamed } = sdk;
renamed.mock("react");
