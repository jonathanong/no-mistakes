const assert = require("node:assert/strict");
const { join } = require("node:path");

const PACKAGE_ROOT = join(__dirname, "..");
const { main } = require("./install");
const { testInstallerFailures } = require("../../../tests/js/test-helpers");

test("package bin points directly to the native executable target", () => {
  const pkg = require("../package.json");
  assert.deepEqual(pkg.bin, { "next-to-fetch": "bin/next-to-fetch" });
});

test("installer main downloads into the direct bin target", async () => {
  const calls = [];
  await main(async (...args) => {
    calls.push(args);
    return "/tmp/next-to-fetch";
  });
  assert.equal(calls.length, 1);
  assert.deepEqual(calls[0].slice(0, 2), ["next-to-fetch", "jonathanong/no-mistakes"]);
  assert.equal(calls[0][2].vendorDir, join(PACKAGE_ROOT, "bin"));
  assert.equal(calls[0][2].destinationName, "next-to-fetch");
});

test("installer reports failures", async () => {
  await testInstallerFailures(main, assert);
});
