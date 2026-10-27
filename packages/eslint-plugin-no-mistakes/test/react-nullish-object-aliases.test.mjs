import { describe, expect, it } from "vitest";
import tsParser from "@typescript-eslint/parser";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { fixture } from "./helpers.mjs";

const rule = "react-no-nullish-react-node";
const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};

function aliasCase(name) {
  return {
    code: fixture(`${rule}/${name}.tsx`),
    filename: "fixture.tsx",
    languageOptions: { parser: tsParser },
  };
}

const suite = {
  valid: [
    aliasCase("object-alias-rewrite"),
    aliasCase("object-alias-suppressed"),
    aliasCase("object-alias-file-suppressed"),
  ],
  invalid: [
    // 5 nullish.
    aliasCase("object-aliases"),
    // 3 nullish.
    aliasCase("object-alias-loop-bindings"),
    // 8 nullish.
    aliasCase("object-alias-target-shadows"),
    // 1 nullish.
    aliasCase("object-alias-controls"),
  ],
};

describe("ReactNode prop object aliases and template destructuring", () => {
  it("snapshots react-nullish-object-aliases diagnostics", async () => {
    await expect(
      snapshotDocument({
        [rule]: snapshotSuite(rule, languageOptions, suite),
      }),
    ).toMatchFileSnapshot("snapshots/react-nullish-object-aliases.snap");
  }, 30_000);
});
