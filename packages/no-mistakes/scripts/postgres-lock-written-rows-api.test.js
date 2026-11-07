const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { dirname, join } = require("node:path");
const { normalizeCheckFailurePaths } = require("./check-policy-test-helpers");
const test = globalThis.test || require("node:test").test;
const addon = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;
const cases = [
  ["resource-transactions", "checks-fail.yml", 1],
  ["resource-options", "resource-options.yml", 3],
  ["hashed-advisory", "checks-fail.yml", 4],
  ["helper-unwritten", "checks-fail.yml", 1],
  ["helper-written", "checks-fail.yml", 0],
  ["helper-imported-unwritten", "checks-fail.yml", 1],
  ["helper-imported-written", "checks-fail.yml", 0],
  ["atomic-cte", "checks-fail.yml", 4],
];

for (const [fixture, config, count] of cases) {
  test(
    `compiled written-row locks ${fixture} preserve CLI, async and batched parity`,
    { skip: !addon?.endsWith(".node") },
    async () => {
      const cjs = require("../index.js");
      const esm = await import("../index.mjs");
      const base = join(__dirname, "../../../test-cases/rules/postgres-lock-written-rows-only");
      const options = {
        root: join(base, "fixture", fixture),
        config: join(base, "configs", config),
      };
      const pending = cjs.check(options);
      assert.ok(pending instanceof Promise);
      const report = normalizeCheckFailurePaths(await pending);
      assert.equal(report.rules.length, count, JSON.stringify(report));
      assert.deepEqual(normalizeCheckFailurePaths(await esm.check(options)), report);
      const batched = await cjs.analyzeProject({ ...options, reports: [{ type: "check" }] });
      assert.deepEqual(normalizeCheckFailurePaths(batched.reports[0].result), report);
      const cli =
        process.env.NO_MISTAKES_TEST_CLI_PATH ||
        join(dirname(addon), process.platform === "win32" ? "no-mistakes.exe" : "no-mistakes");
      const output = spawnSync(
        cli,
        ["check", "--root", options.root, "--config", options.config, "--format", "json"],
        { encoding: "utf8" },
      );
      assert.ok([0, 1].includes(output.status), output.stderr);
      assert.deepEqual(normalizeCheckFailurePaths(JSON.parse(output.stdout)), report);
    },
  );
}
