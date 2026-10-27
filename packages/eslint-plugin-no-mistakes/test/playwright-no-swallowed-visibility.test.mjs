import assert from "node:assert/strict";
import { describe, expect, it } from "vitest";
import { pluginFindings, snapshotCases, snapshotDocument } from "./diagnostic-snapshot.mjs";
import { fixture, plugin } from "./helpers.mjs";

const RULE = "playwright-no-swallowed-visibility";

function visibility(name, file, option) {
  return {
    name,
    file,
    rule: RULE,
    ...(option !== undefined ? { options: option } : {}),
    findings: pluginFindings(fixture(`${RULE}/${file}`), RULE, option, file),
  };
}

describe(RULE, () => {
  it("is opt-in and not a fixer", () => {
    assert.equal(plugin.rules[RULE].meta.docs.recommended, false);
    assert.equal(plugin.rules[RULE].meta.fixable, undefined);
    assert.equal(plugin.configs.recommended.rules[`no-mistakes/${RULE}`], undefined);
    assert.equal(plugin.configs.strict.rules[`no-mistakes/${RULE}`], undefined);
  });
});

it("snapshots playwright-no-swallowed-visibility diagnostics", async () => {
  const cases = [
    visibility(
      "reports a catch that returns a boolean literal: catch-false.mjs",
      "catch-false.mjs",
    ),
    visibility("reports a catch that returns a boolean literal: catch-true.mjs", "catch-true.mjs"),
    visibility("reports a try/catch that returns a boolean literal", "try-catch-literal.mjs"),
    visibility("stays silent when a named handler rethrows", "catch-rethrow.mjs"),
    visibility("stays silent for a plain expect assertion", "expect-assertion.mjs"),
    visibility(
      "reports block bodies, optional calls, and a returned await: invalid-shapes.mjs",
      "invalid-shapes.mjs",
    ),
    visibility(
      "reports block bodies, optional calls, and a returned await: asserted.ts",
      "asserted.ts",
    ),
    visibility(
      "ignores non-literals, other methods, and awaits that leave the try: valid-shapes.mjs",
      "valid-shapes.mjs",
    ),
    visibility(
      "ignores non-literals, other methods, and awaits that leave the try: attached.mjs",
      "attached.mjs",
    ),
    visibility("does not attribute an inner catch to the outer boolean", "nested-try.mjs"),
    visibility(
      "reports a numeric not and a returned state binding: bang-literal.mjs",
      "bang-literal.mjs",
    ),
    visibility(
      "reports a numeric not and a returned state binding: bound-state.mjs",
      "bound-state.mjs",
    ),
    visibility("replaces the default method list: catch-false.mjs isAttached", "catch-false.mjs", {
      methods: ["isAttached"],
    }),
    visibility("replaces the default method list: catch-false.mjs empty", "catch-false.mjs", {
      methods: [],
    }),
    visibility("replaces the default method list: catch-true.mjs isVisible", "catch-true.mjs", {
      methods: ["isVisible"],
    }),
    visibility("replaces the default method list: attached.mjs isAttached", "attached.mjs", {
      methods: ["isAttached"],
    }),
    visibility("replaces the default method list: catch-false.mjs isVisible", "catch-false.mjs", {
      methods: ["isVisible"],
    }),
    visibility("uses the configured message: custom", "catch-false.mjs", {
      message: "Seed the row first.",
    }),
    visibility("uses the configured message: fallback", "catch-false.mjs"),
    visibility(
      "honors file, line, and next-line suppression for this rule only: suppression-file.mjs",
      "suppression-file.mjs",
    ),
    visibility(
      "honors file, line, and next-line suppression for this rule only: suppressions.mjs",
      "suppressions.mjs",
    ),
  ];
  await expect(snapshotDocument(snapshotCases(cases))).toMatchFileSnapshot(
    "snapshots/playwright-no-swallowed-visibility.snap",
  );
});
