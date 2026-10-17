import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { fixture, lint, plugin, __dirname } from "./helpers.mjs";
const id = "gh-api-argument-safety";
const fixtureRoot = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", id);
const wrappers = [
  { callee: "gh", argumentPosition: 1 },
  { callee: "ghApi", argumentPosition: 2, includesApi: false },
];
function check(name, options = {}) {
  return lint(
    fixture(`${id}/${name}`),
    { [`no-mistakes/${id}`]: ["error", options] },
    resolve(fixtureRoot, name),
  );
}
describe(id, () => {
  it("reports endpoint queries and field flags without real methods", () => {
    const messages = check("unsafe.mjs");
    assert.deepEqual(
      messages.map((m) => [m.line, m.messageId]),
      [
        [3, "endpointQuery"],
        [4, "missingMethod"],
        [5, "missingMethod"],
        [6, "missingMethod"],
        [7, "missingMethod"],
        [8, "endpointQuery"],
        [9, "missingMethod"],
      ],
    );
  });
  it("keeps fields, jq, GraphQL, shadowed calls and unknown argv conservative", () => {
    assert.deepEqual(check("safe.mjs"), []);
  });
  it("selects only explicit wrappers and positions, including wrappers adding api", () => {
    assert.deepEqual(
      check("wrappers.ts", { wrappers }).map((m) => [m.line, m.messageId]),
      [
        [3, "endpointQuery"],
        [4, "missingMethod"],
        [15, "endpointQuery"],
      ],
    );
    assert.deepEqual(check("wrappers.ts"), []);
  });
  it("supports configured messages and no-mistakes suppressions", () => {
    assert.equal(
      check("unsafe.mjs", { endpointMessage: "Structured fields only" })[0].message,
      "Structured fields only",
    );
    assert.equal(
      check("unsafe.mjs", { methodMessage: "Declare the method" })[1].message,
      "Declare the method",
    );
    assert.deepEqual(
      check("suppressions.mjs").map((m) => m.line),
      [6],
    );
    assert.deepEqual(check("suppression-file.mjs"), []);
  });
  it("handles boundary flags, CommonJS provenance and conservative unknowns", () => {
    assert.deepEqual(
      check("boundaries.mjs").map((m) => [m.line, m.messageId]),
      [
        [3, "missingMethod"],
        [3, "endpointQuery"],
        [4, "missingMethod"],
        [9, "missingMethod"],
        [19, "endpointQuery"],
        [20, "missingMethod"],
        [21, "missingMethod"],
        [22, "missingMethod"],
      ],
    );
  });
  it("checks static configured members and ignores reassigned wrappers", () => {
    const configured = [
      ...wrappers,
      { callee: "api.gh", argumentPosition: 1 },
      { callee: "replaced", argumentPosition: 1 },
    ];
    assert.deepEqual(
      check("wrappers.ts", { wrappers: configured }).map((m) => m.line),
      [3, 4, 9, 10, 15],
    );
  });
  it("has real oxlint jsPlugins parity on the same saved fixture", () => {
    const config = resolve(fixtureRoot, ".oxlintrc.json");
    const result = spawnSync(
      process.execPath,
      [
        resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
        "--config",
        config,
        "--format",
        "json",
        resolve(fixtureRoot, "unsafe.mjs"),
      ],
      { encoding: "utf8" },
    );
    assert.equal(result.status, 1, result.stderr);
    const report = JSON.parse(result.stdout);
    assert.equal(report.diagnostics.length, 7);
    assert.ok(report.diagnostics.every((d) => d.code.includes(id)));
  });
  it("preserves oxlint parity for safe calls, wrapper scope and suppressions", () => {
    for (const [name, count] of [
      ["safe.mjs", 0],
      ["boundaries.mjs", 8],
      ["wrappers.ts", 5],
      ["suppressions.mjs", 1],
      ["suppression-file.mjs", 0],
    ]) {
      const result = spawnSync(
        process.execPath,
        [
          resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
          "--config",
          resolve(fixtureRoot, ".oxlintrc.json"),
          "--format",
          "json",
          resolve(fixtureRoot, name),
        ],
        { encoding: "utf8" },
      );
      assert.equal(result.status, count ? 1 : 0, result.stderr);
      const diagnostics = JSON.parse(result.stdout).diagnostics.filter((entry) =>
        entry.code.includes(id),
      );
      assert.equal(diagnostics.length, count, name);
    }
  });
  it("exports a non-recommended opt-in rule", () => {
    assert.ok(plugin.rules[id]);
    assert.equal(plugin.configs.recommended.rules[`no-mistakes/${id}`], undefined);
  });
});
