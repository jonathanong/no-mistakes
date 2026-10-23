import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { __dirname, fixture, lint, messages } from "./helpers.mjs";

const rule = "ts-no-function-aliases";
describe("reachable function alias release regression", () => {
  it("preserves all historical private receiver failures as explicit negative cases", () => {
    const code = readFileSync(
      resolve(__dirname, "../../../test-cases/eslint-plugin", rule, "fixture/private-wrappers.ts"),
      "utf8",
    );
    assert.deepEqual(messages(code, rule, undefined, "fixture.ts"), []);
  });
  it("keeps private, runtime and asserted controls while preserving released identifier findings", () => {
    const diagnostics = lint(
      fixture(`${rule}/release-controls.ts`),
      { [`no-mistakes/${rule}`]: "error" },
      "fixture.ts",
    );
    // Only the plain reachable identifiers regain the released behavior; every
    // expanded/private/asserted control in this historical file stays silent.
    assert.deepEqual(
      diagnostics.map((item) => item.line),
      [
        18, 19, 20, 21, 32, 34, 36, 40, 41, 42, 44, 45, 46, 50, 54, 56, 77, 79, 81, 89, 90, 103,
        104, 106,
      ],
    );
  });
  it("reports reachable imported and exported pass-throughs including forwarding", () => {
    assert.deepEqual(
      messages(fixture(`${rule}/release-positive.ts`), rule, undefined, "fixture.ts"),
      Array.from({ length: 24 }, () => "alias"),
    );
  });
});

describe("reachable function alias Oxlint parity", () => {
  for (const [name, count] of [
    ["release-controls.ts", 24],
    ["release-jsdoc.mjs", 7],
    ["release-positive.ts", 24],
    ["release-default-const.ts", 2],
    ["legacy-compatibility.ts", 14],
    ["legacy-instantiation.ts", 1],
    ["expanded-contracts.ts", 5],
    ["private-recall.ts", 0],
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
