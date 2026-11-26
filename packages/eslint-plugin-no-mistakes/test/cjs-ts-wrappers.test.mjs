import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import tsParser from "@typescript-eslint/parser";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { __dirname, fixture, lint } from "./helpers.mjs";

const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  parserOptions: { ecmaFeatures: { jsx: true } },
};

// Each directory's saved Oxlint config is the option set for its fixtures.
const groups = [
  {
    rule: "no-vitest-sequential",
    dir: "sequential",
    invalid: ["helper.js", "default-import.test.ts", "wrappers.test.ts", "dynamic.test.ts"],
    valid: ["not-a-runner.test.ts"],
  },
  {
    rule: "no-vitest-sequential",
    dir: "sequential-ignore",
    options: [{ unanalyzable: "ignore" }],
    valid: ["dynamic.test.ts"],
  },
  {
    rule: "module-mock-boundary",
    dir: "boundary",
    invalid: ["mocks.test.ts", "dynamic.test.ts"],
  },
  {
    rule: "module-mock-boundary",
    dir: "boundary-ignore",
    options: [{ unanalyzable: "ignore" }],
    valid: ["dynamic.test.ts"],
  },
  {
    rule: "module-mock-preserve-exports",
    dir: "preserve",
    invalid: ["mocks.test.ts", "dynamic.test.ts"],
  },
  {
    rule: "module-mock-preserve-exports",
    dir: "preserve-ignore",
    options: [{ unanalyzable: "ignore" }],
    valid: ["dynamic.test.ts"],
  },
  {
    rule: "vitest-mock-test-file-naming",
    dir: "naming",
    invalid: [
      "foo.test.ts",
      "cast-method.test.ts",
      "cast-namespace.test.ts",
      "computed-key.test.ts",
      "import-equals.test.ts",
      "dynamic.test.ts",
    ],
    valid: ["foo.mock.test.ts", "expect-only.test.ts"],
  },
  {
    rule: "vitest-mock-test-file-naming",
    dir: "naming-ignore",
    options: [{ unanalyzable: "ignore" }],
    valid: ["dynamic.test.ts"],
  },
  {
    rule: "test-no-shared-state",
    dir: "shared",
    invalid: ["cjs.test.ts", "wrappers.test.ts", "dynamic.test.ts"],
    valid: ["not-a-callee.test.ts"],
  },
  {
    rule: "test-no-shared-state",
    dir: "shared-ignore",
    options: [{ unanalyzable: "ignore" }],
    valid: ["dynamic.test.ts"],
  },
  {
    rule: "test-no-wall-time-assertions",
    dir: "wall",
    invalid: ["clocks.test.ts", "wrapped.test.ts", "dynamic.test.ts"],
    valid: ["fake-timers.test.ts"],
  },
  {
    rule: "test-no-wall-time-assertions",
    dir: "wall-ignore",
    options: [{ unanalyzable: "ignore" }],
    valid: ["dynamic.test.ts"],
  },
  {
    rule: "test-no-error-message-matching",
    dir: "message",
    invalid: ["cjs.test.ts", "wrappers.test.ts", "dynamic.test.ts"],
  },
  {
    rule: "test-no-error-message-matching",
    dir: "message-ignore",
    options: [{ unanalyzable: "ignore" }],
    valid: ["dynamic.test.ts"],
  },
  {
    rule: "test-no-delayed-rejects",
    dir: "delayed",
    invalid: ["cjs.test.ts", "cast.test.ts", "dynamic.test.ts"],
    valid: ["caught.test.ts"],
  },
  {
    rule: "test-no-delayed-rejects",
    dir: "delayed-ignore",
    options: [{ unanalyzable: "ignore" }],
    valid: ["dynamic.test.ts"],
  },
  {
    rule: "test-no-typed-enqueue-fake",
    dir: "enqueue-exact",
    options: [{ singleExportPattern: "^enqueue$" }],
    invalid: ["cjs.test.ts", "dynamic.test.ts"],
  },
  {
    rule: "test-no-typed-enqueue-fake",
    dir: "enqueue-prefix",
    options: [{ singleExportPattern: "^enqueue" }],
    invalid: ["wrappers.test.ts"],
  },
  {
    rule: "test-no-typed-enqueue-fake",
    dir: "enqueue-ignore",
    options: [{ singleExportPattern: "^enqueue$", unanalyzable: "ignore" }],
    valid: ["dynamic.test.ts"],
  },
  {
    rule: "vitest-timeout-cap",
    dir: "timeout",
    invalid: ["cjs.test.ts", "vitest.config.ts", "apply.test.ts"],
    valid: ["dynamic.test.ts"],
  },
  {
    rule: "vitest-timeout-cap",
    dir: "timeout-finding",
    options: [{ unknownValues: "finding" }],
    invalid: ["dynamic.test.ts"],
    valid: ["vitest.config.ts"],
  },
  {
    rule: "test-no-skips",
    dir: "skips",
    options: [{ testModules: ["./custom-test"] }],
    invalid: ["configured.test.ts", "wrappers.test.ts", "dynamic.test.ts"],
  },
  {
    rule: "test-no-skips",
    dir: "skips-ignore",
    options: [{ testModules: ["./custom-test"], unanalyzable: "ignore" }],
    valid: ["dynamic.test.ts"],
  },
  {
    rule: "test-no-fixed-temp-paths",
    dir: "temp",
    invalid: ["paths.test.ts", "dynamic.test.ts"],
  },
  {
    rule: "test-no-fixed-temp-paths",
    dir: "temp-ignore",
    options: [{ unanalyzable: "ignore" }],
    valid: ["dynamic.test.ts"],
  },
];

