import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import tsParser from "@typescript-eslint/parser";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { fixture, lint, plugin, __dirname } from "./helpers.mjs";

const rule = "vitest-timeout-cap";
const finding = { unknownValues: "finding" };
const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};
const tsLanguage = { parser: tsParser };
const findings = (name, options) =>
  lint(
    fixture(`vitest-timeout-cap/${name}.ts`),
    { [`no-mistakes/${rule}`]: ["error", options || {}] },
    "fixture.test.ts",
  );

function capCase(name, options = {}) {
  return {
    code: fixture(`vitest-timeout-cap/${name}.ts`),
    filename: "fixture.test.ts",
    languageOptions: tsLanguage,
    options: [options],
  };
}

const suite = {
  valid: [
    // Whole-member scalar writes stay known when unknown values are ignored.
    capCase("copy-coercion-wrappers"),
    // Scoped setter aliases that are not recognized carriers.
    capCase("alias-controls", finding),
    // Opaque carriers are silent until the finding policy is explicit.
    capCase("opaque-carriers"),
    // Plain standalone config roots are out of scope until configRoot is set.
    capCase("standalone-root"),
    // Project entries at defaultMax 60000.
    capCase("config", { defaultMax: 60000 }),
    // Unknown values are ignored unless requested.
    capCase("unknown"),
    // Unrelated imports, lookalike data, shadowed aliases and unrecognized exports.
    capCase("lookalikes", finding),
    capCase("shadows", finding),
    // overrideMax 240000 accepts the saved overrides.
    capCase("overrides", { overrideMax: 240000 }),
    // File suppression.
    capCase("suppression-file"),
    // Unknown spreads are silent until the finding policy is explicit.
    capCase("unknown-spreads"),
  ],
  invalid: [
    // 8 unknown, 1 timeout. Callable timeout sinks do not need an opaque consumer.
    capCase("callable-timeout-sinks", finding),
    // 1 timeout.
    capCase("callable-timeout-sinks"),
    // 4 unknown, 1 timeout. A destructured member is not probed against its unprojected initializer.
    capCase("destructured-primitive-reads", finding),
    // 1 timeout.
    capCase("destructured-primitive-reads"),
    // 7 unknown, 2 timeout. Callable property references stay unknown.
    capCase("callable-primitive-reads", finding),
    // A missing own field (50) remains unknown. A later direct deletion no
    // longer invalidates the earlier own-data assignment (20).
    // 42 unknown.
    capCase("copy-coercion", finding),
    // 9 unknown.
    capCase("copy-coercion-wrappers", finding),
    // 38 unknown, 1 timeout. Supported consumers during copied-scalar proof.
    capCase("copy-safe-consumers", finding),
    // 1 timeout.
    capCase("copy-safe-consumers"),
    // 1 timeout, 5 unknown. Primitive property reads stay known.
    capCase("primitive-reads", finding),
    // 2 timeout, 1 invalid, 4 unknown.
    capCase("invocation-safety", finding),
    // 1 timeout, 2 unknown, 1 invalid. Partially bound parameters are not unchanged.
    capCase("bound-callbacks", finding),
    // 3 timeout, 2 invalid, 2 unknown.
    capCase("named-and-invocations", finding),
    // 4 timeout, 3 invalid. Scoped immutable, destructured, bound and computed setter aliases.
    capCase("setter-aliases"),
    // 9 unknown.
    capCase("opaque-carriers", finding),
    // 1 timeout, 1 invalid. Explicit configRoot admits a standalone config.
    capCase("standalone-root", { configRoot: true, defaultMax: 30000 }),
    // 4 timeout, 3 invalid, 1 unknown.
    capCase("chain-and-context", finding),
    // 3 unknown. Partial applications and unresolved merged carriers.
    capCase("partial-and-merge", finding),
    // 10 timeout, 1 invalid. Numeric arithmetic is capped without executing calls.
    capCase("arithmetic"),
    // 10 timeout, 1 invalid, 5 unknown.
    capCase("arithmetic", finding),
    // 13 timeout.
    capCase("overrides"),
    // 4 timeout. Project entries and config defaults against defaultMax.
    capCase("config"),
    // 7 invalid, 3 timeout. Zero and negative timeouts, plus the positive cap boundary.
    capCase("positive-boundaries", { defaultMax: 30_000 }),
    // 1 timeout. Resolved infinity.
    capCase("number-shapes"),
    // 2 unknown, 1 timeout. Opaque NaN stays under the unknown-value policy.
    capCase("number-shapes", finding),
    // 1 timeout. Effective deep merged precedence.
    capCase("merged"),
    // 1 timeout. Spread precedence keeps the project source location.
    capCase("spreads"),
    // 7 unknown.
    capCase("unknown", finding),
    // 2 timeout. Exact custom suppression directives.
    capCase("suppression"),
    // 4 timeout. Config callbacks, array spreads and merged project-array concatenation.
    capCase("config-callbacks"),
    // 4 unknown. One message names the projects carrier.
    capCase("unknown-spreads", finding),
    // 1 invalid, 3 timeout.
    capCase("value-edges"),
    // 1 invalid, 3 timeout, 5 unknown.
    capCase("value-edges", finding),
    // 5 timeout. Nested config roots and namespace hooks, without overridden fragments.
    capCase("merge-edges"),
    // 5 unknown, 5 timeout.
    capCase("merge-edges", finding),
    // 3 timeout. Oversized defaults survive when mergeConfig ignores null overrides.
    capCase("merge-null"),
    // 3 timeout, 3 unknown. Null spreads stay inert.
    capCase("merge-null", finding),
  ],
};

