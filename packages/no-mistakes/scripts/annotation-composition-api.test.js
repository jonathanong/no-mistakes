const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { dirname, join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const addon = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;
const compiled = addon?.endsWith(".node");

test(
  "compiled composed annotation SQL preserves async API and CLI parity",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/rules/postgres-require-query-annotation/fixture/append-composition",
    );
    const cli =
      process.env.NO_MISTAKES_TEST_CLI_PATH ||
      join(dirname(addon), process.platform === "win32" ? "no-mistakes.exe" : "no-mistakes");
    const withoutLedger = ({ sqlVerdicts: _sqlVerdicts, ...report }) => report;

    for (const policy of ["report", "advise", "ignore"]) {
      const config = join(root, `${policy}.yml`);
      let baseline;
      for (const includeSqlVerdicts of [false, true]) {
        const options = { root, config, includeSqlVerdicts };
        const pending = cjs.check(options);
        assert.ok(pending instanceof Promise);
        const report = await pending;
        assert.deepEqual(await esm.check(options), report, `${policy}: ESM parity`);
        assert.deepEqual(
          report.rules.map(({ line }) => line),
          [3],
          `${policy}: valid sibling`,
        );
        assert.deepEqual(
          report.advisories.map(({ line }) => line),
          [5, 8, 11],
          `${policy}: invalid alternatives advise without failing`,
        );
        assert.ok(
          report.advisories.every(
            ({ target, message }) =>
              target === "sql-composition" && message.includes("invalid composed SQL variant"),
          ),
          `${policy}: ${JSON.stringify(report.advisories)}`,
        );

        if (includeSqlVerdicts) {
          assert.ok(Array.isArray(report.sqlVerdicts));
          for (const prefix of ["/* bound */SELECT id", "/* direct */SELECT id"]) {
            const row = report.sqlVerdicts.find(({ knownPrefix }) =>
              knownPrefix?.startsWith(prefix),
            );
            assert.ok(row, `${policy}: missing composed evidence for ${prefix}`);
            assert.match(row.knownPrefix, /owner_id = sql_placeholder_1$/);
            assert.doesNotMatch(row.knownPrefix, /ownerId/);
          }
          assert.deepEqual(
            withoutLedger(report),
            baseline,
            `${policy}: ledger changed public output`,
          );
        } else {
          assert.equal(report.sqlVerdicts, undefined);
          baseline = withoutLedger(report);
        }

        const args = ["check", "--root", root, "--config", config, "--format", "json"];
        if (includeSqlVerdicts) args.push("--include-sql-verdicts");
        const child = spawnSync(cli, args, { encoding: "utf8", env: process.env });
        assert.equal(child.error, undefined);
        assert.equal(child.status, 1, child.stderr);
        assert.deepEqual(JSON.parse(child.stdout), report, `${policy}: CLI parity`);
      }
    }
  },
);
