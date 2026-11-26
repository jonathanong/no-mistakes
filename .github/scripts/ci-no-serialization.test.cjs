const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");
const YAML = require("yaml");
const ci = YAML.parse(readFileSync(path.resolve(__dirname, "../workflows/ci.yml"), "utf8"));

test("Tests with coverage waits directly for the coverage merge and the Linux API shards", () => {
  assert.equal(ci.jobs["javascript-tests"], undefined);
  assert.ok(ci.jobs["tests-gate"].needs.includes("javascript-coverage"));
  assert.ok(ci.jobs["tests-gate"].needs.includes("linux-api-tests"));
});

test("each cache is retained from its own build result", () => {
  for (const name of [
    "linux-runtime",
    "native-build",
    "native-test-build",
    "standalone-build",
    "rust-coverage-build",
  ])
    assert.equal(ci.jobs[`retain-${name}-cache`]?.needs, name, name);
  for (const [retention, job] of [
    ["retain-native-build-macos-cache", "native-api-tests-macos"],
    ["retain-native-test-build-macos-cache", "native-rust-tests-macos"],
  ]) {
    assert.equal(ci.jobs[retention]?.needs, job, job);
    assert.ok(ci.jobs[retention].if.includes(`needs.${job}.outputs.built == 'true'`), job);
  }
  assert.equal(ci.jobs["retain-caches"], undefined);
});
