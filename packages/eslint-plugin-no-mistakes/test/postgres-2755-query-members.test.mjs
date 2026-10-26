import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { __dirname, fixture, lint } from "./helpers.mjs";

const options = { importSpecifier: "pg" };
function findings(name, rule, option = options, filename = "app.ts") {
  return lint(
    fixture(`postgres-2755/${name}`),
    {
      [`no-mistakes/${rule}`]: ["error", option],
    },
    filename,
  ).map(({ line, messageId, ruleId }) => [line, messageId, ruleId]);
}

describe("PostgreSQL query member receiver proof", () => {
  it("keeps proven namespace and named imports while ignoring this.query", () => {
    assert.deepEqual(findings("query-members.ts", "postgres-no-manual-transaction"), [
      [6, "manualTransaction", "no-mistakes/postgres-no-manual-transaction"],
      [7, "manualTransaction", "no-mistakes/postgres-no-manual-transaction"],
      [9, "manualTransaction", "no-mistakes/postgres-no-manual-transaction"],
    ]);
    assert.deepEqual(findings("query-members.ts", "postgres-no-unbounded-query-fanout"), [
      [11, "unboundedFanout", "no-mistakes/postgres-no-unbounded-query-fanout"],
      [12, "unboundedFanout", "no-mistakes/postgres-no-unbounded-query-fanout"],
    ]);
    assert.deepEqual(findings("query-members.ts", "postgres-no-nonliteral-sql-registry-key"), [
      [14, "dynamicKey", "no-mistakes/postgres-no-nonliteral-sql-registry-key"],
    ]);
    const scoped = {
      importSpecifier: "pg",
      executorFactoryNames: ["openTransaction"],
      executorTypeNames: ["TxExecutor"],
    };
    assert.deepEqual(
      findings("scoped-query-members.ts", "postgres-no-manual-transaction", scoped),
      [
        [7, "manualTransaction", "no-mistakes/postgres-no-manual-transaction"],
        [12, "manualTransaction", "no-mistakes/postgres-no-manual-transaction"],
      ],
    );
    assert.deepEqual(
      findings("scoped-query-members.ts", "postgres-no-unbounded-query-fanout", scoped),
      [
        [8, "unboundedFanout", "no-mistakes/postgres-no-unbounded-query-fanout"],
        [13, "unboundedFanout", "no-mistakes/postgres-no-unbounded-query-fanout"],
      ],
    );
    assert.deepEqual(
      findings("scoped-query-members.ts", "postgres-no-nonliteral-sql-registry-key", scoped),
      [
        [9, "dynamicKey", "no-mistakes/postgres-no-nonliteral-sql-registry-key"],
        [14, "dynamicKey", "no-mistakes/postgres-no-nonliteral-sql-registry-key"],
      ],
    );
  });

  it("matches typed and untyped fixtures in actual type-aware Oxlint", () => {
    const root = resolve(__dirname, "../../..");
    const fixtureDir = resolve(root, "test-cases/eslint-snippets/fixture/postgres-2755");
    const result = spawnSync(
      process.execPath,
      [
        resolve(root, "node_modules/oxlint/bin/oxlint"),
        "--type-aware",
        "--deny-warnings",
        "--format",
        "json",
        resolve(fixtureDir, "query-members.js"),
        resolve(fixtureDir, "scoped-query-members.ts"),
      ],
      { cwd: fixtureDir, encoding: "utf8" },
    );
    assert.equal(result.status, 1, result.stderr);
    const actual = JSON.parse(result.stdout)
      .diagnostics.map((item) => [
        item.filename.replaceAll("\\", "/").split("/").at(-1),
        item.labels[0].span.line,
        item.code,
      ])
      .sort((left, right) => left[0].localeCompare(right[0]) || left[1] - right[1]);
    assert.deepEqual(actual, [
      ["query-members.js", 3, "no-mistakes(postgres-no-manual-transaction)"],
      ["query-members.js", 4, "no-mistakes(postgres-no-manual-transaction)"],
      ["query-members.js", 7, "no-mistakes(postgres-no-unbounded-query-fanout)"],
      ["scoped-query-members.ts", 7, "no-mistakes(postgres-no-manual-transaction)"],
      ["scoped-query-members.ts", 8, "no-mistakes(postgres-no-unbounded-query-fanout)"],
      ["scoped-query-members.ts", 9, "no-mistakes(postgres-no-nonliteral-sql-registry-key)"],
      ["scoped-query-members.ts", 12, "no-mistakes(postgres-no-manual-transaction)"],
      ["scoped-query-members.ts", 13, "no-mistakes(postgres-no-unbounded-query-fanout)"],
      ["scoped-query-members.ts", 14, "no-mistakes(postgres-no-nonliteral-sql-registry-key)"],
    ]);
  });
});
