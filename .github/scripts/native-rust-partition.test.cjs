const assert = require("node:assert/strict");
const { test } = require("node:test");
const { partition } = require("./native-rust-partition.cjs");
const costs = require("./native-rust-costs.json");
const savedCases = require("../../test-cases/native-test-artifacts/fixture/cases.json");

test("platform cases are deterministic and selected exactly once", () => {
  const cases = Object.entries(savedCases).flatMap(([executable, names]) =>
    names.map((name) => ({ executable, name })),
  );
  const full = partition(cases, 1, 1);
  assert.deepEqual(partition([...cases].reverse(), 1, 1), full);
  const selected = [1, 2].flatMap((index) => partition(cases, index, 2));
  assert.equal(new Set(selected).size, cases.length);
  assert.deepEqual(new Set(selected), new Set(full));
});

test("the historical Windows and macOS baseline records the full-build cost", () => {
  assert.equal(costs.baselineRun, 46045845814);
  assert.equal(costs.Windows.shardWallSeconds.length, 6);
  assert.ok(costs.Windows.compileJobSeconds > costs.Windows.precompileSeconds);
  assert.ok(costs.macOS.jobSeconds > costs.macOS.precompileSeconds + costs.macOS.testSeconds);
});
