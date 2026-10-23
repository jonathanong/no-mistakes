import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { __dirname, fixture, lint } from "./helpers.mjs";

const rule = "ts-no-function-aliases";
// Each source is saved so type identity, shadowing, and receiver counterexamples
// stay reviewable independently of the implementation's normalization helpers.
describe("cast identity and reachability safety corpus", () => {
  for (const { name, code, expected } of JSON.parse(
    fixture("ts-object-recall/cast-safety-corpus.json"),
  )) {
    it(name, () => {
      const diagnostics = lint(code, { [`no-mistakes/${rule}`]: "error" }, "fixture.ts");
      assert.equal(
        diagnostics.some((item) => item.fatal),
        false,
      );
      assert.deepEqual(
        diagnostics.map((item) => item.messageId),
        Array.from({ length: expected }, () => "alias"),
      );
    });
  }
});

describe("cast contract Oxlint parity", () => {
  for (const [name, expected] of [
    ["asserted-functions", 4],
    ["asserted-suppressed", 0],
    ["asserted-file-suppressed", 0],
  ]) {
    it(name, () => {
      const cwd = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", rule);
      const result = spawnSync(
        process.execPath,
        [
          resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
          "--config",
          ".oxlintrc.json",
          "--format",
          "json",
          resolve(cwd, `../ts-object-recall/${name}.ts`),
        ],
        { cwd, encoding: "utf8" },
      );
      assert.equal(result.status, expected ? 1 : 0, result.stderr);
      const actual = JSON.parse(result.stdout).diagnostics.filter(
        (item) => item.code === `no-mistakes(${rule})`,
      );
      assert.equal(actual.length, expected);
      assert.deepEqual(
        actual.map((item) => item.labels[0].span.line),
        lint(
          fixture(`ts-object-recall/${name}.ts`),
          { [`no-mistakes/${rule}`]: "error" },
          "fixture.ts",
        ).map((item) => item.line),
      );
    });
  }
});
