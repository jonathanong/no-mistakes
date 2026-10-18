import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { __dirname } from "./helpers.mjs";

const fixtureRoot = resolve(
  __dirname,
  "../../../test-cases/eslint-plugin/require-options-on-imported-call/fixture",
);

function oxlint(configName) {
  return spawnSync(
    process.execPath,
    [
      resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
      "--config",
      resolve(fixtureRoot, configName),
      resolve(fixtureRoot, "selectors-invalid.ts"),
    ],
    { encoding: "utf8" },
  );
}

describe("oxlint selector support", () => {
  it("rejects a missing kind, mixed kinds, and an invalid pattern", () => {
    for (const configName of [
      ".oxlintrc.selectors-missing-kind.json",
      ".oxlintrc.selectors-both-kinds.json",
      ".oxlintrc.selectors-bad-pattern.json",
      ".oxlintrc.selectors-uncompilable-imported.json",
    ]) {
      const result = oxlint(configName);
      assert.equal(result.error, undefined, result.stderr);
      assert.notEqual(result.status, 0, configName);
      assert.match(
        `${result.stdout}\n${result.stderr}`,
        /pattern|oneOf|schema|invalid/i,
        configName,
      );
    }
  });

  it("reports global and receiver-shaped calls through the plugin", () => {
    const result = oxlint(".oxlintrc.selectors.json");
    assert.equal(result.error, undefined, result.stderr);
    assert.notEqual(result.status, 0);
    assert.match(
      `${result.stdout}\n${result.stderr}`,
      /statically visible options object|require-options-on-imported-call/i,
    );
  });
});
