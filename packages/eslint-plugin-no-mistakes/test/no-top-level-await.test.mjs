import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { fixture, lint, plugin, __dirname } from "./helpers.mjs";
import { pluginFindings, snapshotCases, snapshotDocument } from "./diagnostic-snapshot.mjs";

const id = "no-top-level-await";
const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", id);
function check(name, options) {
  return lint(
    fixture(`${id}/${name}`),
    {
      [`no-mistakes/${id}`]: options ? ["error", options] : "error",
    },
    resolve(root, name),
  );
}
describe(id, () => {
  it("reports module suspension, including class evaluation and loop sources", () => {
    assert.deepEqual(
      check("unsafe.mjs").map((m) => m.line),
      [1, 2, 3, 4, 5, 6, 8],
    );
    assert.deepEqual(check("safe.mjs"), []);
    assert.deepEqual(
      check("wrapped.ts").map((m) => m.line),
      [2],
    );
    assert.deepEqual(
      check("await-using.ts").map((m) => m.line),
      [1, 3],
    );
  });
  it("allows only direct main-guard consequents, with an explicit opt-out", () => {
    assert.deepEqual(
      check("guards.mjs").map((m) => m.line),
      [1, 2, 3, 4, 5, 7, 7, 8],
    );
    assert.deepEqual(
      check("safe.mjs", { allowInImportMetaMain: false }).map((m) => m.line),
      [7, 8],
    );
    assert.deepEqual(
      check("wrapped.ts", { allowInImportMetaMain: false }).map((m) => m.line),
      [1, 2],
    );
    assert.deepEqual(
      check("await-using.ts", { allowInImportMetaMain: false }).map((m) => m.line),
      [1, 3, 10],
    );
  });
  it("supports actual-comment suppressions and a custom message", () => {
    assert.deepEqual(
      check("suppressions.mjs").map((m) => m.line),
      [5],
    );
    assert.deepEqual(check("suppression-file.mjs"), []);
    assert.equal(
      check("unsafe.mjs", { message: "Initialize in main" })[0].message,
      "Initialize in main",
    );
    assert.throws(() => check("unsafe.mjs", { message: "   " }), /should match pattern/);
  });
  it("is opt-in and has no automatic rewrite", () => {
    assert.equal(plugin.configs.recommended.rules[`no-mistakes/${id}`], undefined);
    assert.equal(plugin.configs.strict.rules[`no-mistakes/${id}`], undefined);
    assert.equal(plugin.rules[id].meta.fixable, undefined);
  });
  it("pins fixture diagnostics", async () => {
    const rows = [
      ...[
        "unsafe.mjs",
        "safe.mjs",
        "guards.mjs",
        "wrapped.ts",
        "await-using.ts",
        "suppressions.mjs",
        "suppression-file.mjs",
      ].map((name) => ({ name, file: name })),
      { name: "strict guard", file: "safe.mjs", options: { allowInImportMetaMain: false } },
      {
        name: "strict guard await using",
        file: "await-using.ts",
        options: { allowInImportMetaMain: false },
      },
      { name: "custom message", file: "unsafe.mjs", options: { message: "Initialize in main" } },
    ].map((item) => ({
      ...item,
      findings: pluginFindings(fixture(`${id}/${item.file}`), id, item.options, item.file),
    }));
    await expect(snapshotDocument(snapshotCases(rows))).toMatchFileSnapshot(
      "./snapshots/no-top-level-await.snap",
    );
  });
  for (const [name, count] of [
    ["unsafe.mjs", 7],
    ["safe.mjs", 0],
    ["guards.mjs", 8],
    ["wrapped.ts", 1],
    ["await-using.ts", 2],
    ["suppressions.mjs", 1],
    ["suppression-file.mjs", 0],
  ]) {
    it(`has actual Oxlint parity for ${name}`, () => {
      const result = spawnSync(
        process.execPath,
        [
          resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
          "--config",
          resolve(root, ".oxlintrc.json"),
          "--format",
          "json",
          resolve(root, name),
        ],
        { encoding: "utf8" },
      );
      assert.equal(result.status, count ? 1 : 0, result.stderr);
      const diagnostics = JSON.parse(result.stdout).diagnostics.filter((d) => d.code.includes(id));
      assert.equal(diagnostics.length, count);
    });
  }
});
