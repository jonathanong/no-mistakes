import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { fixture, lint, plugin, __dirname } from "./helpers.mjs";
import { headingSection, rulePages, sourceFences } from "./docs-snippets.mjs";
const id = "no-argv-main-guard";
const fixtureRoot = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", id);
function check(name, options = {}) {
  return lint(
    fixture(`${id}/${name}`),
    { [`no-mistakes/${id}`]: ["error", options] },
    resolve(fixtureRoot, name),
  );
}
describe(id, () => {
  it("matches equality, main identities, Node wrappers and optional argv forms", () => {
    assert.deepEqual(
      check("unsafe.mjs").map((m) => m.line),
      [5, 6, 7, 8, 9, 10, 11, 12, 13, 14],
    );
  });
  it("follows direct const bindings while rejecting shadowing and unsupported flow", () => {
    assert.deepEqual(
      check("aliases.ts").map((m) => m.line),
      [5, 6, 9, 11, 20],
    );
  });
  it("does not diagnose ordinary argv reads, unrelated paths or shadowed wrappers", () => {
    assert.deepEqual(check("safe.mjs"), []);
    assert.deepEqual(
      check("commonjs.cjs").map((m) => m.line),
      [4],
    );
  });
  it("supports custom messages and actual-comment suppression", () => {
    assert.equal(check("unsafe.mjs", { message: "Inject isMain" })[0].message, "Inject isMain");
    assert.deepEqual(
      check("suppressions.mjs").map((m) => m.line),
      [5],
    );
    assert.deepEqual(check("suppression-file.mjs"), []);
  });
  it("recognizes exact file URL templates through the existing identity classifier", () => {
    assert.deepEqual(
      check("templates.ts").map((m) => m.line),
      [4, 5, 6, 7, 10, 11, 12, 13, 15],
    );
    assert.deepEqual(check("template-controls.mjs"), []);
    assert.deepEqual(
      check("template-suppressions.mjs").map((m) => m.line),
      [6],
    );
    assert.deepEqual(check("template-suppression-file.mjs"), []);
  });
  it("normalizes concatenated and wrapped file URL guards", () => {
    assert.deepEqual(
      check("file-url-recall.mjs").map((m) => m.line),
      [2, 3, 4, 5, 6, 7, 8, 9, 10, 15, 16, 18, 19, 20, 21, 23],
    );
    assert.deepEqual(check("file-url-controls.mjs"), []);
  });
  it("executes every documented disallowed guard and the allowed example", () => {
    const page = rulePages().find((page) => page.ruleId === id);
    const disallowed = sourceFences(headingSection(page.body, "Disallowed"))[0];
    const allowed = sourceFences(headingSection(page.body, "Allowed"))[0];
    const rules = { [`no-mistakes/${id}`]: "error" };
    const findings = lint(disallowed.code, rules, "documented.ts");
    assert.equal(findings.length, 12);
    assert.ok(findings.every((message) => message.ruleId === `no-mistakes/${id}`));
    assert.deepEqual(lint(allowed.code, rules, "documented.ts"), []);
  });
  for (const [name, count] of [
    ["file-url-recall.mjs", 16],
    ["file-url-controls.mjs", 0],
    ["unsafe.mjs", 10],
    ["aliases.ts", 5],
    ["safe.mjs", 0],
    ["suppressions.mjs", 1],
    ["suppression-file.mjs", 0],
    ["commonjs.cjs", 1],
    ["templates.ts", 9],
    ["template-controls.mjs", 0],
    ["template-suppressions.mjs", 1],
    ["template-suppression-file.mjs", 0],
  ]) {
    it(`has genuine oxlint parity for ${name}`, () => {
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
      const diagnostics = JSON.parse(result.stdout).diagnostics.filter((d) => d.code.includes(id));
      assert.equal(diagnostics.length, count, name);
    });
  }
  it("bounds alias expansion and rejects forward initialization", () => {
    assert.deepEqual(
      check("bounded-aliases.mjs").map((m) => m.line),
      [3],
    );
  });
  it("exports an opt-in rule without an unsafe fix", () => {
    assert.ok(plugin.rules[id]);
    assert.equal(plugin.configs.recommended.rules[`no-mistakes/${id}`], undefined);
    assert.equal(plugin.rules[id].meta.fixable, undefined);
  });
});
