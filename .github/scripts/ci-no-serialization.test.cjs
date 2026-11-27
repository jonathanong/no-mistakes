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

test("PR standalone slices build and test in independent runners with the main cache", () => {
  const pr = ci.jobs["standalone-pr"];
  const main = ci.jobs["standalone-build"];
  assert.equal(
    pr.if,
    "${{ github.event_name == 'pull_request' && needs.changes.outputs.rust == 'true' }}",
  );
  assert.equal(
    main.if,
    "${{ github.event_name != 'pull_request' && needs.changes.outputs.rust == 'true' }}",
  );
  assert.equal(pr.needs, "changes");
  assert.deepEqual(pr.strategy, { "fail-fast": false, matrix: { shard: [1, 2, 3] } });
  assert.deepEqual(pr.env, main.env);
  const restore = pr.steps.find(({ id }) => id === "standalone-lib-workspace-cache");
  assert.deepEqual(restore.with, main.steps.find(({ id }) => id === restore.id).with);
  assert.ok(pr.steps.every(({ uses }) => !uses?.startsWith("actions/upload-artifact@")));
  assert.ok(pr.steps.at(-1).run.endsWith("standalone-shards.cjs local ${{ matrix.shard }}"));
  assert.ok(ci.jobs["tests-gate"].needs.includes("standalone-pr"));
});
