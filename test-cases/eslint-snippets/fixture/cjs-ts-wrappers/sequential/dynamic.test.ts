const spec = "vitest";
const vt = require(spec);
vt.test.sequential("dynamic", () => {});
const { test } = require(spec);
test.sequential("dynamic destructure", () => {});

const moduleName = "vitest";
const sdk = require(moduleName);
const alias = sdk;
function register() {
  const sdk = alias;
  sdk.test.sequential("shadowed alias", () => {});
}
void register;
