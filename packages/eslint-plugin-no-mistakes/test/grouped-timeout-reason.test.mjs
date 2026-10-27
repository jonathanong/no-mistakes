import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { expect, it } from "vitest";
import { Linter } from "eslint";
import tsParser from "@typescript-eslint/parser";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { fixture, lint, require, __dirname } from "./helpers.mjs";

const rule = "vitest-timeout-cap";
const name = `${rule}/reason-adjacency.ts`;
const findings = () => lint(fixture(name), { [`no-mistakes/${rule}`]: "error" }, "fixture.test.ts");
const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};

// Grouping and ordinary comments are visible. Blank lines and other code are not.
// 9 reason. Severity is "error" with no options object.
const suite = {
  invalid: [
    {
      code: fixture(name),
      filename: "fixture.test.ts",
      languageOptions: { parser: tsParser },
    },
  ],
};

it("snapshots grouped-timeout-reason diagnostics", async () => {
  await expect(
    snapshotDocument({
      [rule]: snapshotSuite(rule, languageOptions, suite),
    }),
  ).toMatchFileSnapshot("snapshots/grouped-timeout-reason.snap");
}, 30_000);

it("keeps ESLint and Oxlint grouped annotation boundaries identical", () => {
  const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", rule);
  const result = spawnSync(
    process.execPath,
    [
      resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
      "--config",
      resolve(root, ".oxlintrc.json"),
      "--format",
      "json",
      resolve(root, "reason-adjacency.ts"),
    ],
    { encoding: "utf8" },
  );
  assert.equal(result.status, 1, result.stderr);
  assert.deepEqual(
    JSON.parse(result.stdout)
      .diagnostics.filter((item) => item.code?.includes(rule))
      .map((item) => [item.labels[0].span.line, item.message])
      .sort((a, b) => a[0] - b[0]),
    findings().map(({ line, message }) => [line, message]),
  );
});

it("keeps the generic adjacency API inside grouping rather than required syntax", () => {
  const { adjacentAnnotations } = require("../src/adjacent-annotation.js");
  const found = [];
  const messages = new Linter().verify(
    fixture(name),
    [
      {
        files: ["**/*.ts"],
        languageOptions: { parser: tsParser },
        plugins: {
          probe: {
            rules: {
              adjacency: {
                create(context) {
                  return {
                    Program(node) {
                      assert.deepEqual(
                        adjacentAnnotations(context.sourceCode, node, "adjacency:"),
                        [],
                      );
                    },
                    Literal(node) {
                      found.push(...adjacentAnnotations(context.sourceCode, node, "adjacency:"));
                    },
                  };
                },
              },
            },
          },
        },
        rules: { "probe/adjacency": "error" },
      },
    ],
    { filename: "fixture.ts" },
  );
  assert.deepEqual(messages, []);
  assert.deepEqual(found, ["grouped value"]);
});
