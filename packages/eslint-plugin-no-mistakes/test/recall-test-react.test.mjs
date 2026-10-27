import tsParser from "@typescript-eslint/parser";
import { describe, expect, it } from "vitest";
import {
  configuredFindings,
  pluginFindings,
  snapshotCases,
  snapshotDocument,
  snapshotSuite,
} from "./diagnostic-snapshot.mjs";
import { fixture } from "./helpers.mjs";

const snippet = (name) => fixture(name);
const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};
const typescript = { parser: tsParser, sourceType: "module", globals: {} };

function tsCase(ruleDirectory, filename) {
  return {
    code: snippet(`${ruleDirectory}/${filename}`),
    filename,
    languageOptions: typescript,
  };
}

function jsCase(ruleDirectory, filename) {
  return { code: snippet(`${ruleDirectory}/${filename}`), filename };
}

describe("test and react same-file recall", () => {
  it("snapshots remaining diagnostics", async () => {
    await expect(
      snapshotDocument({
        "no-import-only-test-files": snapshotSuite("no-import-only-test-files", languageOptions, {
          valid: [
            jsCase("no-import-only-test-files", "dynamic.test.js"),
            jsCase("no-import-only-test-files", "suppressed.test.js"),
            jsCase("no-import-only-test-files", "dynamic-forward.test.js"),
            jsCase("no-import-only-test-files", "next-line.test.js"),
          ],
          invalid: [
            // An expression-free template require is a test import.
            jsCase("no-import-only-test-files", "recall.test.js"),
            jsCase("no-import-only-test-files", "forwarded.test.js"),
          ],
        }),
        "no-vitest-sequential": snapshotSuite("no-vitest-sequential", languageOptions, {
          invalid: [
            // 15 sequential diagnostics through aliases and static keys.
            tsCase("no-vitest-sequential", "recall.test.ts"),
            tsCase("no-vitest-sequential", "created-runner.test.ts"),
            tsCase("no-vitest-sequential", "renamed-fixture.test.ts"),
            tsCase("no-vitest-sequential", "default-import.test.ts"),
            tsCase("no-vitest-sequential", "module-function.test.ts"),
            tsCase("no-vitest-sequential", "module-let.test.ts"),
            tsCase("no-vitest-sequential", "module-var.test.ts"),
            tsCase("no-vitest-sequential", "const-alias.test.ts"),
          ],
        }),
        "vitest-mock-test-file-naming": snapshotSuite(
          "vitest-mock-test-file-naming",
          languageOptions,
          {
            valid: [tsCase("vitest-mock-test-file-naming", "suppressed.test.ts")],
            invalid: [
              // Namespace module mocks require a .mock.test name.
              tsCase("vitest-mock-test-file-naming", "recall.test.ts"),
              // The default export is not a module mock.
              tsCase("vitest-mock-test-file-naming", "default.mock.test.ts"),
            ],
          },
        ),
      }),
    ).toMatchFileSnapshot("snapshots/recall-test-react-remaining.snap");
  }, 30_000);

  it("snapshots test diagnostics", async () => {
    const groups = [
      [
        "test-no-error-message-matching",
        [
          "recall.test.ts",
          "playwright-expect.test.ts",
          "setup-expect.test.ts",
          "bun-expect.test.ts",
          "chai-expect.test.ts",
          "cjs-expect.test.ts",
          "local-expect.test.ts",
          "function-expect.test.ts",
          "let-expect.test.ts",
          "bracket-call.test.ts",
          "binary-suppressed.test.ts",
          "matcher-call.test.ts",
        ],
      ],
      [
        "test-no-shared-state",
        [
          "recall.test.ts",
          "computed-describe.test.ts",
          "alias-describe.test.ts",
          "sequence-describe.test.ts",
          "serial-alias.test.ts",
          "shadow-test-alias.test.ts",
          "namespace-serial.test.ts",
          "suppressed-alias.test.ts",
        ],
      ],
      ["test-no-delayed-rejects", ["recall.ts", "poll.ts", "suppressed.ts"]],
    ];
    const cases = groups.flatMap(([rule, files]) =>
      files.map((file) => ({
        file,
        findings: pluginFindings(snippet(`${rule}/${file}`), rule, undefined, file),
        name: `${rule} ${file}`,
        rule,
      })),
    );
    cases.push(
      {
        file: "commonjs.test.cjs",
        findings: pluginFindings(
          snippet("test-no-error-message-matching/commonjs.test.cjs"),
          "test-no-error-message-matching",
          undefined,
          "commonjs.test.cjs",
          {},
          "commonjs",
        ),
        name: "test-no-error-message-matching commonjs.test.cjs",
        rule: "test-no-error-message-matching",
        sourceType: "commonjs",
      },
      {
        file: "shadow-namespace.test.ts",
        findings: configuredFindings(
          snippet("test-no-shared-state/shadow-namespace.test.ts"),
          { "no-mistakes/test-no-shared-state": "error" },
          "shadow-namespace.test.ts",
        ),
        name: "test-no-shared-state shadow-namespace.test.ts",
        rule: "test-no-shared-state",
      },
    );
    await expect(snapshotDocument(snapshotCases(cases))).toMatchFileSnapshot(
      "snapshots/recall-test-react.snap",
    );
  });
});
