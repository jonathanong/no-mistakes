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

test("one main-only retention job waits for the cache producers without gating tests", () => {
  const retain = ci.jobs["retain-caches"];
  assert.deepEqual(retain.needs, [
    "linux-runtime",
    "native-api-tests",
    "native-api-tests-macos",
    "native-test-build",
    "native-rust-tests-macos",
    "standalone-build",
    "rust-coverage-build",
  ]);
  assert.match(retain.if, /github.event_name == 'push' && github.ref == 'refs\/heads\/main'/);
  assert.match(retain.if, /contains\(needs\.\*\.result, 'success'\)/);
  for (const producer of ["native-api-tests", "native-api-tests-macos", "native-rust-tests-macos"])
    assert.ok(retain.if.includes(`needs.${producer}.outputs.built == 'true'`));
  assert.equal(retain.uses, "./.github/workflows/cache-retention.yml");
  assert.equal(retain.with.best_effort, true);
  assert.deepEqual(
    Object.keys(ci.jobs).filter((id) => id.startsWith("retain-")),
    ["retain-caches"],
  );
  assert.ok(!ci.jobs["tests-gate"].needs.includes("retain-caches"));
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
