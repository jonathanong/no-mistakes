import { describe, expect, it } from "vitest";
import tsParser from "@typescript-eslint/parser";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { fixture } from "./helpers.mjs";

const rule = "no-vitest-sequential";
const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};

// Visible options stay reported. Unknown argument arrays stay alone.
// 3 sequential.
const suite = {
  invalid: [
    {
      code: fixture("no-vitest-sequential/visible-forwarded-options.test.ts"),
      filename: "fixture.test.ts",
      languageOptions: { parser: tsParser },
    },
  ],
};

describe("visible sequential options in opaque forwarding", () => {
  it("snapshots no-vitest-sequential-visible-options diagnostics", async () => {
    await expect(
      snapshotDocument({
        [rule]: snapshotSuite(rule, languageOptions, suite),
      }),
    ).toMatchFileSnapshot("snapshots/no-vitest-sequential-visible-options.snap");
  }, 30_000);
});
