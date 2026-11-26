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

const fixtureDir = "playwright-import-shapes";
const fixtureRoot = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", fixtureDir);

const cases = [
  {
    name: "assertion CommonJS and import-equals (#4356)",
    rule: "playwright-assertion-timeout-cap",
    file: "e2e/assertion-imports.spec.ts",
    expected: [
      [4, "timeout"],
      [7, "timeout"],
      [10, "timeout"],
      [13, "timeout"],
    ],
  },
  {
    name: "assertion TypeScript wrappers (#4371)",
    rule: "playwright-assertion-timeout-cap",
    file: "e2e/assertion-wrappers.spec.ts",
    expected: [
      [5, "timeout"],
      [6, "timeout"],
      [7, "timeout"],
      [8, "timeout"],
      [9, "timeout"],
      [10, "timeout"],
    ],
  },
  {
    name: "assertion unresolved expect (#4578)",
    rule: "playwright-assertion-timeout-cap",
    file: "e2e/assertion-opaque.spec.ts",
    expected: [
      [5, "unanalyzable"],
      [8, "unanalyzable"],
    ],
  },
  {
    name: "assertion traced-away and reassigned expect",
    rule: "playwright-assertion-timeout-cap",
    file: "e2e/assertion-quiet.spec.ts",
    expected: [],
  },
  {
    name: "assertion unanalyzable ignore",
    rule: "playwright-assertion-timeout-cap",
    file: "e2e/assertion-ignore.spec.ts",
    options: { unanalyzable: "ignore" },
    expected: [],
  },
  {
    name: "test timeout CommonJS and import-equals (#4357)",
    rule: "playwright-test-timeout-cap",
    file: "e2e/test-timeout-require.spec.ts",
    expected: [
      [2, "timeout"],
      [5, "timeout"],
      [8, "timeout"],
    ],
  },
  {
    name: "test timeout wrappers and satisfies sibling (#4377)",
    rule: "playwright-test-timeout-cap",
    file: "e2e/test-timeout-wrappers.spec.ts",
    expected: [
      [5, "timeout"],
      [9, "timeout"],
      [14, "timeout"],
    ],
  },
  {
    name: "defineConfig satisfies stays under the cap (#4377)",
    rule: "playwright-test-timeout-cap",
    file: "playwright.config.ts",
    options: { configFiles: ["playwright.config.ts"], unknownValues: "finding" },
    expected: [],
  },
  {
    name: "dynamic require stays quiet when unknownValues is ignore",
    rule: "playwright-test-timeout-cap",
    file: "e2e/test-timeout-dynamic-ignore.spec.ts",
    expected: [],
  },
  {
    name: "dynamic require is unknown when unknownValues is finding",
    rule: "playwright-test-timeout-cap",
    file: "e2e/test-timeout-dynamic-finding.spec.ts",
    options: { unknownValues: "finding" },
    expected: [[4, "unknown"]],
  },
  {
    name: "wrapped test.slow(false) is not an override",
    rule: "playwright-test-timeout-cap",
    file: "e2e/test-timeout-ban.spec.ts",
    options: { banOverrides: true },
    expected: [[5, "override"]],
  },
  {
    name: "hoisted CommonJS and import-equals (#4362)",
    rule: "playwright-no-hoisted-unique-token",
    file: "e2e/hoisted-imports.spec.ts",
    options: { tokenFactories: ["makeToken"] },
    expected: [
      [4, "hoisted"],
      [10, "hoisted"],
      [16, "hoisted"],
    ],
  },
  {
    name: "hoisted require outside e2e (#4362)",
    rule: "playwright-no-hoisted-unique-token",
    file: "src/hoisted-require.ts",
    options: { tokenFactories: ["makeToken"] },
    expected: [[4, "hoisted"]],
  },
  {
    name: "hoisted wrappers and shielded test callback (#4398)",
    rule: "playwright-no-hoisted-unique-token",
    file: "e2e/hoisted-wrappers.spec.ts",
    options: { tokenFactories: ["randomSuffix"] },
    expected: [
      [5, "hoisted"],
      [9, "hoisted"],
      [13, "hoisted"],
      [17, "hoisted"],
    ],
  },
  {
    name: "opaque beforeAll is unanalyzable",
    rule: "playwright-no-hoisted-unique-token",
    file: "e2e/hoisted-opaque.spec.ts",
    options: { tokenFactories: ["makeToken"] },
    expected: [[5, "unanalyzable"]],
  },
  {
    name: "raw scroll require outside e2e (#4364)",
    rule: "playwright-no-raw-scroll-pagination",
    file: "src/scroll-require.ts",
    expected: [[5, "rawScroll"]],
  },
  {
    name: "raw scroll import-equals outside e2e (#4364)",
    rule: "playwright-no-raw-scroll-pagination",
    file: "src/scroll-equals.ts",
    expected: [[5, "rawScroll"]],
  },
  {
    name: "scroll and sentinel wrappers (#4401)",
    rule: "playwright-no-raw-scroll-pagination",
    file: "e2e/scroll-wrappers.spec.ts",
    options: { sentinelTestIds: ["feed-sentinel"] },
    expected: [
      [9, "rawScroll"],
      [10, "rawScroll"],
      [11, "sentinelScroll"],
      [12, "sentinelScroll"],
    ],
  },
  {
    name: "selector require outside e2e (#4366)",
    rule: "playwright-selector-priority",
    file: "src/selector-require.ts",
    expected: [[4, "semantic"]],
  },
  {
    name: "selector import-equals outside e2e (#4366)",
    rule: "playwright-selector-priority",
    file: "src/selector-equals.ts",
    expected: [[4, "semantic"]],
  },
  {
    name: "defaulted test ids through wrappers (#4368)",
    rule: "playwright-defaults",
    file: "src/defaults.tsx",
    expected: [
      [2, "default"],
      [5, "default"],
    ],
  },
  {
    name: "interactive CommonJS and string imports (#4369)",
    rule: "playwright-require-interactive-test-id",
    file: "src/interactive-imports.tsx",
    options: { interactiveComponents: ["Button"] },
    expected: [
      [2, "missing"],
      [6, "missing"],
      [10, "missing"],
      [13, "missing"],
    ],
  },
  {
    name: "opaque interactive component",
    rule: "playwright-require-interactive-test-id",
    file: "src/interactive-opaque.tsx",
    options: { interactiveComponents: ["Button"] },
    expected: [[4, "unanalyzable"]],
  },
  {
    name: "opaque interactive component ignored",
    rule: "playwright-require-interactive-test-id",
    file: "src/interactive-ignore.tsx",
    options: { interactiveComponents: ["Button"], unanalyzable: "ignore" },
    expected: [],
  },
  {
    name: "swallowed visibility computed keys (#4489)",
    rule: "playwright-no-swallowed-visibility",
    file: "swallowed.ts",
    expected: [
      [3, "swallowed"],
      [4, "swallowed"],
      [5, "swallowed"],
    ],
  },
];

function pairs(findings) {
  return findings.map((finding) => [finding.line, finding.messageId]);
}

describe("Playwright rules follow TypeScript wrappers and CommonJS imports", () => {
  it("snapshots ESLint and Oxlint parity", async () => {
    const eslintResults = cases.map((item) => {
      const findings = pluginFindings(
        fixture(`${fixtureDir}/${item.file}`),
        item.rule,
        item.options,
        item.file,
      );
      expect(pairs(findings), item.name).toEqual(item.expected);
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
      ).toEqual(item.expected.map(([line]) => line));
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
    ).toMatchFileSnapshot("snapshots/playwright-import-shapes.snap");
  });
});
