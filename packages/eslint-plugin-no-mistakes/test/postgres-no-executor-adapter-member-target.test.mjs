import { dirname, resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { expect, it } from "vitest";
import {
  languageOptionsFindings,
  snapshotDocument,
  snapshotSuite,
} from "./diagnostic-snapshot.mjs";
import { __dirname, fixture } from "./helpers.mjs";

const rule = "postgres-no-executor-adapter";
const languageOptions = { ecmaVersion: 2024, sourceType: "module", globals: {} };
const options = { importSpecifier: "@example/db", executorTypeNames: ["QueryExecutor"] };
const unsafeLines = [5, 9, 13, 18, 23, 29, 34, 38, 42, 49, 53, 57, 61, 66, 74, 78, 82];

function testCase(name, mode, typed) {
  const filename = resolve(
    __dirname,
    "../../../test-cases/eslint-snippets/fixture/executor-target",
    name,
  );
  return {
    code: fixture(`executor-target/${name}`),
    filename,
    options: [{ ...options, mode }],
    languageOptions: {
      parser: tsParser,
      parserOptions: typed ? { projectService: true, tsconfigRootDir: dirname(filename) } : {},
    },
  };
}

const cases = [];
for (const typed of [false, true]) {
  for (const mode of ["forwarding-only", "all"]) {
    cases.push(["member-local.ts", mode, typed, mode === "all" ? [10, 18] : [12, 18]]);
    cases.push(["member-unsafe.ts", mode, typed, unsafeLines]);
    cases.push(["member-exported.ts", mode, typed, [4, 7, 10]]);
    cases.push(["member-eval.ts", mode, typed, [6, 11]]);
    cases.push(["member-discovery.ts", mode, typed, []]);
    cases.push([
      "member-imported.ts",
      mode,
      typed,
      typed ? (mode === "all" ? [8, 10] : [7, 8, 10]) : [5, 6, 7, 8, 10, 11],
    ]);
  }
}

it("pins stable property helper targets without widening executor discovery", async () => {
  const rows = [];
  for (const [name, mode, typed, expected] of cases) {
    const sample = testCase(name, mode, typed);
    const findings = languageOptionsFindings({ ...sample, rule });
    expect(
      findings.map(({ line }) => line),
      `${name} ${mode} typed=${typed}`,
    ).toEqual(expected);
    const kind = expected.length ? "invalid" : "valid";
    rows.push(
      ...snapshotSuite(rule, languageOptions, { [kind]: [sample] }).map((row) => ({
        ...row,
        typed,
      })),
    );
  }
  await expect(snapshotDocument(rows)).toMatchFileSnapshot(
    "snapshots/postgres-no-executor-adapter-member-target.snap",
  );
}, 30_000);