function sourceCase(dir, name, options) {
  return {
    code: fixture(`cjs-ts-wrappers/${dir}/${name}`),
    filename: name,
    ...(options ? { options } : {}),
    ...(name.endsWith(".ts") ? { languageOptions: { parser: tsParser } } : {}),
  };
}

function suiteFor(group) {
  return {
    valid: (group.valid ?? []).map((name) => sourceCase(group.dir, name, group.options)),
    invalid: (group.invalid ?? []).map((name) => sourceCase(group.dir, name, group.options)),
  };
}

describe("CommonJS and TypeScript wrappers", () => {
  it("snapshots every saved fixture", async () => {
    const document = {};
    for (const group of groups) {
      const key = `${group.rule}:${group.dir}`;
      document[key] = snapshotSuite(group.rule, languageOptions, suiteFor(group));
    }
    await expect(snapshotDocument(document)).toMatchFileSnapshot("snapshots/cjs-ts-wrappers.snap");
  }, 60_000);

  it("matches Oxlint line numbers for the saved fixtures", () => {
    const oxlint = resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint");
    for (const group of groups) {
      const config = resolve(
        __dirname,
        "../../../test-cases/eslint-snippets/fixture/cjs-ts-wrappers",
        group.dir,
        ".oxlintrc.json",
      );
      const saved = JSON.parse(readFileSync(config, "utf8"));
      const configured = saved.rules[`no-mistakes/${group.rule}`];
      assert.deepEqual(
        configured,
        group.options ? ["error", ...group.options] : "error",
        group.dir,
      );
      for (const name of [...(group.invalid ?? []), ...(group.valid ?? [])]) {
        const file = resolve(config, "..", name);
        const result = spawnSync(
          process.execPath,
          [oxlint, "--config", config, "--format", "json", file],
          { encoding: "utf8" },
        );
        const expected = lint(
          fixture(`cjs-ts-wrappers/${group.dir}/${name}`),
          { [`no-mistakes/${group.rule}`]: configured },
          name,
        );
        assert.equal(
          result.status,
          expected.length ? 1 : 0,
          `${group.dir}/${name}\n${result.stderr}`,
        );
        const actual = JSON.parse(result.stdout).diagnostics.filter((diagnostic) =>
          diagnostic.code.includes(group.rule),
        );
        assert.deepEqual(
          actual.map((diagnostic) => diagnostic.labels[0].span.line).sort((a, b) => a - b),
          expected.map((finding) => finding.line).sort((a, b) => a - b),
          `${group.dir}/${name}`,
        );
      }
    }
  }, 120_000);
});
