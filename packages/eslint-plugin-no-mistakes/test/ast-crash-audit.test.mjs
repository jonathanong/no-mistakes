import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { expect, it } from "vitest";
import { oxlintFindings, pluginFindings, snapshotDocument } from "./diagnostic-snapshot.mjs";
import { __dirname, fixture } from "./helpers.mjs";

const relativeRoot = "ast-crash-audit";
const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", relativeRoot);
const executable = resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint");
const findings = { unknownValues: "finding" };
const cases = [
  ["playwright-configure-no-args.spec.ts", "playwright-test-timeout-cap", findings, ["unknown"]],
  ["playwright-extend-no-args.spec.ts", "playwright-test-timeout-cap", findings, ["unknown"]],
  ["playwright-import-equals.spec.ts", "playwright-test-timeout-cap", findings, ["timeout"]],
  ["playwright-slow-for-of.spec.ts", "playwright-test-timeout-cap", findings, ["unknown"]],
  ["vitest-import-equals.test.ts", "vitest-timeout-cap", findings, ["timeout"]],
  ["vitest-scalar-copy-ambient.test.ts", "vitest-timeout-cap", findings, ["unknown", "unknown"]],
  [
    "async-import-equals-mock.ts",
    "async-call-disposition",
    { targets: [{ sourceSpecifierPatterns: ["@fixture/jobs"], calleeNamePatterns: ["enqueue"] }] },
    ["disposition"],
  ],
  [
    "correlated-global.js",
    "postgres-no-correlated-sql-builder",
    { builderMethods: ["append"], tagNames: ["sql"] },
    ["correlated"],
  ],
  [
    "config-mirror-import-equals.test.ts",
    "test-no-config-mirror-assertion",
    undefined,
    ["configMirror"],
  ],
];

it("reports rule diagnostics on import equals, absent values, and configured globals", async () => {
  const casesWithFindings = cases.map(([file, rule, options, expected]) => {
    const eslint = pluginFindings(fixture(`${relativeRoot}/${file}`), rule, options, file);
    expect(
      eslint.map((row) => row.messageId),
      file,
    ).toEqual(expected);
    return { file, rule, eslint };
  });
  const result = spawnSync(
    process.execPath,
    [
      executable,
      "--config",
      resolve(root, ".oxlintrc.json"),
      "--format",
      "json",
      ...cases.map(([file]) => resolve(root, file)),
    ],
    { cwd: root, encoding: "utf8" },
  );
  expect(result.error).toBeUndefined();
  expect(result.status, result.stderr).toBe(1);
  const oxlint = oxlintFindings(JSON.parse(result.stdout).diagnostics);
  for (const [file, rule, , expected] of cases) {
    const rows = oxlint.filter((row) => row.file.endsWith(file));
    expect(
      rows.map((row) => row.code?.includes(rule)),
      file,
    ).toEqual(expected.map(() => true));
  }
  await expect(snapshotDocument({ cases: casesWithFindings, oxlint })).toMatchFileSnapshot(
    "snapshots/ast-crash-audit.snap",
  );
}, 30_000);
