const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");
const { workflowJob } = require("./workflow-contract.cjs");
const source = readFileSync(path.resolve(__dirname, "../workflows/ci.yml"), "utf8");
const job = (name) => workflowJob(source, name);
const workflow = require("yaml").parse(source);
const families = ["native-build", "native-test-build", "native-rust-tests", "native-api-tests"];

test("the macOS jobs retain the Windows build and test phases on parallel runners", () => {
  for (const [build, run] of [
    ["native-build", "native-api-tests"],
    ["native-test-build", "native-rust-tests"],
  ]) {
    const windows = [workflow.jobs[build], workflow.jobs[run]];
    assert.deepEqual(windows[1].needs, ["changes", build]);
    const macos = workflow.jobs[`${run}-macos`];
    assert.equal(macos.needs, "changes");
    const expected = new Set(
      windows.flatMap((job) =>
        job.steps
          .map((step) => step.name)
          .filter((name) => name !== "Exclude workspace from Microsoft Defender"),
      ),
    );
    const names = macos.steps.map((step) => step.name);
    assert.equal(names.length, expected.size, run);
    assert.deepEqual(new Set(names), expected, run);
    for (const [job, os] of [...windows.map((job) => [job, "windows-2025"]), [macos, "macos-15"]]) {
      assert.ok(job.strategy.matrix.include.length > 0);
      assert.ok(job.strategy.matrix.include.every((row) => row.os === os));
      assert.equal(job.env.NO_MISTAKES_BUILD_NAPI, "1");
      assert.doesNotMatch(job.if, /docs-only/);
    }
  }
});

test("native jobs keep six Windows shards and one macOS job per test suite", () => {
  for (const [name, macosCount] of [
    ["native-rust-tests", 1],
    ["native-api-tests", 1],
  ]) {
    for (const [suffix, count] of [
      ["", 6],
      ["-macos", macosCount],
    ]) {
      const body = job(name + suffix);
      const partitions = [...body.matchAll(/shard:\s+(\d+)\s+total:\s+(\d+)/g)].map((row) => [
        Number(row[1]),
        Number(row[2]),
      ]);
      assert.deepEqual(
        partitions,
        Array.from({ length: count }, (_, offset) => [offset + 1, count]),
      );
      assert.ok(body.includes("${{ matrix.shard }} ${{ matrix.total }}"));
      if (name === "native-api-tests") {
        assert.match(body, /python .github\/scripts\/download-native-runtime.py/);
      } else {
        assert.match(body, /actions\/download-artifact@[a-f0-9]{40}/);
      }
    }
  }
  for (const suffix of ["", "-macos"]) {
    const rust = job(`native-rust-tests${suffix}`);
    assert.match(rust, /target\/native-rust-shard\.json/);
    assert.match(rust, /target\/native-rust-results\.jsonl/);
    const api = job(`native-api-tests${suffix}`);
    assert.match(api, /target\/native-api-inventory\.json/);
    assert.match(api, /target\/native-api-shard\.json/);
  }
});

test("Tests with coverage requires the Windows phases and both macOS build-and-test jobs and both smoke jobs", () => {
  const body = job("tests-gate");
  const needs = body
    .match(/needs:\s*\[([^\]]+)\]/)[1]
    .split(",")
    .map((name) => name.trim())
    .filter(Boolean);
  for (const name of [
    "changes",
    "macos-smoke",
    "windows-smoke",
    ...families,
    "native-api-tests-macos",
    "native-rust-tests-macos",
  ]) {
    assert.ok(needs.includes(name), name);
  }
  assert.match(body, /name: Tests with coverage\n/);
  assert.match(body, /^    if: \$\{\{ !cancelled\(\) \}\}$/m);
});
