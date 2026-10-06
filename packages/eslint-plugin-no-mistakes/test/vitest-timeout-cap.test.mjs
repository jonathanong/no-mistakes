import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { fixture, lint, plugin, __dirname } from "./helpers.mjs";
const rule = "vitest-timeout-cap";
const code = (name) => fixture(`vitest-timeout-cap/${name}.ts`);
const findings = (name, options) =>
  lint(code(name), { [`no-mistakes/${rule}`]: ["error", options || {}] }, "fixture.test.ts");
describe(rule, () => {
  it("caps all test, hook, suite, table, and runtime forms with module constants", () => {
    assert.deepEqual(
      findings("overrides").map((item) => item.line),
      [5, 6, 7, 8, 9, 10, 11, 13, 14, 15, 16, 17, 17],
    );
    assert.ok(findings("overrides").every((item) => item.messageId === "timeout"));
  });
  it("reports project entries and config defaults against defaultMax", () => {
    assert.deepEqual(
      findings("config").map((item) => item.line),
      [6, 9, 10, 11],
    );
    assert.deepEqual(findings("config", { defaultMax: 60000 }), []);
  });
  it("honors effective deep merged/spread precedence and retains project source locations", () => {
    assert.deepEqual(
      findings("merged").map((item) => item.line),
      [3],
    );
    assert.deepEqual(
      findings("spreads").map((item) => item.line),
      [5],
    );
  });
  it("reports unknown values only when requested and retains uncertain known merge keys", () => {
    assert.deepEqual(findings("unknown"), []);
    assert.deepEqual(
      findings("unknown", { unknownValues: "finding" }).map((item) => item.line),
      [6, 7, 9, 10, 11, 11],
    );
  });
  it("ignores unrelated imports, lookalike data, shadowed aliases and unrecognized exports", () => {
    assert.deepEqual(findings("lookalikes", { unknownValues: "finding" }), []);
    assert.deepEqual(findings("shadows", { unknownValues: "finding" }), []);
  });
  it("uses explicit configurable caps and exact custom suppression directives", () => {
    assert.deepEqual(findings("overrides", { overrideMax: 240000 }), []);
    assert.deepEqual(
      findings("suppression").map((item) => item.line),
      [6, 8],
    );
    assert.deepEqual(findings("suppression-file"), []);
  });
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
    assert.ok(
      !JSON.parse(run("suppression-file").stdout).diagnostics.some((item) =>
        item.code?.includes(rule),
      ),
    );
  });
});

describe("vitest-timeout-cap effective config and source boundaries", () => {
  it("recovers config callbacks, array spreads and merged project-array concatenation", () => {
    assert.deepEqual(
      findings("config-callbacks").map((item) => item.line),
      [3, 4, 5, 7],
    );
  });
  it("preserves known field uncertainty after unknown spreads while explicit values restore certainty", () => {
    assert.deepEqual(findings("unknown-spreads"), []);
    assert.deepEqual(
      findings("unknown-spreads", { unknownValues: "finding" }).map((item) => item.messageId),
      ["unknown", "unknown"],
    );
  });
  it("handles signed constants, getters, local unknowns and suppression at module-options usage", () => {
    assert.deepEqual(
      findings("value-edges").map((item) => item.line),
      [12, 18, 22],
    );
    assert.deepEqual(
      findings("value-edges", { unknownValues: "finding" }).map((item) => item.line),
      [12, 14, 15, 18, 19, 22, 22],
    );
  });
  it("handles nested config roots and namespace/global hooks without reporting overridden fragments", () => {
    assert.deepEqual(
      findings("merge-edges").map((item) => item.line),
      [5, 11, 12, 13, 14],
    );
    assert.ok(
      findings("merge-edges", { unknownValues: "finding" }).some(
        (item) => item.messageId === "unknown",
      ),
    );
  });
});

it("preserves oversized defaults when mergeConfig ignores null overrides, and null spreads are inert", () => {
  assert.deepEqual(
    findings("merge-null").map((item) => item.line),
    [2, 6],
  );
  assert.deepEqual(
    findings("merge-null", { unknownValues: "finding" }).map((item) => item.line),
    [2, 4, 6, 8],
  );
});
