import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { __dirname, fixture, lint, messages } from "./helpers.mjs";

const check = (rule, file) => messages(fixture(`${rule}/${file}`), rule, undefined, file);

describe("test rule static recall follow-up", () => {
  it("reads sequential options from literal apply arguments", () => {
    assert.deepEqual(
      check("no-vitest-sequential", "static-apply.test.ts"),
      Array(4).fill("sequential"),
    );
    assert.deepEqual(check("no-vitest-sequential", "static-apply-controls.test.ts"), []);
  });
  it("recognizes mocking aliases and static forwarding", () => {
    assert.deepEqual(check("vitest-mock-test-file-naming", "static-aliases.test.ts"), [
      "needsMock",
    ]);
    for (const name of [
      "const",
      "call",
      "apply",
      "destructure",
      "namespace",
      "template",
      "member",
      "method",
    ]) {
      assert.deepEqual(
        check("vitest-mock-test-file-naming", `static-alias-${name}.test.ts`),
        ["needsMock"],
        name,
      );
    }
    assert.deepEqual(check("vitest-mock-test-file-naming", "static-aliases-controls.test.ts"), []);
    assert.deepEqual(check("vitest-mock-test-file-naming", "static-alias-cycle.test.js"), []);
  });
  it("recognizes Playwright expect and test runner aliases", () => {
    assert.deepEqual(
      check("test-no-error-message-matching", "static-runner.test.ts"),
      Array(5).fill("message"),
    );
    assert.deepEqual(check("test-no-error-message-matching", "static-runner-controls.test.ts"), []);
    assert.deepEqual(
      check("test-no-error-message-matching", "static-runner-aliases.test.ts"),
      Array(6).fill("message"),
    );
    assert.deepEqual(
      check("test-no-error-message-matching", "static-runner-module.test.ts"),
      Array(2).fill("message"),
    );
    assert.deepEqual(check("test-no-error-message-matching", "static-runner-opaque.test.ts"), []);
    assert.deepEqual(check("test-no-error-message-matching", "static-namespace-write.test.js"), []);
  });
  it("keeps Playwright outside the delayed-rejects contract", () => {
    assert.deepEqual(check("test-no-delayed-rejects", "playwright-static-expect-controls.ts"), []);
  });
  it("recognizes require aliases while preserving real work and unknown loaders", () => {
    assert.deepEqual(check("no-import-only-test-files", "require-alias.test.js"), ["aggregate"]);
    for (const suffix of [
      "controls",
      "shadow",
      "mutable",
      "suppressed",
      "opaque",
      "cycle",
      "redeclared",
      "dynamic",
      "extra",
      "file",
      "line",
    ]) {
      assert.deepEqual(
        check("no-import-only-test-files", `require-alias-${suffix}.test.js`),
        [],
        suffix,
      );
    }
  });
  it("matches real Oxlint for the saved static-shape fixtures", () => {
    for (const [rule, file] of [
      ["no-vitest-sequential", "static-apply.test.ts"],
      ["no-vitest-sequential", "static-apply-controls.test.ts"],
      ["vitest-mock-test-file-naming", "static-aliases.test.ts"],
      ["vitest-mock-test-file-naming", "static-aliases-controls.test.ts"],
      ["test-no-error-message-matching", "static-runner.test.ts"],
      ["test-no-error-message-matching", "static-runner-controls.test.ts"],
      ["no-import-only-test-files", "require-alias.test.js"],
      ["no-import-only-test-files", "forwarded.test.js"],
      ["no-import-only-test-files", "require-alias-controls.test.js"],
    ]) {
      const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", rule);
      const result = spawnSync(
        process.execPath,
        [
          resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
          "--config",
          resolve(root, ".oxlintrc-static-followup.json"),
          "--format",
          "json",
          resolve(root, file),
        ],
        { encoding: "utf8" },
      );
      const expected = lint(fixture(`${rule}/${file}`), { [`no-mistakes/${rule}`]: "error" }, file);
      assert.equal(result.status, expected.length ? 1 : 0, `${rule}/${file}: ${result.stderr}`);
      const actual = JSON.parse(result.stdout).diagnostics.filter((item) =>
        item.code.includes(rule),
      );
      assert.deepEqual(
        actual.map((item) => item.labels[0].span.line),
        expected.map((item) => item.line),
        `${rule}/${file}`,
      );
    }
  });
});
