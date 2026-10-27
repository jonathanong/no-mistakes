import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { describe, expect, it } from "vitest";
import {
  pluginFindings,
  snapshotCases,
  snapshotDocument,
  snapshotSuite,
} from "./diagnostic-snapshot.mjs";
import { __dirname } from "./helpers.mjs";

function fixture(name) {
  return readFileSync(
    resolve(__dirname, "../../../test-cases/eslint-plugin/upstreamed-generic/fixture", name),
    "utf8",
  );
}

const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};
const typescript = { parser: tsParser, sourceType: "module", globals: {} };

describe("upstreamed generic rule coverage", () => {
  it("snapshots remaining diagnostics", async () => {
    const code = fixture("coverage.tsx");
    const coverage = { code, filename: "coverage.tsx", languageOptions: typescript };
    await expect(
      snapshotDocument({
        "await-array-methods": snapshotSuite("await-array-methods", languageOptions, {
          invalid: [coverage],
        }),
        "nextjs-metadata-exports-location": snapshotSuite(
          "nextjs-metadata-exports-location",
          languageOptions,
          {
            invalid: [
              {
                code,
                filename: "app/components/meta.tsx",
                languageOptions: typescript,
              },
            ],
          },
        ),
        "no-delete-property": snapshotSuite("no-delete-property", languageOptions, {
          invalid: [coverage],
        }),
        "no-vitest-sequential": snapshotSuite("no-vitest-sequential", languageOptions, {
          invalid: [
            // test.sequential.each reports once, on the inner call. 6 sequential diagnostics.
            coverage,
          ],
        }),
      }),
    ).toMatchFileSnapshot("snapshots/upstreamed-rules-coverage-remaining.snap");
  }, 30_000);

  it("snapshots test and playwright diagnostics", async () => {
    const code = fixture("coverage.tsx");
    const timeoutCap = { max: 20000 };
    const cases = [
      {
        file: "coverage.tsx",
        findings: pluginFindings(code, "test-no-error-message-matching", undefined, "coverage.tsx"),
        name: "coverage.tsx test-no-error-message-matching",
        rule: "test-no-error-message-matching",
      },
      {
        file: "message-angle.ts",
        findings: pluginFindings(
          fixture("message-angle.ts"),
          "test-no-error-message-matching",
          undefined,
          "message-angle.ts",
        ),
        name: "message-angle.ts",
        rule: "test-no-error-message-matching",
      },
      {
        file: "coverage.tsx",
        findings: pluginFindings(code, "test-no-shared-state", undefined, "coverage.tsx"),
        name: "coverage.tsx test-no-shared-state",
        rule: "test-no-shared-state",
      },
      {
        file: "e2e.spec.tsx",
        findings: pluginFindings(code, "playwright-selector-priority", undefined, "e2e.spec.tsx"),
        name: "coverage.tsx e2e.spec.tsx playwright-selector-priority",
        rule: "playwright-selector-priority",
      },
      {
        file: "e2e.spec.tsx",
        findings: pluginFindings(
          code,
          "playwright-assertion-timeout-cap",
          undefined,
          "e2e.spec.tsx",
        ),
        name: "coverage.tsx e2e.spec.tsx playwright-assertion-timeout-cap",
        rule: "playwright-assertion-timeout-cap",
      },
      {
        file: "e2e.spec.tsx",
        findings: pluginFindings(
          code,
          "playwright-assertion-timeout-cap",
          timeoutCap,
          "e2e.spec.tsx",
        ),
        name: `coverage.tsx e2e.spec.tsx playwright-assertion-timeout-cap ${JSON.stringify(timeoutCap)}`,
        options: timeoutCap,
        rule: "playwright-assertion-timeout-cap",
      },
      {
        file: "unit.tsx",
        findings: pluginFindings(code, "playwright-assertion-timeout-cap", undefined, "unit.tsx"),
        name: "coverage.tsx unit.tsx playwright-assertion-timeout-cap",
        rule: "playwright-assertion-timeout-cap",
      },
      {
        file: "unit.tsx",
        findings: pluginFindings(code, "playwright-selector-priority", undefined, "unit.tsx"),
        name: "coverage.tsx unit.tsx playwright-selector-priority",
        rule: "playwright-selector-priority",
      },
    ];
    await expect(snapshotDocument(snapshotCases(cases))).toMatchFileSnapshot(
      "snapshots/upstreamed-rules-coverage.snap",
    );
  });
});
