import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { __dirname, fixture, lint } from "./helpers.mjs";

const RULE = "playwright-require-interactive-test-id";
const options = { interactiveComponents: ["Button"] };
function findings(name, overrides = {}) {
  const code = fixture(`${RULE}/${name}.tsx`);
  return lint(
    code,
    { [`no-mistakes/${RULE}`]: ["error", { ...options, ...overrides }] },
    "slot.tsx",
  ).map(({ messageId, line, column, endColumn }) => [
    messageId,
    line,
    code.split("\n")[line - 1].slice(column - 1, endColumn - 1),
  ]);
}

describe("interactive test-id slot parents", () => {
  it("covers the single direct child through literal parent selectors", () => {
    assert.deepEqual(findings("slots-valid"), []);
  });
  it("reports absent or nonliteral slot selectors only on the parent", () => {
    assert.deepEqual(
      findings("slots-missing"),
      [2, 5, 8, 11, 14, 17, 20, 23].map((line) => ["slot", line, "Button"]),
    );
  });
  it("retains checks for non-slots, ambiguous children and overrides", () => {
    assert.deepEqual(findings("slots-controls"), [
      ...[2, 3, 4, 5, 6, 7, 8, 9, 10].map((line) => ["missing", line, "a"]),
      ["missing", 11, "a"],
      ["missing", 11, "button"],
      ["missing", 12, "button"],
      ...[13, 14, 15, 16, 17, 18, 19, 21].map((line) => ["missing", line, "a"]),
      ["missing", 22, "span"],
      ["slot", 23, "Button"],
      ["missing", 23, "a"],
      ["missing", 23, "button"],
    ]);
  });
  it("replaces the default slot prop and respects custom selector attributes", () => {
    assert.deepEqual(
      findings("slots-custom", { slotProps: ["renderChild"], selectorAttributes: ["data-qa"] }),
      [["missing", 3, "a"]],
    );
    assert.deepEqual(findings("slots-custom", { selectorAttributes: ["data-qa"] }), [
      ["missing", 2, "a"],
    ]);
  });
  it("can disable slot recognition", () => {
    assert.equal(findings("slots-valid", { slotProps: [] }).length, 9);
  });
  it("keeps a suppressed parent finding suppressed through the child", () => {
    assert.deepEqual(findings("slots-suppressed"), []);
  });
  it("does not inherit through a shadow of a configured import", () => {
    assert.deepEqual(findings("slots-shadowed"), [
      ["missing", 5, "a"],
      ["missing", 8, "a"],
    ]);
  });
  it("does not treat configured intrinsic tags as slot components", () => {
    const expected = [
      ["missing", 2, "a"],
      ["missing", 3, "button"],
      ["missing", 4, "a"],
    ];
    assert.deepEqual(
      findings("slots-native", {
        interactiveComponents: ["button", "my-button", "Native-Button"],
      }),
      expected,
    );
    assert.deepEqual(findings("slots-native", { interactiveComponents: ["/.*/"] }), expected);
  });
  it("matches real Oxlint on saved slot fixtures and custom options", () => {
    const root = resolve(__dirname, `../../../test-cases/eslint-snippets/fixture/${RULE}`);
    const entries = [
      "slots-valid",
      "slots-missing",
      "slots-controls",
      "slots-suppressed",
      "slots-shadowed",
    ].map((name) => [name, ".oxlintrc-slots.json", {}]);
    entries.push([
      "slots-custom",
      ".oxlintrc-slots-custom.json",
      {
        slotProps: ["renderChild"],
        selectorAttributes: ["data-qa"],
      },
    ]);
    for (const [name, config, overrides] of entries) {
      const expected = findings(name, overrides).map((finding) => finding[1]);
      const result = spawnSync(
        process.execPath,
        [
          resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
          "--config",
          resolve(root, config),
          "--format",
          "json",
          resolve(root, `${name}.tsx`),
        ],
        { encoding: "utf8" },
      );
      assert.equal(result.status, expected.length ? 1 : 0, result.stderr);
      const actual = JSON.parse(result.stdout)
        .diagnostics.filter((diagnostic) => diagnostic.code.includes(RULE))
        .map((diagnostic) => diagnostic.labels[0].span.line)
        .sort((a, b) => a - b);
      assert.deepEqual(actual, expected, name);
    }
  });
});
