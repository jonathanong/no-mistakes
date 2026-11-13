const assert = require("node:assert/strict");
const { test } = require("node:test");
const fs = require("node:fs");
const path = require("node:path");
const { partition, files, key } = require("./native-api-shards.cjs");
const inventory = require("../../test-cases/native-api-shards/inventory.json");

test("native API shards collect every legacy file including the large real addon suite", () => {
  assert.equal(files.length, 7);
  assert.equal(new Set(files).size, 7);
  assert.ok(files.includes("packages/no-mistakes/scripts/real-napi-api.test.js"));
  const source = fs.readFileSync(path.join(__dirname, "native-api-shards.cjs"), "utf8");
  assert.match(source, /"list",\s*\.\.\.files,\s*"--no-static-parse"/);
  assert.match(source, /"run",\s*\.\.\.files,\s*`--testNamePattern=/);
});

test("case shards partition all collected names exactly once with regex escaping", () => {
  for (const count of [1, 2, 3]) {
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
    () => partition([{ file: "file", name: "x".repeat(25000) }], 1, 1),
    /Windows command budget/,
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
      assert.deepEqual(args.slice(1, 8), files);
      assert.ok(args.includes("--coverage=false"));
      assert.ok(!args.some((arg) => arg.includes("cargo")));
    }
    const failed = invoke(1, { NATIVE_API_FIXTURE_EXIT: "17" });
    assert.equal(failed.status, 17, failed.stderr);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
