const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const packageRoot = join(__dirname, "..");
const declarations = (name) => readFileSync(join(packageRoot, name), "utf8");

test("check SQL verdict declarations are named and exposed from both entrypoints", () => {
  assert.match(
    declarations("traversal-types.d.ts"),
    /export interface CheckOptions extends ProjectOptions \{[\s\S]*?includeSqlVerdicts\?: boolean;/,
  );
  assert.match(
    declarations("analyze-project-types.d.ts"),
    /type BatchedCheckOptions = Pick<[\s\S]*?"includeSqlVerdicts"[\s\S]*?>/,
  );
  assert.match(declarations("check-report-types.d.ts"), /sqlVerdicts\?: SqlObligationVerdict\[\];/);
  assert.match(
    declarations("postgres-query-annotation-types.d.ts"),
    /"report" \| "ignore" \| "advise"/,
  );
  assert.match(
    declarations("check-report-types.d.ts"),
    /export type SqlVerdictStatus = "violation" \| "proven-clear" \| "abstain";/,
  );
  assert.match(
    declarations("check-report-types.d.ts"),
    /export type SqlEvidenceKind = "exact" \| "exhaustive-union" \| "verified-prefix" \| "policy-exemption";/,
  );
  assert.match(declarations("check-report-types.d.ts"), /export interface SqlObligationVerdict \{/);
  assert.match(declarations("check-report-types.d.ts"), /export interface SqlExecutorProfile \{/);
  assert.match(declarations("check-report-types.d.ts"), /profile: SqlExecutorProfile;/);
  assert.match(declarations("check-report-types.d.ts"), /export interface SqlRuleApplication \{/);
  assert.match(declarations("check-report-types.d.ts"), /application\?: SqlRuleApplication;/);
  assert.match(declarations("check-report-types.d.ts"), /advisory: boolean;/);
  assert.match(declarations("check-report-types.d.ts"), /knownPrefix\?: string;/);
  assert.match(declarations("check-report-types.d.ts"), /helperLines\?: number\[\];/);
  assert.match(declarations("types.d.ts"), /export \* from "\.\/traversal-types";/);
});
