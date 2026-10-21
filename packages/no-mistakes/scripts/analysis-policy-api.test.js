const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { dirname, join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const addon = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;
const compiled = addon?.endsWith(".node");

test(
  "compiled check effort and shared policies preserve CLI and async API parity",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/codebase-analysis/rule-analysis-policy/fixture",
    );
    const cli =
      process.env.NO_MISTAKES_TEST_CLI_PATH ||
      join(dirname(addon), process.platform === "win32" ? "no-mistakes.exe" : "no-mistakes");
    for (const [policy, failures, advisories] of [
      ["report", 1, 0],
      ["advise", 0, 1],
      ["ignore", 0, 0],
    ]) {
      const options = { root, config: join(root, `sql-${policy}.yml`) };
      const pending = cjs.check(options);
      assert.ok(pending instanceof Promise);
      const expected = await pending;
      assert.equal(expected.rules.length, failures);
      assert.equal(expected.advisories.length, advisories);
      const encoded = JSON.stringify(expected);
      const legacy = await cjs.check({
        ...options,
        config: join(root, `legacy-sql-${policy}.yml`),
      });
      assert.equal(JSON.stringify(legacy), encoded);
      let defaultStdout;
      for (const effort of [undefined, "max", "low"]) {
        const requested = { ...options, ...(effort ? { effort } : {}) };
        for (const api of [cjs, esm])
          assert.equal(JSON.stringify(await api.check(requested)), encoded);
        const args = ["check", "--root", root, "--config", options.config, "--format", "json"];
        if (effort) args.push("--effort", effort);
        const child = spawnSync(cli, args, { encoding: "utf8", env: process.env });
        assert.equal(child.error, undefined);
        assert.equal(child.status, failures ? 1 : 0, child.stderr);
        assert.deepEqual(JSON.parse(child.stdout), expected);
        if (effort === undefined) defaultStdout = child.stdout;
        else assert.equal(child.stdout, defaultStdout);
      }
      // Each report has a distinct override: sharing a prepared result must not
      // make the first report's effort become the next report's option.
      const request = {
        ...options,
        reports: ["low", "max", undefined, "low"].map((effort, index) => ({
          type: "check",
          id: String(index),
          ...(effort ? { effort } : {}),
        })),
      };
      const batchPending = cjs.analyzeProject(request);
      assert.ok(batchPending instanceof Promise);
      const batch = await batchPending;
      for (const report of batch.reports) assert.equal(JSON.stringify(report.result), encoded);
      assert.deepEqual(await esm.analyzeProject(request), batch);
    }
    const inherited = await cjs.check({ root, config: "project-no-policy.yml" });
    assert.deepEqual(
      inherited.rules.map(({ rule, line }) => [rule, line]),
      [
        ["postgres-no-mutable-sql-registry", 4],
        ["postgres-no-unbounded-sql-identifier", 5],
      ],
    );
    for (const config of [
      "project-fail.yml",
      "project-previously-ignored.yml",
      "opaque-legacy.yml",
    ])
      for (const api of [cjs, esm]) assert.deepEqual(await api.check({ root, config }), inherited);
    await assert.rejects(cjs.check({ root, effort: "high" }), /effort|variant|high/i);
  },
);
