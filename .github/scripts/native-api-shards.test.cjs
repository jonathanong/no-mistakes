const assert = require("node:assert/strict");
const { test } = require("node:test");
const fs = require("node:fs");
const path = require("node:path");
const { partition, key } = require("./native-api-shards.cjs");
const { platformFiles, linuxFiles } = require("./native-api-files.cjs");
const inventory = require("../../test-cases/native-api-shards/inventory.json");

test("native API shards collect every legacy file including the large real addon suite", () => {
  assert.equal(platformFiles.length, 8);
  assert.equal(new Set(platformFiles).size, 8);
  assert.ok(platformFiles.includes("packages/no-mistakes/scripts/real-napi-api.test.js"));
  assert.ok(
    platformFiles.includes("packages/no-mistakes/scripts/dependabot-cooldown-policy-api.test.js"),
  );
  const source = fs.readFileSync(path.join(__dirname, "native-api-shards.cjs"), "utf8");
  assert.match(source, /"list",\s*\.\.\.files,\s*"--no-static-parse"/);
  assert.match(source, /"run",\s*\.\.\.runFiles,\s*`--testNamePattern=/);
  assert.match(source, /NO_MISTAKES_VITEST_LIST:\s*"1"/);
});

test("case shards partition all collected names exactly once with regex escaping", () => {
  for (const count of [1, 2, 3, 6]) {
    const plans = Array.from({ length: count }, (_, index) =>
      partition(inventory, index + 1, count),
    );
    for (const row of inventory) {
      for (const name of [row.name, key(row.name)])
        assert.equal(plans.filter((plan) => new RegExp(plan.pattern).test(name)).length, 1, name);
    }
    for (const plan of plans) assert.equal(plan.total, inventory.length);
    const matching = plans.find((plan) => new RegExp(plan.pattern).test("test 1"));
    assert.ok(matching);
    assert.ok(plans.every((plan) => !new RegExp(plan.pattern).test("test 100")));
  }
});

test("API case partition rejects empty, invalid and oversized inventories", () => {
  assert.throws(() => partition([], 1, 1), /inventory/);
  assert.throws(() => partition(inventory, 0, 1), /shard/);
  assert.throws(() => partition(inventory, 2, 1), /shard/);
  assert.throws(() => partition(inventory, 1, 0), /shard/);
  assert.throws(
    () => partition([{ file: "file", name: "x".repeat(25000) }], 1, 1, "win32"),
    /argument budget/,
  );
  assert.ok(
    partition([{ file: "file", name: "x".repeat(25000) }], 1, 1, "darwin").pattern.length > 24000,
  );
  assert.throws(
    () => partition([{ file: "file", name: "x".repeat(120000) }], 1, 1, "darwin"),
    /argument budget/,
  );
  assert.throws(
    () => partition([{ file: "file", name: "é".repeat(61000) }], 1, 1, "darwin"),
    /argument budget/,
  );
});

test("native API runner collects dynamic cases and executes case filters without rebuilding", () => {
  const os = require("node:os");
  const { spawnSync } = require("node:child_process");
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "native-api-shards-"));
  fs.cpSync(path.resolve(__dirname, "../../test-cases/native-api-shards/fixture"), directory, {
    recursive: true,
  });
  const script = path.join(__dirname, "native-api-shards.cjs");
  const invoke = (index, env = {}) =>
    spawnSync(process.execPath, ["driver.cjs", script, String(index), "3"], {
      cwd: directory,
      encoding: "utf8",
      env: { ...process.env, ...env },
    });
  try {
    const executed = [];
    for (let index = 1; index <= 3; index++) {
      const result = invoke(index);
      assert.equal(result.status, 0, result.stderr);
      executed.push(
        ...JSON.parse(
          fs.readFileSync(path.join(directory, "target/native-api-tests.json"), "utf8"),
        ),
      );
    }
    const sorted = (rows) => rows.map((row) => `${row.file}:${row.name}`).sort();
    assert.deepEqual(sorted(executed), sorted(inventory));
    const commands = fs
      .readFileSync(path.join(directory, "commands.jsonl"), "utf8")
      .trim()
      .split("\n")
      .map(JSON.parse);
    assert.equal(commands.length, 6);
    for (const args of commands) {
      assert.deepEqual(args.slice(1, 9), platformFiles);
      assert.ok(args.includes("--coverage=false"));
      assert.ok(!args.some((arg) => arg.includes("cargo")));
    }
    const failed = invoke(1, { NATIVE_API_FIXTURE_EXIT: "17" });
    assert.equal(failed.status, 17, failed.stderr);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("Linux runner shards execute the complete expanded inventory once and reject unknown scopes", () => {
  const os = require("node:os");
  const { spawnSync } = require("node:child_process");
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "native-api-linux-shards-"));
  fs.cpSync(path.resolve(__dirname, "../../test-cases/native-api-shards/fixture"), directory, {
    recursive: true,
  });
  const script = path.join(__dirname, "native-api-shards.cjs");
  const expected = require("../../test-cases/native-api-shards/fixture/linux-inventory.json");
  const invoke = (index, suite) =>
    spawnSync(process.execPath, ["driver.cjs", script, String(index), "4", suite], {
      cwd: directory,
      encoding: "utf8",
      env: { ...process.env, NATIVE_API_FIXTURE_SUITE: "linux" },
    });
  try {
    assert.equal(new Set(linuxFiles).size, linuxFiles.length);
    assert.deepEqual([...new Set(expected.map((row) => row.file))].sort(), [...linuxFiles].sort());
    assert.ok(platformFiles.every((file) => linuxFiles.includes(file)));
    const executed = [];
    for (let index = 1; index <= 4; index++) {
      const result = invoke(index, "linux");
      assert.equal(result.status, 0, result.stderr);
      executed.push(
        ...JSON.parse(
          fs.readFileSync(path.join(directory, "target/native-api-tests.json"), "utf8"),
        ),
      );
    }
    const sorted = (rows) => rows.map((row) => `${row.file}:${row.name}`).sort();
    assert.deepEqual(sorted(executed), sorted(expected));
    const commands = fs
      .readFileSync(path.join(directory, "commands.jsonl"), "utf8")
      .trim()
      .split("\n")
      .map(JSON.parse);
    assert.equal(commands.length, 8);
    const runFileUnion = new Set();
    for (let offset = 0; offset < commands.length; offset += 2) {
      const listArgs = commands[offset];
      const runArgs = commands[offset + 1];
      assert.deepEqual(listArgs.slice(1, 1 + linuxFiles.length), linuxFiles);
      const patternIndex = runArgs.findIndex((arg) => arg.startsWith("--testNamePattern="));
      const shardRunFiles = runArgs.slice(1, patternIndex);
      const expectedFiles = linuxFiles.filter((file) =>
        partition(expected, offset / 2 + 1, 4).files.includes(file),
      );
      assert.deepEqual(shardRunFiles, expectedFiles);
      assert.ok(shardRunFiles.length < linuxFiles.length);
      for (const file of shardRunFiles) runFileUnion.add(file);
    }
    assert.deepEqual([...runFileUnion].sort(), [...linuxFiles].sort());
    const invalid = invoke(1, "unknown");
    assert.equal(invalid.status, 1);
    assert.match(invalid.stderr, /suite must be platform or linux/);
    const invalidCli = spawnSync(process.execPath, [script, "1", "4", "unknown"], {
      cwd: directory,
      encoding: "utf8",
    });
    assert.equal(invalidCli.status, 1);
    assert.match(invalidCli.stderr, /suite must be linux/);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