describe(rule, () => {
  it("snapshots vitest-timeout-cap diagnostics", async () => {
    await expect(
      snapshotDocument({
        [rule]: snapshotSuite(rule, languageOptions, suite),
      }),
    ).toMatchFileSnapshot("snapshots/vitest-timeout-cap.snap");
  }, 30_000);

  it("exports an opt-in rule and rejects invalid configuration", () => {
    assert.ok(!plugin.configs.strict.rules[`no-mistakes/${rule}`]);
    assert.ok(!plugin.configs.recommended.rules[`no-mistakes/${rule}`]);
    for (const option of [
      { defaultMax: 0 },
      { defaultMax: Infinity },
      { overrideMax: NaN },
      { overrideMax: -1 },
      { unknownValues: "warn" },
    ])
      assert.throws(() => findings("overrides", option));
  });

  it("loads the same timeout and suppression semantics through Oxlint", () => {
    const root = resolve(
      __dirname,
      "../../../test-cases/eslint-snippets/fixture/vitest-timeout-cap",
    );
    const run = (name) =>
      spawnSync(
        process.execPath,
        [
          resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
          "--config",
          resolve(root, ".oxlintrc.json"),
          "--format",
          "json",
          resolve(root, `${name}.ts`),
        ],
        { encoding: "utf8" },
      );
    assert.equal(
      JSON.parse(run("overrides").stdout).diagnostics.filter((item) => item.code?.includes(rule))
        .length,
      13,
    );
    assert.equal(
      JSON.parse(run("arithmetic").stdout).diagnostics.filter((item) => item.code?.includes(rule))
        .length,
      11,
    );
    assert.deepEqual(
      JSON.parse(run("config-return-values").stdout)
        .diagnostics.filter((item) => item.code?.includes(rule))
        .map((item) => item.labels[0].span.line)
        .toSorted((left, right) => left - right),
      [9, 13],
    );
    const boundaries = JSON.parse(run("positive-boundaries").stdout).diagnostics.filter((item) =>
      item.code?.includes(rule),
    );
    assert.equal(boundaries.filter((item) => item.message.includes("must be positive")).length, 7);
    assert.ok(!boundaries.some((item) => item.labels[0].span.line === 22));
    assert.deepEqual(
      JSON.parse(run("number-shapes").stdout)
        .diagnostics.filter((item) => item.code?.includes(rule))
        .map((item) => item.labels[0].span.line),
      [7],
    );
    assert.ok(
      !JSON.parse(run("suppression-file").stdout).diagnostics.some((item) =>
        item.code?.includes(rule),
      ),
    );
  });
});
