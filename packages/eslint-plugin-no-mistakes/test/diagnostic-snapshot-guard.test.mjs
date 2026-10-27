import { expect, it } from "vitest";
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
