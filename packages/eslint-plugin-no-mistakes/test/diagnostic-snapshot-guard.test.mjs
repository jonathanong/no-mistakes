import { expect, it } from "vitest";
import { resolve } from "node:path";
import { __dirname } from "./helpers.mjs";
import { snapshotSuite } from "./diagnostic-snapshot.mjs";

const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};

it("counts only this rule when a case also has a parse error", () => {
  const rows = snapshotSuite("no-delete-property", languageOptions, {
    valid: [{ code: "const =", filename: "parse-only.js" }],
  });
  expect(rows[0].findings.map((item) => item.ruleId)).toEqual([null]);
  expect(() =>
    snapshotSuite("no-delete-property", languageOptions, {
      invalid: [{ code: "const =", filename: "parse-only.js" }],
    }),
  ).toThrow(/produced 0 diagnostics/);
});

it("preserves regex escapes in snapshot options", () => {
  const options = [{ interactiveComponents: ["/\\.Item$/", resolve(__dirname, "fixture.jsx")] }];
  const rows = snapshotSuite("playwright-require-interactive-test-id", languageOptions, {
    valid: [{ code: "const value = 1;", options }],
  });
  expect(rows[0].options).toEqual([
    {
      interactiveComponents: [
        "/\\.Item$/",
        "<root>/packages/eslint-plugin-no-mistakes/test/fixture.jsx",
      ],
    },
  ]);
});
