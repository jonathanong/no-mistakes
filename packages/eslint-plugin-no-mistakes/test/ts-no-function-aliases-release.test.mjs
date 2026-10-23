import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { __dirname, fixture, lint, messages } from "./helpers.mjs";

const rule = "ts-no-function-aliases";
describe("reachable function alias release regression", () => {
  it("preserves released identifier findings in historical private receiver cases", () => {
    const code = readFileSync(
      resolve(__dirname, "../../../test-cases/eslint-plugin", rule, "fixture/private-wrappers.ts"),
      "utf8",
    );
    assert.deepEqual(
      lint(code, { [`no-mistakes/${rule}`]: "error" }, "fixture.ts").map((item) => item.line),
      [1, 5, 9, 13, 15, 17, 19, 21, 25, 27, 29, 45, 50],
    );
  });
  it("keeps expanded controls while preserving all released identifier findings", () => {
    const diagnostics = lint(
      fixture(`${rule}/release-controls.ts`),
      { [`no-mistakes/${rule}`]: "error" },
      "fixture.ts",
    );
    // These lines match the frozen v0.95.0 oracle, including private targets.
    assert.deepEqual(
      diagnostics.map((item) => item.line),
      [
        18, 19, 20, 21, 30, 31, 32, 34, 36, 37, 40, 41, 42, 44, 45, 46, 50, 54, 56, 59, 61, 65, 66,
        75, 77, 79, 81, 84, 89, 90, 100, 103, 104, 106,
      ],
    );
  });
  it("reports reachable imported and exported pass-throughs including forwarding", () => {
    assert.deepEqual(
      messages(fixture(`${rule}/release-positive.ts`), rule, undefined, "fixture.ts"),
      Array.from({ length: 41 }, () => "alias"),
    );
  });
});

describe("reachable function alias Oxlint parity", () => {
  for (const [name, count] of [
    ["release-controls.ts", 34],
    ["release-jsdoc.mjs", 7],
    ["release-positive.ts", 41],
    ["release-default-const.ts", 2],
    ["legacy-compatibility.ts", 14],
    ["legacy-instantiation.ts", 1],
    ["expanded-contracts.ts", 11],
    ["private-recall.ts", 2],
    ["suppress-next-line.ts", 1],
    ["suppress-line.ts", 0],
    ["suppress-file.ts", 0],
  ]) {
    it(`preserves ${name} with the actual plugin loader`, () => {
      const cwd = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", rule);
      const result = spawnSync(
        process.execPath,
        [
          resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
          "--config",
          ".oxlintrc.json",
          "--format",
          "json",
          name,
        ],
        { cwd, encoding: "utf8" },
      );
      const output = JSON.parse(result.stdout);
      assert.equal(result.status, count ? 1 : 0, result.stderr);
      const diagnostics = output.diagnostics.filter((item) => item.code === `no-mistakes(${rule})`);
      assert.equal(diagnostics.length, count);
      const eslint = lint(fixture(`${rule}/${name}`), { [`no-mistakes/${rule}`]: "error" }, name);
      assert.deepEqual(
        diagnostics.map((item) => item.labels[0].span.line),
        eslint.map((item) => item.line),
      );
    });
  }
});
