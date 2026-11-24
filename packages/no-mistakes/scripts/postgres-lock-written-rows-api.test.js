const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { dirname, join } = require("node:path");
const { normalizeCheckFailurePaths } = require("./check-policy-test-helpers");
const test = globalThis.test || require("node:test").test;
const addon = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;
const cases = [
  ["advisory-ban-existing", "advisory-ban.yml", 11, 0],
  ["locking-cte-existing", "locking-cte.yml", 0, 2],
  ["share-lock-sequences", "share-advise.yml", 0, 28],
  ["share-lock-sequences", "share-fail.yml", 12, 16],
  ["share-lock-sequences", "share-read-only-fail.yml", 0, 1],
  ["share-lock-sequences", "disabled.yml", 0, 1],
  ["key-share-key-writes", "share-key-writes.yml", 0, 4],
  ["key-share-key-writes", "share-advise.yml", 0, 1],
  ["redundant-prelocks", "redundant-advise.yml", 0, 20],
  ["redundant-prelock-boundaries", "redundant-advise.yml", 0, 10],
  ["redundant-prelocks", "redundant-no-catalog.yml", 0, 20],
  ["redundant-prelocks", "disabled.yml", 0, 1],
  ["resource-transactions", "checks-fail.yml", 1],
  ["resource-options", "resource-options.yml", 4],
  ["caller-options", "caller-options.yml", 13],
  ["caller-options", "caller-options-written.yml", 1],
  ["hashed-advisory", "checks-fail.yml", 4],
  ["helper-unwritten", "checks-fail.yml", 1],
  ["helper-written", "checks-fail.yml", 0],
  ["helper-imported-unwritten", "checks-fail.yml", 1],
  ["helper-imported-written", "checks-fail.yml", 0],
  ["atomic-cte", "checks-fail.yml", 4],
  ["parameter-transactions", "parameter-transactions-disabled.yml", 0],
  ["parameter-transactions", "parameter-transactions.yml", 0],
  ["parameter-transactions", "parameter-transactions-no-catalog.yml", 0],
  ["membership-followups", "parameter-transactions-no-catalog.yml", 0, 4],
  ["ddl-related", "checks-fail.yml", 0],
  ["ddl-guard", "checks-fail.yml", 0],
  ["ddl-catalog", "ddl-catalog.yml", 1],
  ["ddl-negative", "checks-fail.yml", 18],
  ["trigger-writes", "trigger-writes.yml", 9],
  ["trigger-upsert", "trigger-writes.yml", 2],
  ["trigger-composite", "trigger-writes.yml", 3],
  ["trigger-dynamic-setting", "trigger-writes.yml", 2],
  ["trigger-replication-role", "trigger-writes.yml", 2],
  ["mixed-ddl-trigger", "mixed-ddl-trigger-complete.yml", 0],
  ["mixed-ddl-trigger", "mixed-ddl-trigger-ordering.yml", 1],
  ["mixed-ddl-trigger", "mixed-ddl-trigger-ignored-check.yml", 0],
];

for (const [fixture, config, ruleCount, advisoryCount] of cases) {
  test(
    `compiled written-row locks ${fixture} (${config}) preserve CLI, async and batched parity`,
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
      assert.equal(report.rules.length, ruleCount, JSON.stringify(report));
      if (advisoryCount !== undefined)
        assert.equal(report.advisories.length, advisoryCount, JSON.stringify(report));
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
