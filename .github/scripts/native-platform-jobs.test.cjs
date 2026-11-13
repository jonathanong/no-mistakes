const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");
const { workflowJob } = require("./workflow-contract.cjs");
const source = readFileSync(path.resolve(__dirname, "../workflows/ci.yml"), "utf8");
const job = (name) => workflowJob(source, name);
const families = ["native-build", "native-test-build", "native-rust-tests", "native-api-tests"];

test("each platform retains identical native steps without sharing a producer barrier", () => {
  for (const name of families) {
    const windows = job(name).split("    steps:\n")[1];
    const macos = job(`${name}-macos`).split("    steps:\n")[1];
    assert.equal(
      windows,
      macos
        .replaceAll("-cgu64-", "-")
        .replaceAll("-macos.outputs.artifact-run-id", ".outputs.artifact-run-id"),
    );
    for (const [suffix, os] of [
      ["", "windows-2025"],
      ["-macos", "macos-15"],
    ]) {
      const body = job(name + suffix);
      const operatingSystems = [
        ...body.matchAll(/^            os: (\S+)|^          - os: (\S+)/gm),
      ].map((row) => row[1] || row[2]);
      assert.ok(operatingSystems.length > 0);
      assert.ok(operatingSystems.every((value) => value === os));
      assert.match(body, /NO_MISTAKES_BUILD_NAPI: "1"/);
      assert.doesNotMatch(body, /docs-only/);
    }
  }
  for (const [consumer, producer] of [
    ["native-rust-tests", "native-test-build"],
    ["native-api-tests", "native-build"],
  ]) {
    for (const suffix of ["", "-macos"])
      assert.ok(job(consumer + suffix).includes(`needs: [changes, ${producer}${suffix}]`));
  }
});

test("native platform jobs retain complete six-shard Windows and two-shard macOS partitions", () => {
  for (const name of ["native-rust-tests", "native-api-tests"]) {
    for (const [suffix, count] of [
      ["", 6],
      ["-macos", 2],
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
      assert.match(body, /actions\/download-artifact@[a-f0-9]{40}/);
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

test("native aggregate requires all eight platform producer and consumer jobs", () => {
  const body = job("native-tests");
  const needs = body
    .match(/needs:\s*\[([^\]]+)\]/)[1]
    .split(",")
    .map((name) => name.trim())
    .filter(Boolean)
    .sort();
  assert.deepEqual(needs, families.flatMap((name) => [name, `${name}-macos`]).sort());
  assert.match(body, /name: Native tests\n/);
  assert.match(body, /if: \$\{\{ always\(\) \}\}/);
});
