const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { dirname, join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const addon = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;

test(
  "compiled mutable registry effort preserves async CLI and shared-report parity",
  {
    skip: !addon?.endsWith(".node"),
  },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/rules/postgres-no-mutable-sql-registry/fixture/effort-low",
    );
    const cli = process.env.NO_MISTAKES_TEST_CLI_PATH || join(dirname(addon), "no-mistakes");
    for (const mode of ["report", "advise", "ignore"]) {
      const options = { root, config: join(root, `.no-mistakes-${mode}.yml`) };
      const pending = cjs.check(options);
      assert.ok(pending instanceof Promise);
      const low = await pending;
      assert.equal(low.rules.length, mode === "report" ? 2 : 0);
      assert.equal(low.advisories.length, mode === "advise" ? 2 : 0);
      assert.deepEqual(await esm.check(options), low);
      const child = spawnSync(
        cli,
        ["check", "--root", root, "--config", options.config, "--format", "json"],
        { encoding: "utf8" },
      );
      assert.equal(child.error, undefined);
      assert.equal(child.status, mode === "report" ? 1 : 0, child.stderr);
      assert.deepEqual(JSON.parse(child.stdout), low);
      const max = await cjs.check({ ...options, effort: "max" });
      const request = {
        ...options,
        reports: [
          { type: "check", id: "low", effort: "low" },
          { type: "check", id: "max", effort: "max" },
        ],
      };
      const batch = await cjs.analyzeProject(request);
      assert.deepEqual(batch.reports.find((report) => report.id === "low").result, low);
      assert.deepEqual(batch.reports.find((report) => report.id === "max").result, max);
      assert.deepEqual(await esm.analyzeProject(request), batch);
    }
  },
);
