import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { fixture, lint, messages, plugin, __dirname } from "./helpers.mjs";
const rule = "test-no-skips";
const code = (name) => fixture(`test-no-skips/${name}.ts`);
describe(rule, () => {
  it("resolves immutable lexical keys through runner aliases and extended APIs", () => {
    const findings = lint(
      code("immutable-keys"),
      { [`no-mistakes/${rule}`]: "error" },
      "suite.test.ts",
    );
    assert.deepEqual(
      findings.map(({ line, messageId }) => [line, messageId]),
      [
        [7, "modifier"],
        [8, "modifier"],
        [10, "modifier"],
        [12, "modifier"],
        [13, "modifier"],
        [14, "modifier"],
        [15, "modifier"],
        [18, "modifier"],
        [20, "modifier"],
      ],
    );
  });
  it("leaves dynamic, imported, reassigned, cyclic and shadowed keys unresolved", () => {
    assert.deepEqual(
      messages(code("immutable-key-controls"), rule, undefined, "suite.test.ts"),
      [],
    );
  });
  it("reports each modifier and table chain once, with bound contexts only", () => {
    const findings = lint(code("modifiers"), { [`no-mistakes/${rule}`]: "error" }, "suite.test.ts");
    assert.deepEqual(
      findings.map((item) => item.line),
      [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 15],
    );
    assert.ok(findings.every((item) => item.messageId === "modifier"));
  });
  it("follows Vitest callback skip bindings without treating table data as context", () => {
    const findings = lint(
      code("context-bindings"),
      { [`no-mistakes/${rule}`]: "error" },
      "suite.test.ts",
    );
    assert.deepEqual(
      findings.map(({ line, messageId }) => [line, messageId]),
      [4, 5, 6, 7, 10, 15, 19, 21, 24].map((line) => [line, "modifier"]),
    );
    assert.deepEqual(
      messages(code("context-bindings"), rule, { allow: ["skip"] }, "suite.test.ts"),
      [],
    );
  });
  it("ignores test data, unrelated members, and shadowed bindings", () => {
    assert.deepEqual(messages(code("lookalikes"), rule, undefined, "suite.test.ts"), []);
  });
  it("handles Playwright aliases and namespaces without mistaking fixtures for contexts", () => {
    assert.equal(messages(code("playwright"), rule, undefined, "suite.spec.ts").length, 6);
  });
  it("follows same-file Playwright extend and chained extend", () => {
    const findings = lint(
      code("playwright-extend"),
      { [`no-mistakes/${rule}`]: "error" },
      "suite.spec.ts",
    );
    assert.deepEqual(
      findings.map(({ line, messageId }) => [line, messageId]),
      [
        [14, "modifier"],
        [15, "modifier"],
        [16, "modifier"],
        [17, "annotation"],
        [18, "modifier"],
      ],
    );
  });
  it("admits Playwright test exports only from configured testModules", () => {
    assert.deepEqual(
      messages(code("playwright-test-modules"), rule, undefined, "suite.spec.ts"),
      [],
    );
    const findings = lint(
      code("playwright-test-modules"),
      {
        [`no-mistakes/${rule}`]: ["error", { testModules: ["./fixtures", "@app/test"] }],
      },
      "suite.spec.ts",
    );
    assert.deepEqual(
      findings.map(({ line, messageId }) => [line, messageId]),
      [
        [8, "modifier"],
        [9, "annotation"],
        [10, "modifier"],
        [11, "modifier"],
        [13, "modifier"],
      ],
    );
  });
  it("admits only test from configured namespaces across module forms", () => {
    const findings = lint(
      code("configured-namespace"),
      { [`no-mistakes/${rule}`]: ["error", { testModules: ["./fixtures"] }] },
      "suite.spec.ts",
    );
    assert.deepEqual(
      findings.map(({ line, messageId }) => [line, messageId]),
      [
        [12, "modifier"],
        [13, "modifier"],
        [14, "annotation"],
        [16, "modifier"],
        [17, "modifier"],
      ],
    );
  });
  it("reports Playwright declaration skip/fixme annotations through const aliases", () => {
    const findings = lint(
      code("annotations"),
      { [`no-mistakes/${rule}`]: "error" },
      "suite.spec.ts",
    );
    assert.deepEqual(
      findings.map(({ line, messageId }) => [line, messageId]),
      [
        [18, "annotation"],
        [19, "annotation"],
        [20, "annotation"],
        [21, "annotation"],
        [22, "annotation"],
        [23, "annotation"],
        [24, "annotation"],
        [25, "annotation"],
        [26, "modifier"],
        [26, "annotation"],
        [29, "annotation"],
        [36, "annotation"],
        [41, "annotation"],
        [44, "modifier"],
        [51, "modifier"],
        [60, "annotation"],
        [65, "annotation"],
        [70, "annotation"],
      ],
    );
    assert.equal(
      findings.find((item) => item.line === 29).message,
      "Playwright 'skip or fixme' declaration annotation skips this test or suite. Remove it or explicitly allow the annotation.",
    );
    assert.equal(
      findings.find((item) => item.line === 36).message,
      "Playwright 'fixme or skip' declaration annotation skips this test or suite. Remove it or explicitly allow the annotation.",
    );
    assert.equal(
      findings.find((item) => item.line === 41).message,
      "Playwright 'skip' declaration annotation skips this test or suite. Remove it or explicitly allow the annotation.",
    );
    assert.equal(
      findings.find((item) => item.line === 60).message,
      "Playwright 'skip' declaration annotation skips this test or suite. Remove it or explicitly allow the annotation.",
    );
    assert.equal(
      findings.find((item) => item.line === 65).message,
      "Playwright 'fixme' declaration annotation skips this test or suite. Remove it or explicitly allow the annotation.",
    );
    assert.equal(
      findings.find((item) => item.line === 70).message,
      "Playwright 'skip or fixme' declaration annotation skips this test or suite. Remove it or explicitly allow the annotation.",
    );
    assert.deepEqual(
      messages(
        code("annotations"),
        rule,
        { allow: ["skip", "skipIf", "runIf", "only", "todo", "fixme"] },
        "suite.spec.ts",
      ),
      [],
    );
    const allowSkip = lint(
      code("allow-annotations"),
      { [`no-mistakes/${rule}`]: ["error", { allow: ["skip"] }] },
      "suite.spec.ts",
    );
    const annotationMessage = (name) =>
      `Playwright '${name}' declaration annotation skips this test or suite. Remove it or explicitly allow the annotation.`;
    assert.deepEqual(
      allowSkip.map(({ line, message }) => [line, message]),
      [
        [7, annotationMessage("fixme")],
        [11, annotationMessage("fixme")],
      ],
    );
    const allowFixme = lint(
      code("allow-annotations"),
      { [`no-mistakes/${rule}`]: ["error", { allow: ["fixme"] }] },
      "suite.spec.ts",
    );
    assert.deepEqual(
      allowFixme.map(({ line, message }) => [line, message]),
      [
        [7, annotationMessage("skip")],
        [11, annotationMessage("skip")],
      ],
    );
    assert.deepEqual(
      messages(code("allow-annotations"), rule, { allow: ["skip", "fixme"] }, "suite.spec.ts"),
      [],
    );
  });
  it("handles configured globals and Vitest namespace/quoted members", () => {
    assert.equal(messages(code("globals"), rule, undefined, "suite.test.ts").length, 4);
    assert.equal(messages(code("namespace"), rule, undefined, "suite.test.ts").length, 4);
  });
  it("follows destructuring defaults and nested object patterns", () => {
    const findings = lint(
      code("pattern-bindings"),
      { [`no-mistakes/${rule}`]: "error" },
      "suite.test.ts",
    );
    assert.deepEqual(
      findings.map((item) => item.line),
      [7, 10, 13, 16, 19, 22, 39],
    );
  });
  it("handles statically resolved CommonJS and dynamic module bindings", () => {
    const findings = lint(
      code("module-bindings"),
      { [`no-mistakes/${rule}`]: "error" },
      "suite.test.ts",
    );
    assert.equal(findings.length, 6);
    assert.deepEqual(
      findings.map((item) => item.line),
      [2, 3, 5, 8, 11, 14],
    );
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
    assert.deepEqual(plugin.rules[rule].meta.schema[0].properties.testModules, {
      type: "array",
      items: { type: "string" },
      uniqueItems: true,
    });
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
  it("follows a Playwright default import, a namespace destructure, and static member keys", () => {
    assert.deepEqual(messages(code("recall"), rule, undefined, "suite.test.ts"), [
      "modifier",
      "modifier",
      "modifier",
      "modifier",
    ]);
  });
  it("reads a TypeScript import-equals require without dropping a named test binding", () => {
    assert.deepEqual(messages(code("import-equals-non-test"), rule, undefined, "service.ts"), []);
    const findings = lint(
      code("import-equals-skip"),
      { [`no-mistakes/${rule}`]: "error" },
      "suite.test.ts",
    );
    assert.deepEqual(
      findings.map(({ line, messageId }) => [line, messageId]),
      [
        [4, "modifier"],
        [5, "modifier"],
      ],
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
  const bound = run("context-bindings");
  assert.notEqual(bound.status, 0);
  assert.deepEqual(
    JSON.parse(bound.stdout)
      .diagnostics.filter((item) => item.code?.includes("test-no-skips"))
      .map((item) => item.labels[0].span.line),
    [4, 5, 6, 7, 10, 15, 19, 21, 24],
  );
  const report = (result) => {
    const parsed = JSON.parse(result.stdout);
    delete parsed.start_time;
    return parsed;
  };
  const ruleLines = (parsed) =>
    parsed.diagnostics
      .filter((item) => item.code?.includes("test-no-skips"))
      .map((item) => item.labels[0].span.line);
  const nonTest = [run("import-equals-non-test"), run("import-equals-non-test")];
  assert.deepEqual(
    nonTest.map((item) => item.status),
    [0, 0],
  );
  const nonTestReport = report(nonTest[0]);
  assert.deepEqual(nonTestReport, report(nonTest[1]));
  assert.ok(!nonTest[0].stderr.includes("TypeError"));
  assert.deepEqual(ruleLines(nonTestReport), []);
  const skip = [run("import-equals-skip"), run("import-equals-skip")];
  assert.ok(skip.every((item) => item.status !== 0));
  const skipReport = report(skip[0]);
  assert.deepEqual(skipReport, report(skip[1]));
  assert.ok(!skip[0].stderr.includes("TypeError"));
  assert.deepEqual(ruleLines(skipReport), [4, 5]);
});
