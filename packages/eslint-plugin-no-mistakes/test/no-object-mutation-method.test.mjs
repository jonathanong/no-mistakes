import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { fixture, messages } from "./helpers.mjs";
import { headingSection, rulePages, sourceFences } from "./docs-snippets.mjs";

const rule = "no-object-mutation";
const aliases = fixture("no-object-mutation-method-aliases.ts");
const mutations = (count) => Array.from({ length: count }, () => "mutation");

describe("no-object-mutation method aliases", () => {
  it("retains receiver provenance through static aliases and forwarding", () => {
    assert.deepEqual(messages(aliases, rule, undefined, "fixture.ts"), mutations(8));
  });
  it("keeps origin options and allow paths on the actual receiver", () => {
    for (const [option, count] of [
      [{ calls: false }, 0],
      [{ module: false }, 7],
      [{ imported: false }, 7],
      [{ parameters: false }, 2],
      [{ allow: ["moduleRows", "importedRows", "rows", "more"] }, 0],
    ])
      assert.deepEqual(messages(aliases, rule, option, "fixture.ts"), mutations(count));
  });
  it("does not guess dynamic methods, defaults, shadowed or rebound bindings", () => {
    assert.deepEqual(
      messages(fixture("no-object-mutation-method-controls.ts"), rule, undefined, "fixture.ts"),
      [],
    );
  });
  it("retains captured receiver identity, escape state, and known function bodies", () => {
    assert.deepEqual(
      messages(fixture("no-object-mutation-method-provenance.ts"), rule, undefined, "fixture.ts"),
      mutations(19),
    );
  });
  it("walks inner writes even when forwarding cannot prove an outer mutation", () => {
    assert.deepEqual(
      messages(fixture("no-object-mutation-method-inner-writes.ts"), rule, undefined, "fixture.ts"),
      mutations(4),
    );
  });
  it("executes the real method-alias documentation examples and limits", () => {
    const page = rulePages().find((page) => page.ruleId === rule);
    const disallowed = sourceFences(headingSection(page.body, "Disallowed")).find((snippet) =>
      snippet.code.includes("function append("),
    );
    assert.deepEqual(messages(disallowed.code, rule, undefined, "fixture.ts"), mutations(6));
    for (const snippet of sourceFences(headingSection(page.body, "Limits"))) {
      assert.deepEqual(messages(snippet.code, rule, undefined, "fixture.ts"), []);
    }
  });
  it("honors allowThis and dotted allow paths on saved receiver aliases", () => {
    const code = fixture("no-object-mutation-method-options.ts");
    assert.deepEqual(messages(code, rule, undefined, "fixture.ts"), mutations(2));
    assert.deepEqual(messages(code, rule, { allowThis: false }, "fixture.ts"), mutations(3));
    assert.deepEqual(messages(code, rule, { allow: ["rows.allowed"] }, "fixture.ts"), mutations(1));
  });
  it("preserves ordinary method-object reads and writes", () => {
    const code = fixture("no-object-mutation-method-properties.ts");
    assert.deepEqual(messages(code, rule, undefined, "fixture.ts"), mutations(8));
    assert.deepEqual(messages(code, rule, { parameters: false }, "fixture.ts"), mutations(1));
    assert.deepEqual(messages(code, rule, { module: false }, "fixture.ts"), mutations(7));
  });
  it("honors saved line and file suppression comments", () => {
    for (const name of [
      "no-object-mutation-method-suppressed.ts",
      "no-object-mutation-method-file-suppressed.ts",
    ]) {
      assert.deepEqual(messages(fixture(name), rule, undefined, "fixture.ts"), []);
    }
  });
});
