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
  it("preserves runtime receivers, literal invocation callbacks and opaque argument safety", () => {
    assert.deepEqual(
      findings("invocation-safety", { unknownValues: "finding" }).map(({ line, messageId }) => [
        line,
        messageId,
      ]),
      [
        [2, "timeout"],
        [3, "invalid"],
        [4, "unknown"],
        [5, "unknown"],
        [6, "unknown"],
        [7, "timeout"],
        [13, "unknown"],
      ],
    );
  });
  it("preserves bound callbacks without pretending partially bound parameters are unchanged", () => {
    assert.deepEqual(
      findings("bound-callbacks", { unknownValues: "finding" }).map(({ line, messageId }) => [
        line,
        messageId,
      ]),
      [
        [2, "timeout"],
        [6, "unknown"],
        [7, "unknown"],
        [8, "invalid"],
      ],
    );
  });
  it("retains named TestContext and literal/opaque call/apply hook provenance", () => {
    assert.deepEqual(
      findings("named-and-invocations", { unknownValues: "finding" }).map(({ line, messageId }) => [
        line,
        messageId,
      ]),
      [
        [2, "timeout"],
        [5, "invalid"],
        [7, "timeout"],
        [10, "timeout"],
        [11, "invalid"],
        [13, "unknown"],
        [14, "unknown"],
      ],
    );
  });
  it("tracks scoped immutable, destructured, bound and computed setter aliases", () => {
    assert.deepEqual(
      findings("setter-aliases").map((item) => item.line),
      [4, 6, 8, 10, 12, 14, 16],
    );
    assert.deepEqual(findings("alias-controls", { unknownValues: "finding" }), []);
  });
  it("rejects opaque recognized carriers only under explicit finding policy", () => {
    assert.deepEqual(findings("opaque-carriers"), []);
    assert.deepEqual(
      findings("opaque-carriers", { unknownValues: "finding" }).map((item) => ({
        line: item.line,
        messageId: item.messageId,
      })),
      Array.from({ length: 9 }, (_, index) => ({ line: index + 4, messageId: "unknown" })),
    );
  });
  it("checks plain standalone config roots only in explicitly admitted config scope", () => {
    assert.deepEqual(findings("standalone-root"), []);
    assert.deepEqual(
      findings("standalone-root", { configRoot: true, defaultMax: 30000 }).map((item) => ({
        line: item.line,
        messageId: item.messageId,
      })),
      [
        { line: 2, messageId: "timeout" },
        { line: 2, messageId: "invalid" },
      ],
    );
  });
  it("checks current returned registrars, suite chains, static names and real completion context", () => {
    assert.deepEqual(
      findings("chain-and-context", { unknownValues: "finding" }).map(({ line, messageId }) => [
        line,
        messageId,
      ]),
      [
        [3, "timeout"],
        [4, "invalid"],
        [5, "invalid"],
        [6, "timeout"],
        [7, "invalid"],
        [8, "timeout"],
        [11, "timeout"],
        [12, "unknown"],
      ],
    );
  });
  it("keeps partial applications and unresolved merged/remaining carriers explicit", () => {
    assert.deepEqual(
      findings("partial-and-merge", { unknownValues: "finding" }).map(({ line, messageId }) => [
        line,
        messageId,
      ]),
      [
        [4, "unknown"],
        [5, "unknown"],
        [7, "unknown"],
      ],
    );
  });
  it("caps numeric arithmetic without executing calls or coercing strings", () => {
    assert.deepEqual(
      findings("arithmetic").map(({ line, messageId }) => ({ line, messageId })),
      [
        ...[3, 4, 6, 7, 8, 9].map((line) => ({ line, messageId: "timeout" })),
        { line: 10, messageId: "invalid" },
        { line: 11, messageId: "timeout" },
        { line: 15, messageId: "timeout" },
        { line: 16, messageId: "timeout" },
        { line: 17, messageId: "timeout" },
      ],
    );
    assert.deepEqual(
      findings("arithmetic", { unknownValues: "finding" })
        .filter((item) => item.messageId === "unknown")
        .map((item) => item.line),
      [12, 13, 14, 19, 20],
    );
  });
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
  it("rejects zero and negative timeouts and accepts the positive cap boundary", () => {
    assert.deepEqual(
      findings("positive-boundaries", { defaultMax: 30_000 }).map(({ line, messageId }) => ({
        line,
        messageId,
      })),
      [
        { line: 12, messageId: "invalid" },
        { line: 15, messageId: "invalid" },
        { line: 16, messageId: "timeout" },
        { line: 20, messageId: "invalid" },
        { line: 21, messageId: "invalid" },
        { line: 23, messageId: "timeout" },
        { line: 24, messageId: "invalid" },
        { line: 25, messageId: "invalid" },
        { line: 26, messageId: "invalid" },
        { line: 26, messageId: "timeout" },
      ],
    );
  });
  it("caps resolved infinity while keeping opaque NaN under the unknown-value policy", () => {
    assert.deepEqual(
      findings("number-shapes").map(({ line, messageId }) => ({ line, messageId })),
      [{ line: 7, messageId: "timeout" }],
    );
    assert.deepEqual(
      findings("number-shapes", { unknownValues: "finding" }).map(({ line, messageId }) => ({
        line,
        messageId,
      })),
      [
        { line: 6, messageId: "unknown" },
        { line: 7, messageId: "timeout" },
        { line: 8, messageId: "unknown" },
      ],
    );
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
      [6, 7, 9, 10, 11, 11, 11],
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
    assert.equal(
      JSON.parse(run("arithmetic").stdout).diagnostics.filter((item) => item.code?.includes(rule))
        .length,
      11,
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
      ["unknown", "unknown", "unknown"],
    );
  });
  it("handles signed constants, getters, local unknowns and suppression at module-options usage", () => {
    assert.deepEqual(
      findings("value-edges").map((item) => item.line),
      [11, 12, 18, 22],
    );
    assert.deepEqual(
      findings("value-edges", { unknownValues: "finding" }).map((item) => item.line),
      [11, 12, 14, 15, 18, 19, 22, 22, 22],
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
    [2, 6, 8],
  );
  assert.deepEqual(
    findings("merge-null", { unknownValues: "finding" }).map((item) => item.line),
    [2, 4, 4, 4, 6, 8],
  );
});
