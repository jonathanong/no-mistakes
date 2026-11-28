const spec = "vitest";
const sdk = require(spec);
sdk.mock.call(sdk, "./dep", () => ({}));
sdk.doMock.apply(sdk, ["./dep", () => ({})]);
sdk.mock.call(sdk, ...["react"]);
