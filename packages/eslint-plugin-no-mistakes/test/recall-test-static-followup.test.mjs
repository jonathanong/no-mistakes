import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readdirSync } from "node:fs";
import { resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { describe, expect, it } from "vitest";
import {
  pluginFindings,
  snapshotCases,
  snapshotDocument,
  snapshotSuite,
} from "./diagnostic-snapshot.mjs";
import { __dirname, fixture, lint, messages } from "./helpers.mjs";

const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};

function lintCase(code, filename, sourceType = "module") {
  const item = { code, filename };
  if (/\.[cm]?tsx?$/.test(filename)) {
    item.languageOptions = { parser: tsParser, sourceType, globals: {} };
  } else if (sourceType !== "module") {
    item.languageOptions = { sourceType };
  }
  return item;
}

function explicitCase(rule, file) {
  return lintCase(fixture(`${rule}/${file}`), file);
}

// ESLint diagnostics for fixture files the Oxlint loop compares, excluding
// cases already listed above. Filename and sourceType match that lint() call.
function oxlintEslintCases(rule, skip) {
  const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", rule);
  const files = readdirSync(root, { recursive: true })
    .map(String)
    .filter((file) => /\.[cm]?[jt]sx?$/.test(file))
    .filter(
      (file) => rule !== "no-import-only-test-files" || file !== "require-alias-cycle.test.js",
    )
    .filter((file) => !skip.has(file))
    .sort();
  const valid = [];
  const invalid = [];
  for (const file of files) {
    const filename = resolve(root, file);
    const sourceType = /\.(cjs|cts)$/.test(file)
      ? "commonjs"
      : file === "captured-with.js"
        ? "script"
        : "module";
    const code = fixture(`${rule}/${file}`);
    const ids = messages(code, rule, undefined, filename, {}, sourceType).filter(Boolean);
    const item = lintCase(code, filename, sourceType);
    (ids.length ? invalid : valid).push(item);
  }
  return { valid, invalid };
}

