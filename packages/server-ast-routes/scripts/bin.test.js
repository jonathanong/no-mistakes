const assert = require("node:assert/strict");
const { join } = require("node:path");

const PACKAGE_ROOT = join(__dirname, "..");
const { main } = require("./install");
const { testInstallerMainDownloads, testInstallerFailures } = require("../../../tests/js/test-helpers");

test("package bin points directly to the native executable target", () => {
  const pkg = require("../package.json");
  assert.deepEqual(pkg.bin, { "server-ast-routes": "bin/server-ast-routes" });
});

test("installer main downloads into the direct bin target", async () => {
  await testInstallerMainDownloads(main, "server-ast-routes", PACKAGE_ROOT, join, assert);
});

test("installer reports failures", async () => {
  await testInstallerFailures(main, assert);
});
