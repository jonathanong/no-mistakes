import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { basename, resolve } from "node:path";
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
    invalid: [
      "helper.js",
      "default-import.test.ts",
      "wrappers.test.ts",
      "dynamic.test.ts",
      "shapes.test.ts",
      "shadow.test.ts",
      "alias-member.test.ts",
      "var-binding.test.ts",
      "computed-chain.test.ts",
      "default-named.test.ts",
      "fixture-default.test.ts",
      "late-binding.js",
      "nested-destructure.js",
      "let-dynamic.test.ts",
      "member-require.test.ts",
      "reassigned-destructure.test.ts",
      "alias-reassigned.test.ts",
      "reassigned-test-name.test.ts",
      "each-static.test.ts",
      "each-dynamic.test.ts",
      "each-nested.test.ts",
      "pattern-shape.js",
    ],
    valid: [
      "not-a-runner.test.ts",
      "builtin-pattern.test.ts",
      "redeclare.js",
      "two-defs.test.js",
      "option-call.test.ts",
      "type-only.test.ts",
      "computed-member.test.ts",
      "alias-plain.test.ts",
      "nested-name.js",
      "computed-destructure.test.ts",
      "computed-registered.test.ts",
    ],
  },
  {
    rule: "no-vitest-sequential",
    dir: "sequential-ignore",
    options: [{ unanalyzable: "ignore" }],
    valid: [
      "dynamic.test.ts",
      "let-dynamic.test.ts",
      "reassigned-destructure.test.ts",
      "alias-reassigned.test.ts",
      "reassigned-test-name.test.ts",
      "each-dynamic.test.ts",
      "each-nested.test.ts",
    ],
  },
  {
    rule: "module-mock-boundary",
    dir: "boundary",
    invalid: [
      "mocks.test.ts",
      "dynamic.test.ts",
      "shapes.test.ts",
      "opaque-method.test.ts",
      "identifier-alias.test.ts",
      "call-apply.test.ts",
      "inline-require.test.ts",
      "inline-static.test.ts",
      "opaque-follow.test.ts",
    ],
    valid: [
      "type-only-equals.test.ts",
      "suppressed.test.ts",
      "static-member.test.ts",
      "resolved-computed.test.ts",
      "inline-other.test.ts",
      "let-alias.test.ts",
      "import-equals.test.ts",
    ],
  },
  {
    rule: "module-mock-boundary",
    dir: "boundary-ignore",
    options: [{ unanalyzable: "ignore" }],
    valid: ["dynamic.test.ts", "call-apply.test.ts"],
  },
  {
    rule: "module-mock-preserve-exports",
    dir: "preserve",
    invalid: ["mocks.test.ts", "dynamic.test.ts", "call-apply.test.ts", "opaque-follow.test.ts"],
    valid: ["suppressed.test.ts", "let-alias.test.ts"],
  },
  {
    rule: "module-mock-preserve-exports",
    dir: "preserve-ignore",
    options: [{ unanalyzable: "ignore" }],
    valid: ["dynamic.test.ts", "call-apply.test.ts"],
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
      "dynamic-forward.test.ts",
      "shapes.test.ts",
      "member-alias.test.ts",
      "alias.mock.test.ts",
      "mutable-alias.mock.test.ts",
      "pattern-alias.test.ts",
    ],
    valid: [
      "foo.mock.test.ts",
      "expect-only.test.ts",
      "default-alias.test.ts",
      "pattern-quiet.test.ts",
    ],
  },
  {
    rule: "vitest-mock-test-file-naming",
    dir: "naming-ignore",
    options: [{ unanalyzable: "ignore" }],
    valid: ["dynamic.test.ts", "pattern-alias.test.ts"],
  },
  {
    rule: "test-no-shared-state",
    dir: "shared",
    invalid: [
      "cjs.test.ts",
      "wrappers.test.ts",
      "dynamic.test.ts",
      "shapes.test.ts",
      "shadow-cast.test.ts",
      "named-callback.test.ts",
      "member-require.test.ts",
      "destructured-dynamic.test.ts",
      "member-dynamic.test.ts",
      "let-dynamic.test.ts",
      "nested-call.test.ts",
    ],
    valid: [
      "computed-modifier.test.ts",
      "computed-binding.test.ts",
      "wrapped-mock.test.ts",
      "type-import.test.ts",
      "not-a-callee.test.ts",
      "suppressed.test.ts",
      "reassign.test.ts",
      "computed-key.test.ts",
      "reassigned-dynamic.test.ts",
      "member-receiver.test.ts",
      "playwright-default.test.ts",
      "alias-let.test.ts",
    ],
  },
  {
    rule: "test-no-shared-state",
    dir: "shared-ignore",
    options: [{ unanalyzable: "ignore" }],
    valid: [
      "dynamic.test.ts",
      "destructured-dynamic.test.ts",
      "member-dynamic.test.ts",
      "let-dynamic.test.ts",
    ],
  },
  {
    rule: "test-no-wall-time-assertions",
    dir: "wall",
    invalid: [
      "clocks.test.ts",
      "wrapped.test.ts",
      "dynamic.test.ts",
      "shapes.test.ts",
      "decoy-timer.test.ts",
    ],
    valid: [
      "fake-timers.test.ts",
      "local-fluent.test.ts",
      "late-fake-timers.test.ts",
      "computed-now.test.ts",
      "default-timers.test.ts",
    ],
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
    invalid: [
      "cjs.test.ts",
      "wrappers.test.ts",
      "dynamic.test.ts",
      "require-expect.test.ts",
      "require-runner.test.ts",
      "default-namespace.test.ts",
      "spread-apply.test.ts",
      "static-apply.test.ts",
      "default-expect.test.ts",
    ],
    valid: [
      "default-direct.test.ts",
      "spread-call.test.ts",
      "spread-apply-site.test.ts",
      "named-member.test.ts",
    ],
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
    valid: ["caught.test.ts", "computed-key.test.ts"],
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
    invalid: [
      "cjs.test.ts",
      "dynamic.test.ts",
      "type-namespace.test.ts",
      "type-qualified.test.ts",
      "wrapped-fn.test.ts",
      "namespace-vi.test.ts",
      "function-type.test.ts",
      "member-vi.test.ts",
      "member-export.test.ts",
      "opaque-receiver.test.ts",
      "inline-require.test.ts",
    ],
    valid: [
      "tuple.test.ts",
      "reassign.test.ts",
      "other-export.test.ts",
      "local-parameters.test.ts",
      "opaque-quiet.test.ts",
      "computed-member.test.ts",
      "computed-vi.test.ts",
      "other-require.test.ts",
      "computed-enqueue.test.ts",
    ],
  },
  {
    rule: "test-no-typed-enqueue-fake",
    dir: "enqueue-prefix",
    options: [{ singleExportPattern: "^enqueue" }],
    invalid: ["wrappers.test.ts", "shapes.test.ts"],
  },
  {
    rule: "test-no-typed-enqueue-fake",
    dir: "enqueue-ignore",
    options: [{ singleExportPattern: "^enqueue$", unanalyzable: "ignore" }],
    valid: ["dynamic.test.ts", "opaque-receiver.test.ts"],
  },
  {
    rule: "vitest-timeout-cap",
    dir: "timeout",
    invalid: ["cjs.test.ts", "vitest.config.ts", "apply.test.ts", "call-receiver.test.ts"],
    valid: ["dynamic.test.ts", "other-receiver.test.ts"],
  },
  {
    rule: "vitest-timeout-cap",
    dir: "timeout-finding",
    options: [{ unknownValues: "finding" }],
    invalid: [
      "dynamic.test.ts",
      "pattern.config.ts",
      "bare.config.ts",
      "loop.config.ts",
      "object.config.ts",
      "rest.config.ts",
      "default.config.ts",
      "for-in.config.ts",
    ],
    valid: ["vitest.config.ts"],
  },
  {
    rule: "test-no-skips",
    dir: "skips",
    options: [{ testModules: ["./custom-test"] }],
    invalid: [
      "configured.test.ts",
      "default-import.test.ts",
      "wrappers.test.ts",
      "dynamic.test.ts",
      "awaited.test.ts",
      "member-require.js",
      "dynamic-import.test.ts",
    ],
    valid: ["not-a-runner.js", "computed-runner.js", "computed-dynamic.test.ts"],
  },
  {
    rule: "test-no-skips",
    dir: "skips-ignore",
    options: [{ testModules: ["./custom-test"], unanalyzable: "ignore" }],
    valid: ["dynamic.test.ts", "dynamic-import.test.ts"],
  },
  {
    rule: "test-no-fixed-temp-paths",
    dir: "temp",
    invalid: ["paths.test.ts", "dynamic.test.ts", "promises.test.ts"],
    valid: [
      "mock-after.test.ts",
      "other-member.test.ts",
      "computed-member.test.ts",
      "computed-pattern.test.ts",
    ],
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
      const names = [...(group.invalid ?? []), ...(group.valid ?? [])];
      const result = spawnSync(
        process.execPath,
        [
          oxlint,
          "--config",
          config,
          "--format",
          "json",
          ...names.map((name) => resolve(config, "..", name)),
        ],
        { encoding: "utf8", maxBuffer: 10 * 1024 * 1024 },
      );
      assert.ok(result.status === 0 || result.status === 1, `${group.dir}\n${result.stderr}`);
      const diagnostics = JSON.parse(result.stdout).diagnostics.filter((diagnostic) =>
        diagnostic.code.includes(group.rule),
      );
      for (const name of names) {
        const expected = lint(
          fixture(`cjs-ts-wrappers/${group.dir}/${name}`),
          { [`no-mistakes/${group.rule}`]: configured },
          name,
        );
        const actual = diagnostics.filter((diagnostic) => basename(diagnostic.filename) === name);
        assert.deepEqual(
          actual.map((diagnostic) => diagnostic.labels[0].span.line).sort((a, b) => a - b),
          expected.map((finding) => finding.line).sort((a, b) => a - b),
          `${group.dir}/${name}`,
        );
      }
    }
  }, 30_000);
});
