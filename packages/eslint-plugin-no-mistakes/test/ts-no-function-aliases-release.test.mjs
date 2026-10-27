import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import tsParser from "@typescript-eslint/parser";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { __dirname, fixture, lint } from "./helpers.mjs";

const rule = "ts-no-function-aliases";
const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};
const tsLanguage = { parser: tsParser };
const privateWrappers = readFileSync(
  resolve(__dirname, "../../../test-cases/eslint-plugin", rule, "fixture/private-wrappers.ts"),
  "utf8",
);

function releaseCase(code, filename) {
  const item = { code, filename };
  if (/\.[cm]?tsx?$/.test(filename)) item.languageOptions = tsLanguage;
  return item;
}

const oxlintNames = [
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
];

const suite = {
  valid: oxlintNames
    .filter(([, count]) => count === 0)
    .map(([name]) => releaseCase(fixture(`${rule}/${name}`), name)),
  invalid: [
    // 13 alias. Released identifier findings in historical private receiver cases.
    releaseCase(privateWrappers, "fixture.ts"),
    // 34 alias. These lines match the frozen v0.95.0 oracle, including private targets.
    releaseCase(fixture(`${rule}/release-controls.ts`), "fixture.ts"),
    // 41 alias. Reachable imported and exported pass-throughs, including forwarding.
    releaseCase(fixture(`${rule}/release-positive.ts`), "fixture.ts"),
    ...oxlintNames
      .filter(([, count]) => count !== 0)
      .map(([name]) => releaseCase(fixture(`${rule}/${name}`), name)),
  ],
};

describe("reachable function alias release regression", () => {
  it("snapshots ts-no-function-aliases-release diagnostics", async () => {
    await expect(
      snapshotDocument({
        [rule]: snapshotSuite(rule, languageOptions, suite),
      }),
    ).toMatchFileSnapshot("snapshots/ts-no-function-aliases-release.snap");
  }, 30_000);
});

describe("reachable function alias Oxlint parity", () => {
  for (const [name, count] of oxlintNames) {
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
