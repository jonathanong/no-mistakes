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
        assert.deepEqual(report.warnings, []);
        for (const field of ["rules", "advisories"]) {
          assert.ok(
            !current[field].some(
              ({ file, line }) =>
                [
                  "tests/format-bind.test.ts",
                  "tests/controls.test.ts",
                  "tests/file-suppressed.test.ts",
                ].includes(file) ||
                (file === "tests/suppressed.test.ts" && [4, 5].includes(line)),
            ),
          );
        }
        if (policy === "low") {
          for (const line of [4, 7, 10, 12, 14, 17, 19]) {
            assert.ok(
              current.rules.some((row) => row.file === "tests/exact.test.ts" && row.line === line),
            );
          }
          for (const [file, line] of [
            ["tests/wrong-this.test.ts", 3],
            ["tests/prebound.test.ts", 5],
            ["tests/after-copy.test.ts", 7],
            ["tests/receiver-escaped-after.test.ts", 6],
            ["tests/local-bodies.test.ts", 11],
            ["tests/assigned.test.ts", 4],
            ...[5, 6, 9, 10].map((line) => ["tests/mixed-roles.test.ts", line]),
          ]) {
            assert.ok(current.advisories.some((row) => row.file === file && row.line === line));
            assert.ok(!current.rules.some((row) => row.file === file && row.line === line));
          }
          assert.ok(
            current.rules.some((row) => row.file === "tests/suppressed.test.ts" && row.line === 6),
          );
        }
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
