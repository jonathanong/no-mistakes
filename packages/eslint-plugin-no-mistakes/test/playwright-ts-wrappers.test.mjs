import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import {
  oxlintFindings,
  pluginFindings,
  snapshotCases,
  snapshotDocument,
} from "./diagnostic-snapshot.mjs";
import { __dirname, fixture } from "./helpers.mjs";

const fixtureDir = "playwright-ts-wrappers";
const fixtureRoot = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", fixtureDir);

const cases = [
  {
    name: "playwright-unique (#4411)",
    rule: "playwright-unique",
    file: "unique.tsx",
    expectedLines: [4, 5, 6, 7],
  },
  {
    name: "playwright-selector-priority (#4409)",
    rule: "playwright-selector-priority",
    file: "selector-priority.spec.ts",
    expectedLines: [5, 6, 7, 8, 9, 10, 11, 12, 13, 14],
  },
  {
    name: "playwright-require-interactive-test-id (#4408)",
    rule: "playwright-require-interactive-test-id",
    file: "interactive-test-id.tsx",
    options: { interactiveComponents: ["Button"] },
    expectedLines: [11, 12, 13, 14, 15, 16, 17, 18],
  },
  {
    name: "playwright-require-exported-component-attribute (#4405)",
    rule: "playwright-require-exported-component-attribute",
    file: "exported-component.tsx",
    expectedLines: [5, 5, 6, 9, 10, 11],
  },
  {
    name: "playwright-prefer-get-by-test-id (#4403)",
    rule: "playwright-prefer-get-by-test-id",
    file: "prefer-get-by-test-id.spec.ts",
    expectedLines: [3, 4, 5, 6, 7, 8, 9, 10],
  },
  {
    name: "playwright-no-empty (#4392)",
    rule: "playwright-no-empty",
    file: "no-empty.tsx",
    expectedLines: [3, 4, 5, 6, 7],
  },
  {
    name: "playwright-naming-convention (#4387)",
    rule: "playwright-naming-convention",
    file: "naming-convention.tsx",
    expectedLines: [3, 4, 5, 6, 7],
  },
  {
    name: "playwright-literals JSX (#4385)",
    rule: "playwright-literals",
    file: "literals.tsx",
    expectedLines: [11],
  },
  {
    name: "playwright-literals calls (#4385)",
    rule: "playwright-literals",
    file: "literals-calls.spec.ts",
    expectedLines: [6, 7, 8, 9, 10, 12, 13],
  },
];

describe("Playwright selector and attribute rules unwrap TypeScript wrappers", () => {
  it("snapshots ESLint and Oxlint parity across all G7a fixtures", async () => {
    const eslintResults = cases.map((item) => {
      const findings = pluginFindings(
        fixture(`${fixtureDir}/${item.file}`),
        item.rule,
        item.options,
        item.file,
      );
      expect(
        findings.map((finding) => finding.line),
        item.name,
      ).toEqual(item.expectedLines);
      return {
        name: item.name,
        file: item.file,
        rule: item.rule,
        ...(item.options ? { options: item.options } : {}),
        findings,
      };
    });

    const oxlintResult = spawnSync(
      process.execPath,
      [
        resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
        "--config",
        resolve(fixtureRoot, ".oxlintrc.json"),
        "--format",
        "json",
        ...cases.map((item) => resolve(fixtureRoot, item.file)),
      ],
      { encoding: "utf8" },
    );
    assert.equal(oxlintResult.status, 1, oxlintResult.stderr);
    assert.equal(oxlintResult.stdout.includes("Error running JS plugin"), false);
    const parsed = JSON.parse(oxlintResult.stdout).diagnostics.filter((entry) =>
      entry.code?.startsWith("no-mistakes("),
    );
    const oxlintResults = cases.map((item) => {
      const findings = oxlintFindings(
        parsed.filter(
          (entry) =>
            entry.filename?.endsWith(item.file) && entry.code === `no-mistakes(${item.rule})`,
        ),
        item.file,
      );
      expect(
        findings.map((finding) => finding.line),
        `oxlint ${item.name}`,
      ).toEqual(item.expectedLines);
      return {
        name: item.name,
        file: item.file,
        rule: item.rule,
        findings,
      };
    });

    await expect(
      snapshotDocument({
        eslint: snapshotCases(eslintResults),
        oxlint: snapshotCases(oxlintResults),
      }),
    ).toMatchFileSnapshot("snapshots/playwright-ts-wrappers.snap");
  });
});
