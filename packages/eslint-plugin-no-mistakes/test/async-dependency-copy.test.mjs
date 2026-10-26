import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { Linter } from "eslint";
import tsParser from "@typescript-eslint/parser";
import { it } from "vitest";
import { __dirname, plugin, require } from "./helpers.mjs";

const { adoptNode } = require("../src/rules/async-consumer-property-carrier.js");

const root = resolve(
  __dirname,
  "../../../test-cases/eslint-plugin/async-call-disposition/fixture/proven-consumers/dependency-copy",
);
const caller = resolve(root, "caller.ts");
const expected = [
  72, 78, 84, 90, 96, 102, 108, 144, 150, 156, 162, 168, 174, 180, 186, 192, 198, 204, 210, 216,
  222, 228, 234, 240, 252, 264, 270, 276, 282, 288,
];
const targets = [{ sourceSpecifierPatterns: ["@app/jobs"], calleeNamePatterns: ["enqueueEmail"] }];

function reports(typed) {
  return new Linter({ cwd: dirname(caller) })
    .verify(
      readFileSync(caller, "utf8"),
      {
        files: ["**/*.ts"],
        languageOptions: {
          parser: tsParser,
          parserOptions: typed
            ? { project: resolve(root, "tsconfig.json"), disallowAutomaticSingleRunInference: true }
            : {},
        },
        plugins: { "no-mistakes": plugin },
        rules: { "no-mistakes/async-call-disposition": ["error", { targets }] },
      },
      { filename: caller },
    )
    .map(({ line, messageId }) => [line, messageId]);
}

it.each([false, true])("follows a ?? dependency copy across files (typed=%s)", (typed) => {
  assert.deepEqual(
    reports(typed),
    expected.map((line) => [line, "disposition"]),
  );
});

it("follows the dependency copy under real type-aware Oxlint", () => {
  const result = spawnSync(
    process.execPath,
    [
      resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
      "--type-aware",
      "--deny-warnings",
      "--config",
      resolve(root, "../.oxlintrc-proven.json"),
      "--format",
      "json",
      caller,
    ],
    { encoding: "utf8" },
  );
  assert.equal(result.status, 1, result.stderr);
  assert.deepEqual(
    JSON.parse(result.stdout).diagnostics.map((item) => [item.labels[0].span.line, item.code]),
    expected.map((line) => [line, "no-mistakes(async-call-disposition)"]),
  );
});

// A parsed program always parents an expression, and a const binding cannot be
// reassigned. The copy walker still rejects both, so a future parser cannot turn
// them into adoption.
function copiedConst(variable) {
  const init = { type: "ObjectExpression" };
  const id = { type: "Identifier", name: "dependencies" };
  const declarator = {
    type: "VariableDeclarator",
    id,
    init,
    parent: { type: "VariableDeclaration", kind: "const" },
  };
  init.parent = declarator;
  id.parent = declarator;
  variable.name = "dependencies";
  return adoptNode(
    init,
    "enqueueBulkCrawlReferralLinks",
    { sourceCode: { getScope: () => ({ variables: [variable], upper: null }) } },
    new Map(),
    {
      propertyWritten: () => false,
      proveCall: () => false,
      receiverShape: () => new Map(),
      forward: () => false,
      variable: null,
    },
    new WeakSet(),
  );
}

it("rejects a parentless copy, a merged const, and a write to that const", () => {
  assert.equal(adoptNode({ type: "Identifier" }, null, {}, new Map(), {}, new WeakSet()), false);
  assert.equal(copiedConst({ defs: [{}, {}], references: [] }), false);
  assert.equal(
    copiedConst({
      defs: [{}],
      references: [{ init: false, isWrite: () => true, identifier: { type: "Identifier" } }],
    }),
    false,
  );
});
