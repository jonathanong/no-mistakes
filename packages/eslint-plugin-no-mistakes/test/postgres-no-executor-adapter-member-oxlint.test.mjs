import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { expect, it } from "vitest";
import { oxlintFindings } from "./diagnostic-snapshot.mjs";
import { __dirname } from "./helpers.mjs";

const repoRoot = resolve(__dirname, "../../..");
const fixtureRoot = "test-cases/eslint-snippets/fixture/executor-target";
const commonLines = {
  "member-unsafe.ts": [5, 9, 13, 18, 23, 29, 34, 38, 42, 49, 53, 57, 61, 66, 74, 78, 82],
  "member-exported.ts": [4, 7, 10],
  "member-eval.ts": [6, 11],
  "member-discovery.ts": [],
  "member-imported.ts": [5, 6, 7, 8, 10, 11],
};

it.each([
  ["forwarding-only", [12, 18]],
  ["all", [10, 18]],
])(
  "matches untyped ESLint diagnostics through the Oxlint plugin in %s mode",
  (mode, localLines) => {
    const expected = { "member-local.ts": localLines, ...commonLines };
    const result = spawnSync(
      process.execPath,
      [
        resolve(repoRoot, "node_modules/oxlint/bin/oxlint"),
        "--config",
        `${fixtureRoot}/member-${mode}.oxlint.json`,
        "--format",
        "json",
        ...Object.keys(expected).map((name) => `${fixtureRoot}/${name}`),
      ],
      { cwd: repoRoot, encoding: "utf8", timeout: 4_000 },
    );

    expect(result.error, result.stderr).toBeUndefined();
    expect(result.status, result.stderr).toBe(1);
    const findings = oxlintFindings(JSON.parse(result.stdout).diagnostics);
    expect(new Set(findings.map(({ code }) => code))).toEqual(
      new Set(["no-mistakes(postgres-no-executor-adapter)"]),
    );
    for (const [name, lines] of Object.entries(expected)) {
      const file = `${fixtureRoot}/${name}`;
      expect(
        findings.filter((finding) => finding.file === file).map(({ line }) => line),
        name,
      ).toEqual(lines);
    }
    expect(findings).toHaveLength(Object.values(expected).flat().length);
  },
);
