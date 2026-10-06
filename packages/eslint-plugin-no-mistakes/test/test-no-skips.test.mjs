import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { fixture, lint, messages, plugin, __dirname } from "./helpers.mjs";
const rule = "test-no-skips";
const code = (name) => fixture(`test-no-skips/${name}.ts`);
describe(rule, () => {
  it("reports each modifier and table chain once, with bound contexts only", () => {
    const findings = lint(code("modifiers"), { [`no-mistakes/${rule}`]: "error" }, "suite.test.ts");
    assert.deepEqual(
      findings.map((item) => item.line),
      [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 15],
    );
    assert.ok(findings.every((item) => item.messageId === "modifier"));
  });
  it("ignores test data, unrelated members, and shadowed bindings", () => {
    assert.deepEqual(messages(code("lookalikes"), rule, undefined, "suite.test.ts"), []);
  });
  it("handles Playwright aliases and namespaces without mistaking fixtures for contexts", () => {
    assert.equal(messages(code("playwright"), rule, undefined, "suite.spec.ts").length, 6);
  });
  it("handles configured globals and Vitest namespace/quoted members", () => {
    assert.equal(messages(code("globals"), rule, undefined, "suite.test.ts").length, 4);
    assert.equal(messages(code("namespace"), rule, undefined, "suite.test.ts").length, 4);
  });
  it("allows named modifiers including runtime context skip", () => {
    assert.equal(
      messages(code("modifiers"), rule, { allow: ["todo"] }, "suite.test.ts").length,
      11,
    );
    assert.deepEqual(
      messages(
        code("modifiers"),
        rule,
        { allow: ["skip", "skipIf", "runIf", "only", "todo", "fixme"] },
        "suite.test.ts",
      ),
      [],
    );
  });
  it("exports an opt-in rule without widening presets", () => {
    assert.ok(plugin.rules[rule]);
    assert.ok(!plugin.configs.recommended.rules[`no-mistakes/${rule}`]);
    assert.ok(!plugin.configs.strict.rules[`no-mistakes/${rule}`]);
    assert.deepEqual(plugin.rules[rule].meta.schema[0].properties.allow.items.enum, [
      "skip",
      "skipIf",
      "runIf",
      "only",
      "todo",
      "fixme",
    ]);
  });
});

describe("test-no-skips suppression and binding edges", () => {
  it("uses actual file/line/next-line comments with exact rule identity", () => {
    const findings = lint(
      code("suppression"),
      { [`no-mistakes/${rule}`]: "error" },
      "suite.test.ts",
    );
    assert.deepEqual(
      findings.map((item) => item.line),
      [5, 7, 9],
    );
    assert.deepEqual(messages(code("suppression-file"), rule, undefined, "suite.test.ts"), []);
  });
  it("ignores non-test imports and data/context lookalikes while matching static quoted members", () => {
    const findings = lint(
      code("edge-bindings"),
      { [`no-mistakes/${rule}`]: "error" },
      "suite.test.ts",
    );
    assert.deepEqual(
      findings.map((item) => item.line),
      [17, 18, 19, 21],
    );
  });
});

it("loads test-no-skips in Oxlint with the same import and suppression semantics", () => {
  const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture/test-no-skips");
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
  const result = run("playwright");
  assert.notEqual(result.status, 0);
  const diagnostics = JSON.parse(result.stdout).diagnostics.filter((item) =>
    item.code?.includes("test-no-skips"),
  );
  assert.equal(diagnostics.length, 6);
  const suppressed = run("suppression-file");
  assert.ok(
    !JSON.parse(suppressed.stdout).diagnostics.some((item) => item.code?.includes("test-no-skips")),
  );
});
