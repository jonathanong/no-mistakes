import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { fixture, lint, __dirname } from "./helpers.mjs";

const id = "gh-api-argument-safety";
const options = {
  wrappers: [
    { callee: "ghApi", argumentPosition: 2, includesApi: false },
    { callee: "api.gh", argumentPosition: 1 },
    { callee: "api.call", argumentPosition: 1 },
    { callee: "api.apply", argumentPosition: 1 },
  ],
};
function check(name) {
  return lint(
    fixture(`${id}/${name}.ts`),
    { [`no-mistakes/${id}`]: ["error", options] },
    `${name}.ts`,
  );
}
describe("configured GH wrapper recall", () => {
  it("follows stable aliases, destructures, static members and forwarding arguments", () => {
    assert.deepEqual(
      check("wrapper-recall").map((m) => [m.line, m.messageId]),
      [
        [4, "endpointQuery"],
        [5, "endpointQuery"],
        [6, "missingMethod"],
        [8, "endpointQuery"],
        [10, "endpointQuery"],
        [12, "endpointQuery"],
        [13, "endpointQuery"],
        [15, "endpointQuery"],
        [18, "endpointQuery"],
        [21, "endpointQuery"],
        [26, "endpointQuery"],
      ],
    );
  });
  it("preserves shadowing, dynamic keys, fallback and argument uncertainty", () => {
    assert.deepEqual(check("wrapper-recall-controls"), []);
    assert.deepEqual(check("wrapper-recall-cycle"), []);
  });
  it("matches real Oxlint diagnostics for the same saved fixtures", () => {
    const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", id);
    for (const name of ["wrapper-recall", "wrapper-recall-controls", "wrapper-recall-suppressed"]) {
      const result = spawnSync(
        process.execPath,
        [
          resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
          "--config",
          resolve(root, ".oxlintrc-wrapper-recall.json"),
          "--format",
          "json",
          resolve(root, `${name}.ts`),
        ],
        { encoding: "utf8" },
      );
      const expected = check(name);
      assert.equal(result.status, expected.length ? 1 : 0, result.stderr);
      const actual = JSON.parse(result.stdout).diagnostics.filter((d) => d.code.includes(id));
      assert.deepEqual(
        actual.map((d) => d.labels[0].span.line).sort((a, b) => a - b),
        expected.map((d) => d.line),
      );
    }
  });
  it("honors file suppression for aliased forwarding calls", () => {
    assert.deepEqual(check("wrapper-recall-suppressed"), []);
  });
});
