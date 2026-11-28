const moduleName = "vitest";
const sdk = require(moduleName);
let alias = sdk;
alias.mock("react", () => ({ ready: true }));
