import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { __dirname } from "./helpers.mjs";

const fixtureRoot = resolve(
  __dirname,
  "../../../test-cases/eslint-plugin/require-options-on-imported-call/fixture",
);

describe("oxlint selector support", () => {
  it("reports global and receiver-shaped calls through the plugin", () => {
    const result = spawnSync(
      process.execPath,
      [
        resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
        "--config",
        resolve(fixtureRoot, ".oxlintrc.selectors.json"),
        resolve(fixtureRoot, "selectors-invalid.ts"),
      ],
      { encoding: "utf8" },
    );
    assert.equal(result.error, undefined, result.stderr);
    assert.notEqual(result.status, 0);
    assert.match(
      `${result.stdout}\n${result.stderr}`,
      /statically visible options object|require-options-on-imported-call/i,
    );
  });
});
