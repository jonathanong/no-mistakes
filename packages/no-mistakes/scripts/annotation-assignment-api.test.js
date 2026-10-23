const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { dirname, join } = require("node:path");
const { spawnSync } = require("node:child_process");
const test = globalThis.test || require("node:test").test;
const addon = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;

test(
  "compiled low annotation decides mutable literal heads before unknown policy",
  { skip: !addon?.endsWith(".node") },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/rules/postgres-require-query-annotation/fixture/low-assignments",
    );
    const source = readFileSync(join(root, "src/query.mts"), "utf8");
    const marked = (marker) =>
      source.split("\n").flatMap((line, index) => (line.includes(marker) ? [index + 1] : []));
    const missing = marked("EXPECT_MISSING_");
    const unknown = marked("EXPECT_UNKNOWN_");
    const lines = (rows) => rows.map((row) => row.line).sort((left, right) => left - right);
    const cli =
      process.env.NO_MISTAKES_TEST_CLI_PATH ||
      join(dirname(addon), process.platform === "win32" ? "no-mistakes.exe" : "no-mistakes");
    for (const policy of ["report", "advise", "ignore"]) {
      const config = join(root, `.no-mistakes-low-${policy}.yml`);
      const pending = cjs.check({ root, config });
      assert.ok(pending instanceof Promise);
      const result = await pending;
      assert.deepEqual(result.warnings, []);
      assert.deepEqual(
        lines(result.rules),
        (policy === "report" ? [...missing, ...unknown] : missing).sort((a, b) => a - b),
      );
      assert.deepEqual(lines(result.advisories), policy === "advise" ? unknown : []);
      for (const finding of result.rules.filter((row) => missing.includes(row.line))) {
        assert.match(finding.message, /query execution must start with a \/\* name \*\//);
      }
      assert.deepEqual(await esm.check({ root, config }), result);
      const detailed = await cjs.check({ root, config, includeSqlVerdicts: true });
      const { sqlVerdicts, ...ordinary } = detailed;
      assert.deepEqual(ordinary, result);
      for (const line of missing) {
        const verdict = sqlVerdicts.find(
          (row) => source.slice(0, row.span[0]).split("\n").length === line,
        );
        assert.equal(verdict?.status, "violation", `line ${line}`);
        assert.equal(verdict.advisory, false);
      }
      const child = spawnSync(
        cli,
        ["check", "--root", root, "--config", config, "--format", "json", "--jobs", "4"],
        { encoding: "utf8" },
      );
      assert.equal(child.error, undefined);
      assert.equal(child.status, 1, child.stderr);
      assert.deepEqual(JSON.parse(child.stdout), result);
    }
    const maximum = await cjs.check({ root, config: join(root, ".no-mistakes-max-report.yml") });
    // Max locates finite alternatives at their SQL literals; low uses the call.
    const maxMissing = marked("EXPECT_MAX_MISSING");
    assert.deepEqual(
      lines(maximum.rules.filter((row) => maxMissing.includes(row.line))),
      maxMissing,
    );
    const allNamed = marked("EXPECT_ACCEPTED_ALL_NAMED")[0];
    assert.ok(!maximum.rules.some((row) => row.line === allNamed));
    const opaqueCall = marked("EXPECT_UNKNOWN_CALL")[0];
    assert.ok(
      maximum.rules.some((row) => row.line === opaqueCall && /unanalyzable/.test(row.message)),
    );
  },
);
