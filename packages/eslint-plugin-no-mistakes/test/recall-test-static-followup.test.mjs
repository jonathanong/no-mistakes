import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readdirSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { pluginFindings, snapshotCases, snapshotDocument } from "./diagnostic-snapshot.mjs";
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
  it("snapshots test diagnostics", async () => {
    const specs = [
      ["test-no-error-message-matching", "static-runner.test.ts"],
      ["test-no-error-message-matching", "static-runner-controls.test.ts"],
      ["test-no-error-message-matching", "static-runner-aliases.test.ts"],
      ["test-no-error-message-matching", "static-runner-module.test.ts"],
      ["test-no-error-message-matching", "static-runner-opaque.test.ts"],
      ["test-no-error-message-matching", "static-namespace-write.test.js"],
      ["test-no-delayed-rejects", "playwright-static-expect-controls.ts"],
    ];
    const cases = specs.map(([rule, file]) => ({
      file,
      findings: pluginFindings(fixture(`${rule}/${file}`), rule, undefined, file),
      name: file,
      rule,
    }));
    await expect(snapshotDocument(snapshotCases(cases))).toMatchFileSnapshot(
      "snapshots/recall-test-static-followup.snap",
    );
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
  for (const rule of [
    "no-import-only-test-files",
    "no-vitest-sequential",
    "postgres-no-correlated-sql-builder",
    "react-no-nullish-react-node",
    "test-no-error-message-matching",
    "vitest-mock-test-file-naming",
  ]) {
    it(`matches type-aware Oxlint for every supported ${rule} fixture`, () => {
      const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", rule);
      const files = readdirSync(root, { recursive: true })
        .filter((file) => /\.[cm]?[jt]sx?$/.test(file))
        // Oxlint itself aborts on this TDZ cycle, even with an empty config.
        .filter(
          (file) => rule !== "no-import-only-test-files" || file !== "require-alias-cycle.test.js",
        )
        .sort();
      assert.ok(files.length > 0);
      const result = spawnSync(
        process.execPath,
        [
          resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
          "--type-aware",
          "--config",
          resolve(root, ".oxlintrc-static-followup.json"),
          "--format",
          "json",
          ...files.map((file) => resolve(root, file)),
        ],
        { encoding: "utf8", timeout: 4000, maxBuffer: 4 * 1024 * 1024 },
      );
      assert.equal(result.error, undefined, `${rule}: ${result.error}`);
      assert.equal(result.signal, null, `${rule}: ${result.stderr}`);
      const actual = JSON.parse(result.stdout).diagnostics.filter((item) =>
        item.code.includes(rule),
      );
      for (const file of files) {
        const expected = lint(
          fixture(`${rule}/${file}`),
          { [`no-mistakes/${rule}`]: "error" },
          resolve(root, file),
          {},
          /\.(cjs|cts)$/.test(file)
            ? "commonjs"
            : file === "captured-with.js"
              ? "script"
              : "module",
        );
        assert.ok(
          expected.every((item) => !item.fatal),
          `${rule}/${file}: invalid fixture mode`,
        );
        const lines = actual
          .filter((item) => resolve(item.filename) === resolve(root, file))
          .map((item) => item.labels[0].span.line)
          .sort((a, b) => a - b);
        assert.deepEqual(
          lines,
          expected.map((item) => item.line).sort((a, b) => a - b),
          `${rule}/${file}`,
        );
      }
      assert.equal(result.status, actual.length ? 1 : 0, `${rule}: ${result.stderr}`);
    });
  }
});
