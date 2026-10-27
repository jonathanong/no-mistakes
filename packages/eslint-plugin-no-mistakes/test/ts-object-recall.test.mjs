import { describe, expect, it } from "vitest";
import tsParser from "@typescript-eslint/parser";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { fixture } from "./helpers.mjs";

const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};
const tsLanguage = { parser: tsParser };

function recallCase(name, options) {
  const item = {
    code: fixture(`ts-object-recall/${name}.ts`),
    filename: "fixture.ts",
    languageOptions: tsLanguage,
  };
  if (options !== undefined) item.options = [options];
  return item;
}

const aliases = {
  invalid: [
    // 4 alias. Only casts with a proven unchanged callable or result type.
    recallCase("asserted-functions"),
  ],
  valid: [
    // All three directive forms suppress asserted forwarding.
    recallCase("asserted-suppressed"),
    recallCase("asserted-file-suppressed"),
  ],
};

describe("TypeScript and object recall audit", () => {
  it("snapshots ts-object-recall diagnostics", async () => {
    await expect(
      snapshotDocument({
        "no-delete-property": snapshotSuite("no-delete-property", languageOptions, {
          invalid: [
            // 6 delete.
            recallCase("delete-property"),
          ],
        }),
        "no-object-mutation": snapshotSuite("no-object-mutation", languageOptions, {
          invalid: [
            // 4 mutation.
            recallCase("object-mutators"),
          ],
        }),
        "no-placeholder-never-type-exports": snapshotSuite(
          "no-placeholder-never-type-exports",
          languageOptions,
          {
            invalid: [
              // 2 placeholder.
              recallCase("placeholder-types"),
            ],
          },
        ),
        "ts-no-const-aliases": snapshotSuite("ts-no-const-aliases", languageOptions, {
          invalid: [
            // 1 alias.
            recallCase("const-aliases"),
          ],
        }),
        "ts-no-export-renaming": snapshotSuite("ts-no-export-renaming", languageOptions, {
          invalid: [
            // 2 renamed.
            recallCase("export-renaming"),
          ],
        }),
        "ts-no-function-aliases": snapshotSuite("ts-no-function-aliases", languageOptions, aliases),
        "ts-preserve-null-option-defaults": snapshotSuite(
          "ts-preserve-null-option-defaults",
          languageOptions,
          {
            invalid: [
              // 3 default.
              recallCase("null-options"),
            ],
          },
        ),
        "vitest-timeout-cap": snapshotSuite("vitest-timeout-cap", languageOptions, {
          valid: [
            // Parameter-passed budgets stay unresolved without the finding policy.
            recallCase("vitest-parameter"),
          ],
          invalid: [
            // 1 unknown.
            recallCase("vitest-parameter", { unknownValues: "finding" }),
          ],
        }),
      }),
    ).toMatchFileSnapshot("snapshots/ts-object-recall.snap");
  }, 30_000);
});
