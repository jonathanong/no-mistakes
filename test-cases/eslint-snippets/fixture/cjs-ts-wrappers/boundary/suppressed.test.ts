const spec = "vitest";
const sdk = require(spec);
// no-mistakes-disable-next-line module-mock-boundary -- dynamic framework load
sdk.mock("./dep", () => ({}));
// no-mistakes-disable-next-line module-mock-boundary -- dynamic framework load
sdk.mock.call(sdk, "./dep", () => ({}));
