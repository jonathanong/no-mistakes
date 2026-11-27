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

test("Clippy jobs share check metadata while retaining separate lint invocations", () => {
  const rust = ci.jobs["quality-rust"];
  const cache = rust.steps.find(({ name }) => name === "Cache Rust quality artifacts");
  assert.equal(cache.with["env-vars"], "NM_CI_RUST_CACHE_PROFILE");
  assert.equal(
    cache.env.NM_CI_RUST_CACHE_PROFILE,
    "${{ startsWith(matrix.command, 'cargo clippy ') && 'clippy-check' || '' }}",
  );
  assert.equal(rust.needs, "changes");
});

test("the source policy job scans committed binaries before installing tooling", () => {
  const steps = ci.jobs["quality-source"].steps.map(({ name }) => name);
  assert.deepEqual(steps.slice(0, 2), ["Checkout", "Check committed binary files"]);
});
