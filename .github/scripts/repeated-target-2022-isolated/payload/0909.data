const assert = require("node:assert/strict");
const { existsSync, readFileSync } = require("node:fs");
const { resolve } = require("node:path");
const test = globalThis.test || require("node:test").test;

const packageRoot = resolve(__dirname, "..");
const packageJson = JSON.parse(readFileSync(resolve(packageRoot, "package.json"), "utf8"));
const policyExports = ["./test-timeout-policy", "./test-timeout-policy/plugin"];

test("timeout policy public entrypoints are included in the npm package contract", async () => {
  for (const subpath of policyExports) {
    const entry = packageJson.exports[subpath];
    assert.ok(entry, `${subpath} must be exported`);
    assert.equal(typeof entry.import, "string");
    assert.equal(typeof entry.types, "string");
    for (const target of [entry.import, entry.types]) {
      assert.ok(packageJson.files.some((pattern) => pattern.includes("test-timeout-policy/")));
      assert.ok(existsSync(resolve(packageRoot, target)), `${target} must exist`);
    }
  }
  assert.equal(packageJson.peerDependenciesMeta.vitest.optional, true);
});

test("timeout policy imports expose their declared runtime API", async () => {
  const policy = await import("no-mistakes/test-timeout-policy");
  const plugin = await import("no-mistakes/test-timeout-policy/plugin");
  const coreNames = [
    "assertTestTimeout",
    "assertTimeoutConfig",
    "snapshotTimeoutOptions",
    "protectTimeoutConfig",
    "guardRegistration",
    "guardPublicRunner",
    "guardRuntimeConfig",
    "guardVitestExports",
    "protectRunnerTimeouts",
  ];
  assert.deepEqual(Object.keys(policy).sort(), coreNames.toSorted());
  assert.deepEqual(Object.keys(plugin).sort(), [
    "browserTestTimeoutPolicyPlugin",
    "testTimeoutPolicyPlugin",
  ]);
  for (const name of coreNames) {
    assert.equal(typeof policy[name], "function", `${name} runtime export`);
    assert.match(
      readFileSync(
        resolve(packageRoot, packageJson.exports["./test-timeout-policy"].types),
        "utf8",
      ),
      new RegExp(`\\b${name}\\b`),
      `${name} declaration`,
    );
  }
  for (const name of ["testTimeoutPolicyPlugin", "browserTestTimeoutPolicyPlugin"]) {
    assert.equal(typeof plugin[name], "function", `${name} runtime export`);
    assert.match(
      readFileSync(
        resolve(packageRoot, packageJson.exports["./test-timeout-policy/plugin"].types),
        "utf8",
      ),
      new RegExp(`\\b${name}\\b`),
      `${name} declaration`,
    );
  }
});
