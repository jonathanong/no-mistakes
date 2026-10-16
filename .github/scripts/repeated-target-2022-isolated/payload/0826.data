const assert = require("node:assert/strict");

test("package bin points to the JavaScript launcher", () => {
  const pkg = require("../package.json");
  assert.deepEqual(pkg.bin, { "no-mistakes": "bin/no-mistakes.js" });
});
