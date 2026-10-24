import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { RuleTester } from "eslint";
import parser from "@typescript-eslint/parser";
import { describe, it } from "vitest";
import rule from "../src/rules/test-no-shared-state.js";
import { __dirname, fixture, lint } from "./helpers.mjs";

const directory = "test-no-shared-state";
const ruleName = "test-no-shared-state";
const cases = [
  "forwarded",
  "forwarded-controls",
  "forwarded-suppressed",
  "forwarded-file",
  "forwarded-shadowed-serial",
  "forwarded-serial",
];
const code = (name) => fixture(`${directory}/${name}.test.ts`);
const lines = (name) =>
  code(name)
    .split("\n")
    .flatMap((line, index) => (line.includes("// report") ? [index + 1] : []));

RuleTester.describe = describe;
RuleTester.it = it;
const tester = new RuleTester({ languageOptions: { parser } });
tester.run(ruleName, rule, {
  valid: cases
    .filter((name) => lines(name).length === 0)
    .map((name) => ({ code: code(name), filename: `${name}.test.ts` })),
  invalid: cases
    .filter((name) => lines(name).length > 0)
    .map((name) => ({
      code: code(name),
      filename: `${name}.test.ts`,
      errors: lines(name).map((line) => ({ messageId: "shared", line })),
    })),
});

describe("shared-state forwarded registrations", () => {
  for (const name of cases) {
    it(`preserves plugin-loader diagnostics for ${name}`, () => {
      const findings = lint(
        code(name),
        { [`no-mistakes/${ruleName}`]: "error" },
        `${name}.test.ts`,
      );
      assert.deepEqual(
        findings.map((finding) => [finding.messageId, finding.line]),
        lines(name).map((line) => ["shared", line]),
      );
    });
  }
  it("matches real Oxlint on saved forwarding fixtures", () => {
    const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", directory);
    for (const name of cases) {
      const result = spawnSync(
        process.execPath,
        [
          resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
          "--config",
          resolve(root, ".oxlintrc-forwarded.json"),
          "--format",
          "json",
          resolve(root, `${name}.test.ts`),
        ],
        { encoding: "utf8" },
      );
      assert.equal(result.status, lines(name).length ? 1 : 0, result.stderr);
      const diagnostics = JSON.parse(result.stdout).diagnostics.filter((finding) =>
        finding.code.includes(ruleName),
      );
      assert.deepEqual(
        diagnostics.map((finding) => finding.labels[0].span.line).sort((a, b) => a - b),
        lines(name),
      );
    }
  });
});
