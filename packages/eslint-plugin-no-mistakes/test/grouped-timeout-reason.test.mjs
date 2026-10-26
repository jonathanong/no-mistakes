import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { it } from "vitest";
import { Linter } from "eslint";
import tsParser from "@typescript-eslint/parser";
import { fixture, lint, require, __dirname } from "./helpers.mjs";

const rule = "vitest-timeout-cap";
const name = `${rule}/reason-adjacency.ts`;
const findings = () => lint(fixture(name), { [`no-mistakes/${rule}`]: "error" }, "fixture.test.ts");

it("looks through grouping and ordinary comments without crossing blank lines or other code", () => {
  assert.deepEqual(
    findings().map(({ line, messageId }) => [line, messageId]),
    [
      [6, "reason"],
      [19, "reason"],
      [21, "reason"],
      [23, "reason"],
      [26, "reason"],
      [30, "reason"],
      [47, "reason"],
      [50, "reason"],
      [53, "reason"],
    ],
  );
});

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
