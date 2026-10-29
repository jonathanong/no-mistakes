const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
const root = join(
  __dirname,
  "../../../test-cases/rules/postgres-no-test-state-mutation/detached-obligations",
);
function stable({ jobsPoolWarning: _warning, ...report }) {
  return report;
}
function snapshot(report) {
  const rows = (items) =>
    items.map((row) => ({
      ...row,
      sourceLine: readFileSync(join(root, row.file), "utf8").split(/\r?\n/u)[row.line - 1].trim(),
    }));
  return { rules: rows(report.rules), advisories: rows(report.advisories) };
}
for (const policy of ["low", "max", "low-report", "max-report", "low-ignore", "max-ignore"])
  test(
    `compiled executor reference obligations ${policy} preserve async API parity`,
    { skip: !compiled },
    async () => {
      const cjs = require("../index.js");
      const esm = await import("../index.mjs");
      let expected;
      for (const jobs of [1, 2]) {
        const options = { root, config: join(root, `${policy}.yml`), jobs };
        const pending = cjs.check(options);
        assert.equal(typeof pending.then, "function");
        const report = await pending;
        assert.deepEqual(report.warnings, []);
        assert.deepEqual(stable(await esm.check(options)), stable(report));
        for (const api of [cjs, esm]) {
          const prepared = await api.analyzeProject({ ...options, reports: [{ type: "check" }] });
          assert.deepEqual(stable(prepared.reports[0].result), stable(report));
        }
        const current = snapshot(report);
        const has = (field, file, line) =>
          current[field].some((row) => row.file === file && row.line === line);
        for (const [file, lines] of [
          ["tests/references.test.ts", [3, 5, 6, 7, 8, 9, 10, 11, 12, 14, 15, 19, 20, 21, 22, 24]],
          ["tests/names.test.ts", [3, 5, 6, 7, 9, 10]],
          ["tests/wrapped-reference.test.ts", [2]],
          ["tests/forward-export.test.ts", [2]],
          ["tests/parameter-alias.test.ts", [2, 3, 6, 7]],
        ])
          for (const line of lines) {
            assert.equal(has("advisories", file, line), !policy.includes("-"), `${file}:${line}`);
            assert.equal(has("rules", file, line), policy.endsWith("-report"), `${file}:${line}`);
          }
        for (const line of [3, 4, 5, 6]) assert.ok(has("rules", "tests/known.test.ts", line));
        for (const line of [2, 4]) {
          const uncertain = policy.startsWith("low");
          assert.equal(
            has("rules", "tests/direct-wrappers.test.ts", line),
            !uncertain || policy.endsWith("-report"),
          );
          assert.equal(
            has("advisories", "tests/direct-wrappers.test.ts", line),
            uncertain && !policy.includes("-"),
          );
        }
        for (const field of ["rules", "advisories"])
          assert.ok(
            !current[field].some(
              (row) =>
                row.rule === "postgres-no-test-state-mutation" &&
                [
                  "tests/controls.test.ts",
                  "tests/type-uses.test.ts",
                  "tests/suppressed.test.ts",
                  "tests/file-suppressed.test.ts",
                ].includes(row.file),
            ),
          );
        if (expected) assert.deepEqual(current, expected);
        else expected = current;
      }
      await globalThis
        .expect(`${JSON.stringify(expected, null, 2)}\n`)
        .toMatchFileSnapshot(`snapshots/postgres-executor-obligations-${policy}-api.snap`);
    },
  );
