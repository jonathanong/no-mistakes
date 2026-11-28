const moduleName = "vitest";
const sdk = require(moduleName);
const cast = sdk as any;
cast.mock("react", () => ({ ready: true }));
const alias = sdk;
function register() {
  const sdk = alias;
  sdk.mock("react", () => ({ ready: true }));
}
const api = require(moduleName).vi;
api.mock("react", () => ({ ready: true }));
const { vi: renamed } = sdk;
renamed.mock("react", () => ({ ready: true }));
