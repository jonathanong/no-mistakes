import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import assert from "node:assert/strict";
import { describe, expect, it } from "vitest";
import { fixture, plugin, __dirname } from "./helpers.mjs";
import {
  oxlintFindings,
  pluginFindings,
  snapshotCases,
  snapshotDocument,
} from "./diagnostic-snapshot.mjs";

const rule = "test-no-skips";
const code = (name) => fixture(`test-no-skips/${name}.ts`);
const allowAll = ["skip", "skipIf", "runIf", "only", "todo", "fixme"];
const rows = [
  ["immutable-keys", "immutable-keys", "suite.test.ts"],
  ["immutable-key-controls", "immutable-key-controls", "suite.test.ts"],
  ["modifiers", "modifiers", "suite.test.ts"],
  ["context-bindings", "context-bindings", "suite.test.ts"],
  ["context-bindings allow skip", "context-bindings", "suite.test.ts", { allow: ["skip"] }],
  ["lookalikes", "lookalikes", "suite.test.ts"],
  ["playwright", "playwright", "suite.spec.ts"],
  ["playwright-extend", "playwright-extend", "suite.spec.ts"],
  ["playwright-test-modules", "playwright-test-modules", "suite.spec.ts"],
  [
    "playwright-test-modules testModules",
    "playwright-test-modules",
    "suite.spec.ts",
    { testModules: ["./fixtures", "@app/test"] },
  ],
  [
    "configured-namespace",
    "configured-namespace",
    "suite.spec.ts",
    { testModules: ["./fixtures"] },
  ],
  ["annotations", "annotations", "suite.spec.ts"],
  ["annotations allow modifiers", "annotations", "suite.spec.ts", { allow: [...allowAll] }],
  ["allow-annotations allow skip", "allow-annotations", "suite.spec.ts", { allow: ["skip"] }],
  ["allow-annotations allow fixme", "allow-annotations", "suite.spec.ts", { allow: ["fixme"] }],
  [
    "allow-annotations allow skip and fixme",
    "allow-annotations",
    "suite.spec.ts",
    { allow: ["skip", "fixme"] },
  ],
  ["globals", "globals", "suite.test.ts"],
  ["namespace", "namespace", "suite.test.ts"],
  ["pattern-bindings", "pattern-bindings", "suite.test.ts"],
  ["module-bindings", "module-bindings", "suite.test.ts"],
  ["modifiers allow todo", "modifiers", "suite.test.ts", { allow: ["todo"] }],
  ["modifiers allow all", "modifiers", "suite.test.ts", { allow: [...allowAll] }],
  ["suppression", "suppression", "suite.test.ts"],
  ["suppression-file", "suppression-file", "suite.test.ts"],
  ["edge-bindings", "edge-bindings", "suite.test.ts"],
  ["recall", "recall", "suite.test.ts"],
  ["import-equals-non-test", "import-equals-non-test", "service.ts"],
  ["import-equals-unsupported", "import-equals-unsupported", "service.ts"],
  ["import-equals-skip", "import-equals-skip", "suite.test.ts"],
];

function eslintCases() {
  return rows.map(([name, fixtureName, filename, options]) => ({
    name,
    file: filename,
    rule,
    ...(options === undefined ? {} : { options }),
    findings: pluginFindings(code(fixtureName), rule, options, filename),
  }));
}

describe(rule, () => {
  it("snapshots diagnostics", async () => {
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
    const filtered = (result, name) =>
      oxlintFindings(
        JSON.parse(result.stdout).diagnostics.filter((item) => item.code?.includes(rule)),
        `${name}.ts`,
      );
    const report = (result) => {
      const parsed = JSON.parse(result.stdout);
      delete parsed.start_time;
      return parsed;
    };
    const playwright = run("playwright");
    assert.notEqual(playwright.status, 0);
    const suppressed = run("suppression-file");
    const bound = run("context-bindings");
    assert.notEqual(bound.status, 0);
    const nonTest = [run("import-equals-non-test"), run("import-equals-non-test")];
    assert.deepEqual(
      nonTest.map((item) => item.status),
      [0, 0],
    );
    assert.deepEqual(report(nonTest[0]), report(nonTest[1]));
    assert.ok(!nonTest[0].stderr.includes("TypeError"));
    const unsupported = [run("import-equals-unsupported"), run("import-equals-unsupported")];
    assert.deepEqual(
      unsupported.map((item) => item.status),
      [0, 0],
    );
    assert.deepEqual(report(unsupported[0]), report(unsupported[1]));
    assert.ok(!unsupported[0].stderr.includes("TypeError"));
    const skip = [run("import-equals-skip"), run("import-equals-skip")];
    assert.ok(skip.every((item) => item.status !== 0));
    assert.deepEqual(report(skip[0]), report(skip[1]));
    assert.ok(!skip[0].stderr.includes("TypeError"));
    const oxlintCases = [
      ["playwright", playwright],
      ["suppression-file", suppressed],
      ["context-bindings", bound],
      ["import-equals-non-test", nonTest[0]],
      ["import-equals-unsupported", unsupported[0]],
      ["import-equals-skip", skip[0]],
    ].map(([name, result]) => ({
      name,
      file: `${name}.ts`,
      rule,
      findings: filtered(result, name),
    }));
    await expect(
      snapshotDocument({
        eslint: snapshotCases(eslintCases()),
        oxlint: snapshotCases(oxlintCases),
      }),
    ).toMatchFileSnapshot("snapshots/test-no-skips.snap");
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
