import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { describe, expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { __dirname, fixture } from "./helpers.mjs";

const options = { importSpecifier: "pg" };
const scoped = {
  importSpecifier: "pg",
  executorFactoryNames: ["openTransaction"],
  executorTypeNames: ["TxExecutor"],
};
const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};

function memberCase(name, option) {
  return {
    code: fixture(`postgres-2755/${name}`),
    filename: "app.ts",
    options: [option],
    languageOptions: { parser: tsParser },
  };
}

// reports unproven manual transactions while other rules retain receiver proof.
const manual = {
  invalid: [
    // Four manualTransaction findings.
    memberCase("query-members.ts", options),
    // Two manualTransaction findings.
    memberCase("scoped-query-members.ts", scoped),
  ],
};
const fanout = {
  invalid: [
    // Two unboundedFanout findings.
    memberCase("query-members.ts", options),
    // Two unboundedFanout findings.
    memberCase("scoped-query-members.ts", scoped),
  ],
};
const registry = {
  invalid: [
    // One dynamicKey finding.
    memberCase("query-members.ts", options),
    // Two dynamicKey findings.
    memberCase("scoped-query-members.ts", scoped),
  ],
};

it("snapshots postgres query member diagnostics", async () => {
  await expect(
    snapshotDocument({
      "postgres-no-manual-transaction": snapshotSuite(
        "postgres-no-manual-transaction",
        languageOptions,
        manual,
      ),
      "postgres-no-unbounded-query-fanout": snapshotSuite(
        "postgres-no-unbounded-query-fanout",
        languageOptions,
        fanout,
      ),
      "postgres-no-nonliteral-sql-registry-key": snapshotSuite(
        "postgres-no-nonliteral-sql-registry-key",
        languageOptions,
        registry,
      ),
    }),
  ).toMatchFileSnapshot("snapshots/postgres-2755-query-members.snap");
}, 30_000);

describe("PostgreSQL query member receiver proof", () => {
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
      ["query-members.js", 11, "no-mistakes(postgres-no-manual-transaction)"],
      ["scoped-query-members.ts", 7, "no-mistakes(postgres-no-manual-transaction)"],
      ["scoped-query-members.ts", 8, "no-mistakes(postgres-no-unbounded-query-fanout)"],
      ["scoped-query-members.ts", 9, "no-mistakes(postgres-no-nonliteral-sql-registry-key)"],
      ["scoped-query-members.ts", 12, "no-mistakes(postgres-no-manual-transaction)"],
      ["scoped-query-members.ts", 13, "no-mistakes(postgres-no-unbounded-query-fanout)"],
      ["scoped-query-members.ts", 14, "no-mistakes(postgres-no-nonliteral-sql-registry-key)"],
    ]);
  });
});
