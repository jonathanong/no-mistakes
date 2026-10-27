import { expect, it } from "vitest";
import { resolve } from "node:path";
import { __dirname } from "./helpers.mjs";
import { relativeDiagnosticFile, snapshotSuite } from "./diagnostic-snapshot.mjs";

const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};

it.each(["valid", "invalid"])("rejects fatal parse errors in %s cases", (kind) => {
  expect(() =>
    snapshotSuite("no-delete-property", languageOptions, {
      [kind]: [{ code: "const =", filename: "parse-only.js" }],
    }),
  ).toThrow(/parse-only\.js failed to parse/);
});

it("counts only the selected rule for nonfatal diagnostics", () => {
  const item = { code: "// eslint-disable-next-line missing-rule\nconst value = 1;" };
  const rows = snapshotSuite("no-delete-property", languageOptions, { valid: [item] });
  expect(rows[0].findings.map((finding) => finding.ruleId)).toEqual(["missing-rule"]);
  expect(() => snapshotSuite("no-delete-property", languageOptions, { invalid: [item] })).toThrow(
    /produced 0 diagnostics/,
  );
});

it("labels absolute paths relative to their package or repository root", () => {
  expect(relativeDiagnosticFile(resolve(__dirname, "../web/app/index.ts"))).toBe(
    "web/app/index.ts",
  );
  expect(relativeDiagnosticFile(resolve(__dirname, "../../../web/app/index.ts"))).toBe(
    "web/app/index.ts",
  );
});

it("preserves regex escapes in snapshot options", () => {
  const options = [{ interactiveComponents: ["/\\.Item$/", resolve(__dirname, "fixture.jsx")] }];
  const rows = snapshotSuite("playwright-require-interactive-test-id", languageOptions, {
    valid: [{ code: "const value = 1;", options }],
  });
  expect(rows[0].options).toEqual([
    {
      interactiveComponents: ["/\\.Item$/", "<root>/test/fixture.jsx"],
    },
  ]);
});
