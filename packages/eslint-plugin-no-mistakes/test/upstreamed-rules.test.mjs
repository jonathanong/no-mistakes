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

function lintCase(code, filename) {
  const item = { code, filename };
  if (/\.[cm]?tsx?$/.test(filename)) item.languageOptions = typescript;
  return item;
}

describe("upstreamed generic rules", () => {
  it("snapshots remaining diagnostics", async () => {
    const validCode = fixture("valid.tsx");
    const invalidCode = fixture("invalid.tsx");
    await expect(
      snapshotDocument({
        "await-array-methods": snapshotSuite("await-array-methods", languageOptions, {
          valid: [lintCase(validCode, "valid.tsx")],
          invalid: [lintCase(invalidCode, "invalid.tsx")],
        }),
        "nextjs-metadata-exports-location": snapshotSuite(
          "nextjs-metadata-exports-location",
          languageOptions,
          {
            valid: [lintCase(validCode, "app/page.tsx")],
          },
        ),
        "nextjs-no-manual-script-tags": snapshotSuite(
          "nextjs-no-manual-script-tags",
          languageOptions,
          {
            valid: [
              lintCase(validCode, "valid.tsx"),
              lintCase(invalidCode, "components/widget.tsx"),
            ],
            invalid: [
              lintCase(invalidCode, "app/page.tsx"),
              lintCase(fixture("next-script-import.tsx"), "components/widget.tsx"),
            ],
          },
        ),
        "no-delete-property": snapshotSuite("no-delete-property", languageOptions, {
          valid: [lintCase(validCode, "valid.tsx")],
          invalid: [lintCase(invalidCode, "invalid.tsx")],
        }),
        "no-import-only-test-files": snapshotSuite("no-import-only-test-files", languageOptions, {
          valid: [lintCase(fixture("import-only.valid.test.ts"), "import-only.valid.test.ts")],
          invalid: [
            lintCase(fixture("import-only.invalid.test.ts"), "import-only.invalid.test.ts"),
            lintCase(fixture("import-only.invalid.test.cjs"), "import-only.invalid.test.cjs"),
          ],
        }),
        "no-placeholder-never-type-exports": snapshotSuite(
          "no-placeholder-never-type-exports",
          languageOptions,
          {
            valid: [lintCase(validCode, "valid.tsx")],
            invalid: [lintCase(invalidCode, "invalid.tsx")],
          },
        ),
        "no-vitest-sequential": snapshotSuite("no-vitest-sequential", languageOptions, {
          valid: [lintCase(validCode, "valid.tsx")],
          invalid: [lintCase(invalidCode, "invalid.tsx")],
        }),
        "react-no-iife-in-jsx": snapshotSuite("react-no-iife-in-jsx", languageOptions, {
          valid: [lintCase(validCode, "valid.tsx")],
          invalid: [
            lintCase(invalidCode, "invalid.tsx"),
            // JSX IIFE invocations inside expression arrays.
            {
              code: `
      export default function Component() {
        return <>{[ (function renderTag() { return "tag"; })() ]}</>;
      }
    `,
              filename: "fixture.jsx",
            },
          ],
        }),
        "react-no-use-promise-resolve": snapshotSuite(
          "react-no-use-promise-resolve",
          languageOptions,
          {
            valid: [lintCase(validCode, "valid.tsx")],
            invalid: [lintCase(invalidCode, "invalid.tsx")],
          },
        ),
        "vitest-mock-test-file-naming": snapshotSuite(
          "vitest-mock-test-file-naming",
          languageOptions,
          {
            valid: [
              lintCase(fixture("mock-name.valid.mock.test.ts"), "mock-name.valid.mock.test.ts"),
              lintCase(fixture("mock-name.valid.test.ts"), "mock-name.valid.test.ts"),
              // jest.mock via @jest/globals import should satisfy .mock.test naming.
              lintCase(
                fixture("mock-name.jest-import.mock.test.ts"),
                "mock-name.jest-import.mock.test.ts",
              ),
              // jest.mock via require('@jest/globals') should satisfy .mock.test naming.
              {
                code: fixture("mock-name.jest-require.mock.test.cjs"),
                filename: "mock-name.jest-require.mock.test.cjs",
              },
              // typed vi.fn<T>() callback stub with no module mocking should not require .mock.test (#388).
              lintCase(
                fixture("mock-name.fn-stub.valid.test.ts"),
                "mock-name.fn-stub.valid.test.ts",
              ),
            ],
            invalid: [
              lintCase(fixture("mock-name.invalid.test.ts"), "mock-name.invalid.test.ts"),
              lintCase(fixture("mock-name.invalid.mock.test.ts"), "mock-name.invalid.mock.test.ts"),
              lintCase(
                fixture("mock-name.global.invalid.test.ts"),
                "mock-name.global.invalid.test.ts",
              ),
              {
                code: fixture("mock-name.cjs.invalid.test.cjs"),
                filename: "mock-name.cjs.invalid.test.cjs",
              },
              // vi.fn() alone does not justify a .mock.test filename (#388).
              lintCase(
                fixture("mock-name.fn-stub.invalid.mock.test.ts"),
                "mock-name.fn-stub.invalid.mock.test.ts",
              ),
            ],
          },
        ),
      }),
    ).toMatchFileSnapshot("snapshots/upstreamed-rules-remaining.snap");
  }, 30_000);

  it("snapshots test and playwright diagnostics", async () => {
    const valid = fixture("valid.tsx");
    const invalid = fixture("invalid.tsx");
    const playwrightInvalid = fixture("playwright.invalid.ts");
    const playwrightValid = fixture("playwright.valid.ts");
    const cases = [
      {
        file: "valid.tsx",
        findings: pluginFindings(valid, "test-no-shared-state", undefined, "valid.tsx"),
        name: "valid.tsx test-no-shared-state",
        rule: "test-no-shared-state",
      },
      {
        file: "valid.tsx",
        findings: pluginFindings(valid, "test-no-error-message-matching", undefined, "valid.tsx"),
        name: "valid.tsx test-no-error-message-matching",
        rule: "test-no-error-message-matching",
      },
      {
        file: "invalid.tsx",
        findings: pluginFindings(invalid, "test-no-shared-state", undefined, "invalid.tsx"),
        name: "invalid.tsx test-no-shared-state",
        rule: "test-no-shared-state",
      },
      {
        file: "invalid.tsx",
        findings: pluginFindings(
          invalid,
          "test-no-error-message-matching",
          undefined,
          "invalid.tsx",
        ),
        name: "invalid.tsx test-no-error-message-matching",
        rule: "test-no-error-message-matching",
      },
      {
        file: "shared-state-shadow.invalid.test.ts",
        findings: pluginFindings(
          fixture("shared-state-shadow.invalid.test.ts"),
          "test-no-shared-state",
          undefined,
          "shared-state-shadow.invalid.test.ts",
        ),
        name: "shared-state-shadow.invalid.test.ts",
        rule: "test-no-shared-state",
      },
      {
        file: "e2e.spec.ts",
        findings: pluginFindings(
          playwrightInvalid,
          "playwright-assertion-timeout-cap",
          undefined,
          "e2e.spec.ts",
        ),
        name: "playwright.invalid.ts playwright-assertion-timeout-cap",
        rule: "playwright-assertion-timeout-cap",
      },
      {
        file: "e2e.spec.ts",
        findings: pluginFindings(
          playwrightInvalid,
          "playwright-selector-priority",
          undefined,
          "e2e.spec.ts",
        ),
        name: "playwright.invalid.ts playwright-selector-priority",
        rule: "playwright-selector-priority",
      },
      {
        file: "e2e.spec.ts",
        findings: pluginFindings(
          playwrightValid,
          "playwright-assertion-timeout-cap",
          undefined,
          "e2e.spec.ts",
        ),
        name: "playwright.valid.ts playwright-assertion-timeout-cap",
        rule: "playwright-assertion-timeout-cap",
      },
      {
        file: "e2e.spec.ts",
        findings: pluginFindings(
          playwrightValid,
          "playwright-selector-priority",
          undefined,
          "e2e.spec.ts",
        ),
        name: "playwright.valid.ts playwright-selector-priority",
        rule: "playwright-selector-priority",
      },
    ];
    await expect(snapshotDocument(snapshotCases(cases))).toMatchFileSnapshot(
      "snapshots/upstreamed-rules.snap",
    );
  });
});
