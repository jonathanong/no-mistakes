const assert = require("node:assert/strict");
const { existsSync, readFileSync } = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");
const YAML = require("yaml");
const workflows = path.resolve(__dirname, "../workflows");
const load = (file) =>
  existsSync(path.join(workflows, file))
    ? YAML.parse(readFileSync(path.join(workflows, file), "utf8"))
    : { jobs: {} };
const ci = load("ci.yml");
const jobIn = (name, files = [ci]) => files.map((f) => f.jobs[name]).find(Boolean);
const matrix = (job) => job?.strategy?.matrix;
const entries = (job) => matrix(job)?.include?.length ?? 0;
const shards = (job) => matrix(job)?.shard?.length ?? 0;

test("parallel work is never folded into fewer jobs (docs/development/ci-builds.md)", () => {
  const minimums = [
    ["quality-rust", entries, 7],
    ["quality-js", entries, 4],
    ["javascript-shards", shards, 4],
    ["linux-api-tests", shards, 4],
    ["rust-coverage-shards", shards, 6],
  ];
  for (const [name, count, min] of minimums) {
    const job = jobIn(name);
    assert.ok(job, `${name} must exist in ci.yml`);
    assert.ok(count(job) >= min, `${name} needs at least ${min} parallel units`);
  }
  assert.deepEqual(ci.jobs["javascript-tests"].needs, ["javascript-coverage", "linux-api-tests"]);
});

test("benchmark shards stay one matrix entry each, in ci.yml or benchmarks.yml", () => {
  const job = jobIn("benchmark-shards", [ci, load("benchmarks.yml")]);
  assert.ok(job, "benchmark-shards must exist in ci.yml or benchmarks.yml");
  assert.ok(entries(job) >= 24, "benchmark-shards needs at least 24 matrix entries");
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
