const assert = require("node:assert/strict");
const { test } = require("node:test");
const { outlierPartition, schedulingSummary } = require("./native-rust-partition.cjs");
const { casePlan, executionPlan } = require("./native-test-artifacts.cjs");
const { targets } = require("./native-rust-tests.cjs");
const costs = require("./native-rust-costs.json");
const finalInventory = require("../../test-cases/native-rust-partition/final-macos-inventory.json");
const skewed = require("../../test-cases/native-rust-partition/skewed.json");
const savedCases = require("../../test-cases/native-test-artifacts/fixture/cases.json");

function inventory(platform, cases) {
  const entries = targets(platform).map(({ selector }) => ({
    name: selector[1],
    kind: selector[0].slice(2),
    executable: `target/debug/deps/${selector[1]}-hash`,
  }));
  const groups = executionPlan(platform, entries).map((group) => ({
    ...group,
    tests: cases
      .filter(
        (row) =>
          group.executable.endsWith(`/${row.harness}-hash`) &&
          group.args
            .filter((arg) => !arg.startsWith("--"))
            .every((filter) => row.name.includes(filter)),
      )
      .map((row) => row.name),
  }));
  return { entries, groups };
}

test("saved lower-bound weights retain log provenance without claiming exact durations", () => {
  assert.equal(costs.defaultWeight, 1);
  assert.match(costs.interpretation, /not exact per-case durations or wall-time forecasts/);
  assert.deepEqual(
    costs.cases.map((row) => row.weight),
    [174, 78, 96, 71, 73],
  );
  for (const row of costs.cases) {
    const observed = row.observations.map((sample) => {
      assert.ok(
        costs.runs.some((run) => run.id === sample.runId && /^[a-f0-9]{40}$/.test(run.headSha)),
      );
      const elapsed = Date.parse(sample.completedAt) - Date.parse(sample.noticeAt);
      assert.ok(elapsed > 0);
      return Math.floor(costs.noticeThresholdSeconds + elapsed / 1000);
    });
    assert.equal(row.weight, Math.max(...observed));
  }
});

test("macOS two-shard scheduling balances the saved outliers in heuristic weight units", () => {
  const { entries, groups } = inventory("macOS", finalInventory.cases);
  const full = casePlan("macOS", entries, groups, 1, 1);
  const plans = [1, 2].map((index) => casePlan("macOS", entries, groups, index, 2));
  assert.equal(full.length, 373);
  assert.deepEqual(
    plans.map((plan) => plan.length),
    [185, 188],
  );
  assert.deepEqual(
    plans.map((plan) => schedulingSummary(plan, entries).assignedWeightUnits),
    [430, 430],
  );
  const identities = (rows) => rows.map((row) => `${row.executable}:${row.name}`).sort();
  assert.deepEqual(identities(plans.flat()), identities(full));
  assert.equal(new Set(identities(plans.flat())).size, 373);
  for (const plan of plans) {
    assert.deepEqual(
      plan,
      full.filter((entry) =>
        plan.some((row) => row.name === entry.name && row.executable === entry.executable),
      ),
    );
    assert.ok(
      plan.every((entry) => Object.keys(entry).sort().join(",") === "args,executable,name"),
    );
  }
  assert.deepEqual(
    plans.map((plan) =>
      schedulingSummary(plan, entries)
        .caseWeights.filter((row) => row.source === "observed-lower-bound")
        .map((row) => row.units)
        .sort((a, b) => b - a),
    ),
    [
      [174, 73],
      [96, 78, 71],
    ],
  );
});

test("Windows six-shard assignments and flags keep the exact existing modulo plan", () => {
  const cases = Object.entries(savedCases).flatMap(([harness, names]) =>
    names.map((name) => ({ harness, name })),
  );
  const { entries, groups } = inventory("Windows", cases);
  const full = casePlan("Windows", entries, groups, 1, 1);
  for (let index = 1; index <= 6; index++)
    assert.deepEqual(
      casePlan("Windows", entries, groups, index, 6),
      full.filter((_, offset) => offset % 6 === index - 1),
    );
  assert.ok(
    full
      .filter((entry) => entry.executable.includes("windows_job_object"))
      .every((entry) => entry.args.includes("--test-threads=1")),
  );
});

test("new cases use the equal placeholder and removed weighted cases are harmless", () => {
  const plans = [1, 2].map((index) =>
    outlierPartition(skewed.cases, skewed.entries, index, skewed.evidence),
  );
  assert.deepEqual(
    plans.map((plan) => plan.map((row) => row.name)),
    skewed.expectedNames,
  );
  assert.deepEqual(
    plans.map(
      (plan) => schedulingSummary(plan, skewed.entries, skewed.evidence).assignedWeightUnits,
    ),
    skewed.expectedWeightUnits,
  );
  const summary = schedulingSummary(plans.flat(), skewed.entries, skewed.evidence);
  assert.equal(summary.caseWeights.find((row) => row.name === "new_case").units, 1);
  assert.equal(
    summary.caseWeights.find((row) => row.harness === "other" && row.name === "alpha").units,
    1,
  );
});

test("stable harness identities break ties without depending on executable hashes or input order", () => {
  const expected = [1, 2].map((index) =>
    outlierPartition(skewed.cases, skewed.entries, index, skewed.evidence)
      .map((row) => `${row.executable}:${row.name}`)
      .sort(),
  );
  for (const index of [1, 2]) {
    const plan = outlierPartition(
      [...skewed.cases].reverse(),
      skewed.entries,
      index,
      skewed.evidence,
    );
    assert.deepEqual(
      plan.map((row) => `${row.executable}:${row.name}`).sort(),
      expected[index - 1],
    );
    const changed = (value) =>
      value.replace("postgres-aaa", "postgres-zzz").replace("other-bbb", "other-yyy");
    const cases = skewed.cases.map((row) => ({ ...row, executable: changed(row.executable) }));
    const entries = skewed.entries.map((row) => ({ ...row, executable: changed(row.executable) }));
    assert.deepEqual(
      outlierPartition(cases, entries, index, skewed.evidence).map((row) => row.name),
      skewed.expectedNames[index - 1],
    );
  }
});

test("invalid outlier configuration fails before selecting a partial plan", () => {
  assert.throws(() => outlierPartition(skewed.cases, skewed.entries, 0), /shard 1 or 2/);
  assert.throws(
    () => outlierPartition(skewed.cases, [], 1, skewed.evidence),
    /missing native harness/,
  );
  const row = skewed.evidence.cases[0];
  for (const cases of [[{ ...row, weight: 0 }], [row, row]])
    assert.throws(
      () => outlierPartition(skewed.cases, skewed.entries, 1, { defaultWeight: 1, cases }),
      /unique positive integers/,
    );
});
