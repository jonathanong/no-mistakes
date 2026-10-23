import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { fixture, lint, __dirname } from "./helpers.mjs";
import { runInNewContext } from "node:vm";
const id = "require-options-on-imported-call";
const requirement = { optionsPosition: 2, requiredProperties: ["safe"] };
const options = {
  targets: [
    { globalNamePatterns: ["createResponse"], ...requirement },
    { receiverPatterns: ["client.responses"], calleeNamePatterns: ["create"], ...requirement },
    { receiverPatterns: ["client"], calleeNamePatterns: ["call"], ...requirement },
  ],
};
function check(name, overrides = {}) {
  return lint(
    fixture(`${id}/${name}.mjs`),
    { [`no-mistakes/${id}`]: ["error", { ...options, ...overrides }] },
    `${name}.mjs`,
  );
}
describe("required global and receiver selector aliases", () => {
  it("follows same-file immutable callee, receiver and destructured aliases", () => {
    assert.deepEqual(
      check("selector-aliases").map((m) => [m.line, m.messageId]),
      [1, 3, 4, 6, 8, 10, 11, 12, 14, 17, 20, 25, 26, 27, 28, 29, 30, 31].map((line) => [
        line,
        "missingOptions",
      ]),
    );
  });
  it("preserves binding, dynamic key, default and uncertain forwarding limits", () => {
    assert.deepEqual(check("selector-alias-controls"), []);
    assert.deepEqual(check("selector-alias-suppressed"), []);
  });
  it("does not classify a property default that actually invokes another API", () => {
    assert.equal(runInNewContext(fixture(`${id}/selector-default-fallback.mjs`)), "other:input");
    assert.deepEqual(check("selector-default-fallback"), []);
  });
  it("preserves configured receiver roots initialized through opaque expressions", () => {
    assert.deepEqual(
      check("selector-receiver-roots").map((m) => m.line),
      [3, 5, 9],
    );
  });
  it("keeps a broad receiver pattern from treating a bare alias as its receiver", () => {
    assert.deepEqual(
      check("selector-receiver-roots", {
        targets: [{ receiverPatterns: ["**"], calleeNamePatterns: ["create"], ...requirement }],
      }).map((m) => m.line),
      [3, 5, 9],
    );
  });
  it("keeps literal dotted and slashed keys distinct from nested receiver segments", () => {
    assert.deepEqual(
      Array.from(runInNewContext(fixture(`${id}/selector-segment-boundaries.mjs`))),
      ["dotted", "dotted", "slashed", "slashed", "nested"],
    );
    const nested = {
      receiverPatterns: ["client.foo.responses"],
      calleeNamePatterns: ["create"],
      ...requirement,
    };
    assert.deepEqual(
      check("selector-segment-boundaries", { targets: [nested] }).map((m) => m.line),
      [14],
    );
    assert.deepEqual(
      check("selector-segment-boundaries", {
        targets: [{ ...nested, receiverPatterns: ["client.*"] }],
      }).map((m) => m.line),
      [8, 10, 11, 13],
    );
    // Regex selectors retain their documented joined dotted-text input.
    assert.deepEqual(
      check("selector-segment-boundaries", {
        targets: [{ ...nested, receiverPatterns: ["/^client\\.foo\\.responses$/"] }],
      }).map((m) => m.line),
      [8, 10, 14],
    );
  });
  it("matches literal percent and slash bytes in configured receiver segments", () => {
    assert.deepEqual(Array.from(runInNewContext(fixture(`${id}/selector-literal-patterns.mjs`))), [
      "percent",
      "percent",
      "slashed",
      "slashed",
    ]);
    for (const [pattern, lines] of [
      ["client.foo%2Fbar", [6, 8]],
      ["client.foo/responses", [9, 11]],
    ]) {
      assert.deepEqual(
        check("selector-literal-patterns", {
          targets: [
            {
              receiverPatterns: [pattern],
              calleeNamePatterns: ["create"],
              ...requirement,
            },
          ],
        }).map((m) => m.line),
        lines,
      );
    }
  });
  it("matches real Oxlint diagnostics for the saved aliases and controls", () => {
    const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", id);
    const entries = [
      "selector-aliases",
      "selector-alias-controls",
      "selector-alias-suppressed",
      "selector-receiver-roots",
      "selector-default-fallback",
    ].map((name) => [name, ".oxlintrc-selector-aliases.json", {}]);
    entries.push([
      "selector-segment-boundaries",
      ".oxlintrc-selector-segments.json",
      {
        targets: [
          {
            receiverPatterns: ["client.foo.responses"],
            calleeNamePatterns: ["create"],
            ...requirement,
          },
        ],
      },
    ]);
    entries.push([
      "selector-literal-patterns",
      ".oxlintrc-selector-literals.json",
      {
        targets: [
          {
            receiverPatterns: ["client.foo%2Fbar", "client.foo/responses"],
            calleeNamePatterns: ["create"],
            ...requirement,
          },
        ],
      },
    ]);
    for (const [name, config, overrides] of entries) {
      const expected = check(name, overrides);
      const result = spawnSync(
        process.execPath,
        [
          resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
          "--config",
          resolve(root, config),
          "--format",
          "json",
          resolve(root, `${name}.mjs`),
        ],
        { encoding: "utf8" },
      );
      assert.equal(result.status, expected.length ? 1 : 0, result.stderr);
      const actual = JSON.parse(result.stdout).diagnostics.filter((d) => d.code.includes(id));
      assert.deepEqual(
        actual.map((d) => d.labels[0].span.line).sort((a, b) => a - b),
        expected.map((d) => d.line),
      );
    }
  });
  it("deduplicates overlapping requirements on aliased calls", () => {
    assert.equal(
      check("selector-aliases", { targets: [...options.targets, options.targets[0]] }).length,
      18,
    );
  });
});
