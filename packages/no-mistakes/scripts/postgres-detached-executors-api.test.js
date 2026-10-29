const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
const root = join(
  __dirname,
  "../../../test-cases/rules/postgres-no-test-state-mutation/detached-executors",
);

function stable(report) {
  const { jobsPoolWarning: _poolWarning, ...result } = report;
  return result;
}

function snapshot(report) {
  const rows = (items) =>
    items.map((row) => ({
      ...row,
      sourceLine: readFileSync(join(root, row.file), "utf8").split(/\r?\n/u)[row.line - 1].trim(),
    }));
  return { rules: rows(report.rules), advisories: rows(report.advisories) };
}

for (const policy of ["max", "low", "report", "ignore"])
  test(
    `compiled detached query ${policy} preserves asynchronous CJS, ESM, and prepared checks`,
    { skip: !compiled },
    async () => {
      const cjs = require("../index.js");
      const esm = await import("../index.mjs");
      const snapshots = {};
      let expected;
      for (const jobs of [1, 2]) {
        const options = { root, config: join(root, `${policy}.yml`), jobs };
        const pending = cjs.check(options);
        assert.equal(typeof pending.then, "function");
        const report = await pending;
        assert.deepEqual(stable(await esm.check(options)), stable(report));
        for (const api of [cjs, esm]) {
          const prepared = await api.analyzeProject({ ...options, reports: [{ type: "check" }] });
          assert.deepEqual(stable(prepared.reports[0].result), stable(report));
        }
        const current = snapshot(report);
        assert.ok(
          current.rules.some(({ file, line }) => file === "tests/exact.test.ts" && line === 4),
        );
        assert.equal(current.advisories.length === 0, policy === "report" || policy === "ignore");
        if (expected) assert.deepEqual(current, expected, `${policy} differs across jobs`);
        else expected = current;
      }
      snapshots[policy] = expected;
      await globalThis
        .expect(`${JSON.stringify(snapshots, null, 2)}\n`)
        .toMatchFileSnapshot(`snapshots/postgres-detached-executors-${policy}-api.snap`);
    },
  );
