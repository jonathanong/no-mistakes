const spec = "./custom-test";
const custom = require(spec);
custom.test.skip("dynamic", () => {});
