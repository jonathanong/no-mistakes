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

function lintCase(code, filename, option) {
  const item = { code, filename };
  if (/\.[cm]?tsx?$/.test(filename)) item.languageOptions = typescript;
  if (option !== undefined) item.options = [option];
  return item;
}

describe("upstreamed generic rule scoping", () => {
  it("snapshots remaining diagnostics", async () => {
    const invalidMetadata = fixture("metadata.invalid.ts");
    const validPage = fixture("metadata.valid-page.tsx");
    const inlineBoot = fixture("next-script-inline-boot.tsx");
    await expect(
      snapshotDocument({
        "nextjs-metadata-exports-location": snapshotSuite(
          "nextjs-metadata-exports-location",
          languageOptions,
          {
            valid: [
              lintCase(invalidMetadata, "lib/metadata.ts"),
              lintCase(validPage, "app/page.tsx"),
              { code: validPage, filename: "app/page.js" },
            ],
            invalid: [
              lintCase(invalidMetadata, "app/lib/metadata.ts"),
              lintCase(validPage, "pages/page.tsx"),
              lintCase(validPage, "app/template.tsx"),
            ],
          },
        ),
        "nextjs-no-manual-script-tags": snapshotSuite(
          "nextjs-no-manual-script-tags",
          languageOptions,
          {
            valid: [
              lintCase(fixture("next-script-jsonld.tsx"), "app/page.tsx"),
              lintCase(inlineBoot, "app/layout.tsx", { allowInlineScriptIds: ["theme-init"] }),
              lintCase(inlineBoot, "app/layout.tsx", { allowInlineScriptIdPatterns: ["^theme-"] }),
              lintCase(inlineBoot, "app/layout.tsx", {
                allowInlineScriptIdPatterns: ["[", "^theme-"],
              }),
            ],
            invalid: [
              lintCase(inlineBoot, "app/layout.tsx"),
              lintCase(inlineBoot, "app/layout.tsx", { allowInlineScriptIdPatterns: ["["] }),
            ],
          },
        ),
        "no-placeholder-never-type-exports": snapshotSuite(
          "no-placeholder-never-type-exports",
          languageOptions,
          {
            valid: [lintCase(fixture("placeholder-shadow.ts"), "shadow.ts")],
          },
        ),
        "react-no-use-promise-resolve": snapshotSuite(
          "react-no-use-promise-resolve",
          languageOptions,
          {
            valid: [lintCase(fixture("react-use-shadow.tsx"), "shadow.tsx")],
          },
        ),
      }),
    ).toMatchFileSnapshot("snapshots/upstreamed-rules-scope-remaining.snap");
  }, 30_000);

  it("snapshots test-no-error-message-matching diagnostics", async () => {
    const cases = [
      {
        file: "static-code-analysis/rules/example.test.ts",
        findings: pluginFindings(
          fixture("diagnostic-message.valid.test.ts"),
          "test-no-error-message-matching",
          undefined,
          "static-code-analysis/rules/example.test.ts",
        ),
        name: "diagnostic-message.valid.test.ts",
        rule: "test-no-error-message-matching",
      },
      {
        file: "backend/services/email-address-validator/errors.generated.test.mts",
        findings: pluginFindings(
          fixture("generated-message.generated.test.ts"),
          "test-no-error-message-matching",
          undefined,
          "backend/services/email-address-validator/errors.generated.test.mts",
        ),
        name: "generated-message.generated.test.ts",
        rule: "test-no-error-message-matching",
      },
    ];
    await expect(snapshotDocument(snapshotCases(cases))).toMatchFileSnapshot(
      "snapshots/upstreamed-rules-scope.snap",
    );
  });
});
