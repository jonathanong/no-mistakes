const moduleName = "vitest";
const sdk = require(moduleName);
const alias = sdk;
alias.mock("./dep");
