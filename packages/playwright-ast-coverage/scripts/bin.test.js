const assert = require("node:assert/strict");
const { join } = require("node:path");

const PACKAGE_ROOT = join(__dirname, "..");
const { main } = require("./install");

test("package bin points directly to the native executable target", () => {
  const pkg = require("../package.json");
  assert.deepEqual(pkg.bin, { "playwright-ast-coverage": "bin/playwright-ast-coverage" });
});

test("installer main downloads into the direct bin target", async () => {
  const calls = [];
  await main(async (...args) => {
    calls.push(args);
    return "/tmp/playwright-ast-coverage";
  });
  assert.equal(calls.length, 1);
  assert.deepEqual(calls[0].slice(0, 2), ["playwright-ast-coverage", "jonathanong/no-mistakes"]);
  assert.equal(calls[0][2].vendorDir, join(PACKAGE_ROOT, "bin"));
  assert.equal(calls[0][2].destinationName, "playwright-ast-coverage");
});
