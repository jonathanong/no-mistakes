const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");
const { workflowJob } = require("./workflow-contract.cjs");
const source = readFileSync(path.resolve(__dirname, "../workflows/ci.yml"), "utf8");
const job = (name) => workflowJob(source, name);
const stepNames = (body) => [...body.matchAll(/^      - name: (.+)$/gm)].map((match) => match[1]);
const families = ["native-test-build", "native-rust-tests", "native-api-tests"];

test("the macOS jobs retain the Windows build and test phases on parallel runners", () => {
  for (const [windowsJobs, run] of [
    [["native-api-tests"], "native-api-tests"],
    [["native-test-build", "native-rust-tests"], "native-rust-tests"],
  ]) {
    const windows = windowsJobs.map(job);
    if (windowsJobs.length === 1) assert.match(windows[0], /^    needs: changes$/m);
    else assert.ok(windows[1].includes(`    needs: [changes, ${windowsJobs[0]}]`));
    const macos = job(`${run}-macos`);
    assert.match(macos, /^    needs: changes$/m);
    const expected = new Set(
      windows
        .flatMap(stepNames)
        .filter((name) => name !== "Exclude workspace from Microsoft Defender"),
    );
    if (run === "native-rust-tests") expected.add("Test native build diagnostics");

    const names = stepNames(macos);
    assert.equal(names.length, expected.size, run);
    assert.deepEqual(new Set(names), expected, run);
    for (const [body, os] of [
      ...windows.map((body) => [body, "windows-2025"]),
      [macos, "macos-15"],
    ]) {
      const runners = [...body.matchAll(/^          (?:- |  )os: (.+)$/gm)];
      assert.ok(runners.length > 0);
      assert.ok(runners.every((row) => row[1] === os));
      if (run === "native-rust-tests") assert.doesNotMatch(body, /NO_MISTAKES_BUILD_NAPI/);
      else assert.match(body, /^      NO_MISTAKES_BUILD_NAPI: "1"$/m);
      assert.doesNotMatch(body, /^    if: .*docs-only/m);
    }
  }
});

test("native jobs use one job for each native test suite", () => {
  for (const [name, windowsCount] of [
    ["native-rust-tests", 1],
    ["native-api-tests", 1],
  ]) {
    for (const [suffix, count] of [
      ["", windowsCount],
      ["-macos", 1],
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

test("Tests with coverage requires the Windows phases and both macOS build-and-test jobs", () => {
  const body = job("tests-gate");
  const needs = body
    .match(/needs:\s*\[([^\]]+)\]/)[1]
    .split(",")
    .map((name) => name.trim())
    .filter(Boolean);
  for (const name of [
    "changes",
    ...families,
    "native-api-tests-macos",
    "native-rust-tests-macos",
  ]) {
    assert.ok(needs.includes(name), name);
  }
  assert.match(body, /name: Tests with coverage\n/);
  assert.match(body, /^    if: \$\{\{ !cancelled\(\) \}\}$/m);
});
