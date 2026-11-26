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

test("each cache is retained by its own job right after its producer", () => {
  for (const name of [
    "linux-runtime",
    "native-build",
    "native-build-macos",
    "native-test-build",
    "native-test-build-macos",
    "standalone-build",
    "rust-coverage-build",
  ])
    assert.equal(ci.jobs[`retain-${name}-cache`]?.needs, name, name);
  assert.equal(ci.jobs["retain-caches"], undefined);
});
