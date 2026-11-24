const { workflowJob } = require("./workflow-contract.cjs");
const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");
const root = path.resolve(__dirname, "../..");
const workflow = readFileSync(path.join(root, ".github/workflows/ci.yml"), "utf8");
const job = (name) => workflowJob(workflow, name);
const entries = (name) => (job(name).match(/^ {10}- name: /gm) ?? []).length;

test("parallel work is never folded into fewer jobs (docs/development/ci-builds.md)", () => {
  assert.equal(entries("quality-rust"), 7);
  assert.equal(entries("quality-js"), 4);
  assert.equal(entries("benchmark-shards"), 24);
  assert.match(job("javascript-shards"), /shard: \[1, 2, 3, 4\]/);
  assert.match(job("linux-api-tests"), /shard: \[1, 2, 3, 4\]/);
  assert.match(job("rust-coverage-shards"), /shard: \[1, 2, 3, 4, 5, 6\]/);
  assert.match(job("javascript-tests"), /needs: \[javascript-coverage, linux-api-tests\]/);
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
    assert.match(job(`retain-${name}-cache`), new RegExp(`needs: ${name}\\n`), name);
  assert.doesNotMatch(workflow, /^ {2}retain-caches:/m);
});