describe("test rule static recall follow-up", () => {
  it("snapshots remaining diagnostics", async () => {
    const sequentialSkip = new Set(["static-apply.test.ts", "static-apply-controls.test.ts"]);
    const mockSkip = new Set([
      "static-aliases.test.ts",
      "static-alias-const.test.ts",
      "static-alias-call.test.ts",
      "static-alias-apply.test.ts",
      "static-alias-destructure.test.ts",
      "static-alias-namespace.test.ts",
      "static-alias-template.test.ts",
      "static-alias-member.test.ts",
      "static-alias-method.test.ts",
      "static-aliases-controls.test.ts",
      "static-alias-cycle.test.js",
    ]);
    const importSkip = new Set([
      "require-alias.test.js",
      ...[
        "controls",
        "shadow",
        "mutable",
        "suppressed",
        "opaque",
        "cycle",
        "redeclared",
        "dynamic",
        "extra",
        "file",
        "line",
      ].map((suffix) => `require-alias-${suffix}.test.js`),
    ]);
    const sequentialExtra = oxlintEslintCases("no-vitest-sequential", sequentialSkip);
    const mockExtra = oxlintEslintCases("vitest-mock-test-file-naming", mockSkip);
    const importExtra = oxlintEslintCases("no-import-only-test-files", importSkip);
    const postgresExtra = oxlintEslintCases("postgres-no-correlated-sql-builder", new Set());
    const reactExtra = oxlintEslintCases("react-no-nullish-react-node", new Set());
    await expect(
      snapshotDocument({
        "no-import-only-test-files": snapshotSuite("no-import-only-test-files", languageOptions, {
          valid: [
            ...[
              "controls",
              "shadow",
              "mutable",
              "suppressed",
              "opaque",
              "cycle",
              "redeclared",
              "dynamic",
              "extra",
              "file",
              "line",
            ].map((suffix) =>
              explicitCase("no-import-only-test-files", `require-alias-${suffix}.test.js`),
            ),
            ...importExtra.valid,
          ],
          invalid: [
            explicitCase("no-import-only-test-files", "require-alias.test.js"),
            ...importExtra.invalid,
          ],
        }),
        "no-vitest-sequential": snapshotSuite("no-vitest-sequential", languageOptions, {
          valid: [
            explicitCase("no-vitest-sequential", "static-apply-controls.test.ts"),
            ...sequentialExtra.valid,
          ],
          invalid: [
            // 4 sequential diagnostics from literal apply arguments.
            explicitCase("no-vitest-sequential", "static-apply.test.ts"),
            ...sequentialExtra.invalid,
          ],
        }),
        "postgres-no-correlated-sql-builder": snapshotSuite(
          "postgres-no-correlated-sql-builder",
          languageOptions,
          postgresExtra,
        ),
        "react-no-nullish-react-node": snapshotSuite(
          "react-no-nullish-react-node",
          languageOptions,
          reactExtra,
        ),
        "vitest-mock-test-file-naming": snapshotSuite(
          "vitest-mock-test-file-naming",
          languageOptions,
          {
            valid: [
              explicitCase("vitest-mock-test-file-naming", "static-aliases-controls.test.ts"),
              explicitCase("vitest-mock-test-file-naming", "static-alias-cycle.test.js"),
              ...mockExtra.valid,
            ],
            invalid: [
              explicitCase("vitest-mock-test-file-naming", "static-aliases.test.ts"),
              ...[
                "const",
                "call",
                "apply",
                "destructure",
                "namespace",
                "template",
                "member",
                "method",
              ].map((name) =>
                explicitCase("vitest-mock-test-file-naming", `static-alias-${name}.test.ts`),
              ),
              ...mockExtra.invalid,
            ],
          },
        ),
      }),
    ).toMatchFileSnapshot("snapshots/recall-test-static-followup-remaining.snap");
  }, 30_000);

  it("snapshots test diagnostics", async () => {
    const specs = [
      ["test-no-error-message-matching", "static-runner.test.ts"],
      ["test-no-error-message-matching", "static-runner-controls.test.ts"],
      ["test-no-error-message-matching", "static-runner-aliases.test.ts"],
      ["test-no-error-message-matching", "static-runner-module.test.ts"],
      ["test-no-error-message-matching", "static-runner-opaque.test.ts"],
      ["test-no-error-message-matching", "static-namespace-write.test.js"],
      ["test-no-delayed-rejects", "playwright-static-expect-controls.ts"],
    ];
    const cases = specs.map(([rule, file]) => ({
      file,
      findings: pluginFindings(fixture(`${rule}/${file}`), rule, undefined, file),
      name: file,
      rule,
    }));
    await expect(snapshotDocument(snapshotCases(cases))).toMatchFileSnapshot(
      "snapshots/recall-test-static-followup.snap",
    );
  });
  for (const rule of [
    "no-import-only-test-files",
    "no-vitest-sequential",
    "postgres-no-correlated-sql-builder",
    "react-no-nullish-react-node",
    "test-no-error-message-matching",
    "vitest-mock-test-file-naming",
  ]) {
    it(`matches type-aware Oxlint for every supported ${rule} fixture`, () => {
      const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", rule);
      const files = readdirSync(root, { recursive: true })
        .filter((file) => /\.[cm]?[jt]sx?$/.test(file))
        // Oxlint itself aborts on this TDZ cycle, even with an empty config.
        .filter(
          (file) => rule !== "no-import-only-test-files" || file !== "require-alias-cycle.test.js",
        )
        .sort();
      assert.ok(files.length > 0);
      const result = spawnSync(
        process.execPath,
        [
          resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
          "--type-aware",
          "--config",
          resolve(root, ".oxlintrc-static-followup.json"),
          "--format",
          "json",
          ...files.map((file) => resolve(root, file)),
        ],
        { encoding: "utf8", timeout: 4000, maxBuffer: 4 * 1024 * 1024 },
      );
      assert.equal(result.error, undefined, `${rule}: ${result.error}`);
      assert.equal(result.signal, null, `${rule}: ${result.stderr}`);
      const actual = JSON.parse(result.stdout).diagnostics.filter((item) =>
        item.code.includes(rule),
      );
      for (const file of files) {
        const expected = lint(
          fixture(`${rule}/${file}`),
          { [`no-mistakes/${rule}`]: "error" },
          resolve(root, file),
          {},
          /\.(cjs|cts)$/.test(file)
            ? "commonjs"
            : file === "captured-with.js"
              ? "script"
              : "module",
        );
        assert.ok(
          expected.every((item) => !item.fatal),
          `${rule}/${file}: invalid fixture mode`,
        );
        const lines = actual
          .filter((item) => resolve(item.filename) === resolve(root, file))
          .map((item) => item.labels[0].span.line)
          .sort((a, b) => a - b);
        assert.deepEqual(
          lines,
          expected.map((item) => item.line).sort((a, b) => a - b),
          `${rule}/${file}`,
        );
      }
      assert.equal(result.status, actual.length ? 1 : 0, `${rule}: ${result.stderr}`);
    });
  }
});
