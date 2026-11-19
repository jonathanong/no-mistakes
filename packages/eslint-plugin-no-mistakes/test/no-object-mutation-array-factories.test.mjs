import { dirname, resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import {
  languageOptionsFindings,
  snapshotDocument,
  snapshotSuite,
} from "./diagnostic-snapshot.mjs";
import { __dirname, fixture } from "./helpers.mjs";

beforeEach(() => vi.stubEnv("CI", "true"));
afterEach(() => vi.unstubAllEnvs());
const rule = "no-object-mutation";
function savedCase(name, typed = false) {
  const filename = resolve(
    __dirname,
    "../../../test-cases/eslint-snippets/fixture/no-object-mutation-array-factories",
    name,
  );
  return {
    code: fixture(`no-object-mutation-array-factories/${name}`),
    filename,
    languageOptions: {
      parser: tsParser,
      ecmaVersion: 2024,
      sourceType: "module",
      parserOptions: typed ? { projectService: true, tsconfigRootDir: dirname(filename) } : {},
    },
  };
}
it.each([false, true])("accepts builtin array producers with typed=%s", (typed) => {
  expect(languageOptionsFindings({ ...savedCase("positive.ts", typed), rule })).toEqual([]);
});
it("accepts checker-proven array and tuple receivers", () => {
  expect(languageOptionsFindings({ ...savedCase("typed.ts", true), rule })).toEqual([]);
});
it("snapshots array producer and checker proof boundaries", async () => {
  const suite = {
    valid: [
      savedCase("positive.ts"),
      savedCase("positive.ts", true),
      savedCase("typed.ts", true),
      ...["accept-only-from.ts", "accept-only-factory-escape.ts", "mixed-contents.ts"].flatMap(
        (name) => [savedCase(name), savedCase(name, true)],
      ),
      savedCase("type-names.ts"),
      savedCase("type-names.ts", true),
      savedCase("custom-lib/override.ts", true),
    ],
    invalid: [
      savedCase("typed.ts"),
      savedCase("accept-only-unknown-elements.ts"),
      savedCase("accept-only-unknown-elements.ts", true),
      savedCase("accept-only-factory-arguments.ts"),
      savedCase("accept-only-factory-arguments.ts", true),
      ...["accept-only-pending.ts", "accept-only-pending-direct.ts"].flatMap((name) => [
        savedCase(name),
        savedCase(name, true),
      ]),
      savedCase("joined-controls.ts", true),
      savedCase("this-receiver.ts"),
      savedCase("this-receiver.ts", true),
      savedCase("joined-escapes.ts"),
      savedCase("joined-escapes.ts", true),
      savedCase("negative.ts"),
      savedCase("negative.ts", true),
      savedCase("overrides.ts"),
      savedCase("overrides.ts", true),
      ...["global-alias.ts", "global-escape.ts", "prototype-alias.ts"].flatMap((name) => [
        savedCase(name),
        savedCase(name, true),
      ]),
    ],
  };
  await expect(snapshotDocument(snapshotSuite(rule, {}, suite))).toMatchFileSnapshot(
    "snapshots/no-object-mutation-array-factories.snap",
  );
});

it.each([false, true])("retains every joined escape with typed=%s", (typed) => {
  const findings = languageOptionsFindings({ ...savedCase("joined-escapes.ts", typed), rule });
  expect(findings.map((finding) => finding.line)).toEqual([8, 14, 18, 29]);
});

it.each([false, true])("rejects escaped and overridden this receivers with typed=%s", (typed) => {
  const findings = languageOptionsFindings({ ...savedCase("this-receiver.ts", typed), rule });
  expect(findings.map((finding) => finding.line)).toEqual([5, 15]);
});

it.each([false, true])(
  "preserves main diagnostics for accepted existing shapes with typed=%s",
  (typed) => {
    for (const name of ["accept-only-from.ts"]) {
      expect(languageOptionsFindings({ ...savedCase(name, typed), rule }), name).toEqual([]);
    }
  },
);

const mainDiagnostics = JSON.parse(
  fixture("no-object-mutation-array-factories/main-diagnostics.json"),
);
it.each(mainDiagnostics.cases)(
  "only removes main diagnostics: $file typed=$typed",
  ({ file, typed, diagnostics }) => {
    const actual = languageOptionsFindings({ ...savedCase(file, typed), rule }).map(
      ({ messageId, line, column, endLine, endColumn }) => ({
        messageId,
        line,
        column,
        endLine,
        endColumn,
      }),
    );
    if (file.startsWith("accept-only-")) expect(actual, file).toEqual(diagnostics);
    const allowed = diagnostics.map((finding) => JSON.stringify(finding));
    for (const finding of actual) {
      const { messageId, line, column, endLine, endColumn } = finding;
      const signature = JSON.stringify({ messageId, line, column, endLine, endColumn });
      const index = allowed.indexOf(signature);
      expect(index, `${file}: ${signature}`).toBeGreaterThanOrEqual(0);
      allowed.splice(index, 1);
    }
  },
);
