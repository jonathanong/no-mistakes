const assert = require("node:assert/strict");
const { test } = require("node:test");
const base = require("../../vitest.config.js");
const shard = require("./vitest-shard.config.cjs");

test("only the merged JavaScript report enforces the unchanged global floors", () => {
  assert.deepEqual(base.test.coverage.thresholds, {
    statements: 98,
    branches: 98,
    functions: 98,
    lines: 98,
  });
  assert.equal(shard.test.coverage.thresholds, undefined);
  assert.deepEqual(shard.test.coverage.include, base.test.coverage.include);
  assert.deepEqual(shard.test.coverage.reporter, base.test.coverage.reporter);
  assert.deepEqual(shard.test.include, base.test.include);
  assert.equal(shard.test.setupFiles, base.test.setupFiles);
});
