const moduleName = "vitest";
const sdk = require(moduleName);
const cast = sdk as any;
cast.mock("react");
const alias = sdk;
function register() {
  const sdk = alias;
  sdk.mock("react");
}
const api = require(moduleName).vi;
api.mock("react");
const { vi: renamed } = sdk;
renamed.mock("react");
