const spec = "vitest";
const sdk = require(spec);
sdk.mock.call(sdk, "./dep", () => ({ replaced: 1 }));
sdk.unstable_mockModule.apply(sdk, ["./dep", () => ({ replaced: 1 })]);
