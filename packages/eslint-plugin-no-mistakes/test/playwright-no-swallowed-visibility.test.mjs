import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { fixture, lint, messages, plugin } from "./helpers.mjs";

const RULE = "playwright-no-swallowed-visibility";
const SWALLOWED = "swallowed";

function code(name) {
  return fixture(`${RULE}/${name}`);
}

function ids(name, option, filename = name) {
  return messages(code(name), RULE, option, filename);
}

describe(RULE, () => {
  it("is opt-in and not a fixer", () => {
    assert.equal(plugin.rules[RULE].meta.docs.recommended, false);
    assert.equal(plugin.rules[RULE].meta.fixable, undefined);
    assert.equal(plugin.configs.recommended.rules[`no-mistakes/${RULE}`], undefined);
    assert.equal(plugin.configs.strict.rules[`no-mistakes/${RULE}`], undefined);
  });

  it("reports a catch that returns a boolean literal", () => {
    assert.deepEqual(ids("catch-false.mjs"), [SWALLOWED]);
    assert.deepEqual(ids("catch-true.mjs"), [SWALLOWED]);
  });

  it("reports a try/catch that returns a boolean literal", () => {
    assert.deepEqual(ids("try-catch-literal.mjs"), [SWALLOWED, SWALLOWED]);
  });

  it("stays silent when a named handler rethrows", () => {
    assert.deepEqual(ids("catch-rethrow.mjs"), []);
  });

  it("stays silent for a plain expect assertion", () => {
    assert.deepEqual(ids("expect-assertion.mjs"), []);
  });

  it("reports block bodies, optional calls, and a returned await", () => {
    assert.deepEqual(ids("invalid-shapes.mjs"), Array(6).fill(SWALLOWED));
    assert.deepEqual(ids("asserted.ts"), [SWALLOWED, SWALLOWED]);
  });

  it("ignores non-literals, other methods, and awaits that leave the try", () => {
    assert.deepEqual(ids("valid-shapes.mjs"), []);
    assert.deepEqual(ids("attached.mjs"), []);
  });

  it("does not attribute an inner catch to the outer boolean", () => {
    assert.deepEqual(ids("nested-try.mjs"), [SWALLOWED, SWALLOWED]);
  });

  it("replaces the default method list", () => {
    assert.deepEqual(ids("catch-false.mjs", { methods: ["isAttached"] }), []);
    assert.deepEqual(ids("catch-false.mjs", { methods: [] }), []);
    assert.deepEqual(ids("catch-true.mjs", { methods: ["isVisible"] }), []);
    assert.deepEqual(ids("attached.mjs", { methods: ["isAttached"] }), [SWALLOWED]);
    assert.deepEqual(ids("catch-false.mjs", { methods: ["isVisible"] }), [SWALLOWED]);
  });

  it("uses the configured message", () => {
    const ruleId = `no-mistakes/${RULE}`;
    const custom = lint(
      code("catch-false.mjs"),
      { [ruleId]: ["error", { message: "Seed the row first." }] },
      "catch-false.mjs",
    );
    assert.equal(custom[0].message, "Seed the row first.");
    const fallback = lint(code("catch-false.mjs"), { [ruleId]: "error" }, "catch-false.mjs");
    assert.match(fallback[0].message, /toBeVisible/);
    assert.match(fallback[0].message, /toBeHidden/);
  });

  it("honors file, line, and next-line suppression for this rule only", () => {
    assert.deepEqual(ids("suppression-file.mjs"), []);
    assert.deepEqual(ids("suppressions.mjs"), [SWALLOWED, SWALLOWED]);
  });
});
