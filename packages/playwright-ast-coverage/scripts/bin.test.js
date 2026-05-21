const assert = require("node:assert/strict");
const { join } = require("node:path");

const PACKAGE_ROOT = join(__dirname, "..");
const { main } = require("./install");
const { testInstallerMainDownloads } = require("../../../tests/js/test-helpers");

test("package bin points directly to the native executable target", () => {
  const pkg = require("../package.json");
  assert.deepEqual(pkg.bin, { "playwright-ast-coverage": "bin/playwright-ast-coverage" });
});

test("installer main downloads into the direct bin target", async () => {
  await testInstallerMainDownloads(main, "playwright-ast-coverage", PACKAGE_ROOT, assert);
});
