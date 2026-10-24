import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { fixture, lint, __dirname, plugin } from "./helpers.mjs";

const id = "postgres-no-manual-transaction";
const directory = "postgres-manual-sql-chain";
const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", directory);
const options = { importSpecifier: "@example/db" };
function check(name, extra = {}) {
  return lint(
    fixture(`${directory}/${name}`),
    { [`no-mistakes/${id}`]: ["error", { ...options, ...extra }] },
    resolve(root, name),
    {},
    name.endsWith(".cjs") ? "script" : "module",
  );
}

describe("manual transaction primitive const SQL chains", () => {
  it("finds literal/template chains through existing static executor forms", () => {
    const diagnostics = check("recall.ts");
    assert.deepEqual(
      diagnostics.map((m) => [m.line, m.messageId]),
      [
        [6, "manualTransaction"],
        [9, "manualTransaction"],
        [10, "manualTransaction"],
        [11, "manualTransaction"],
        [13, "manualTransaction"],
        [16, "manualTransaction"],
      ],
    );
    assert.ok(
      diagnostics.every((m) => m.message.includes("withTransaction / withTransactionOptions")),
    );
    assert.equal(plugin.rules[id].meta.fixable, undefined);
  });
  it("keeps uncertain values and cross-scope/dynamic reads unknown", () => {
    assert.deepEqual(check("controls.ts"), []);
    assert.deepEqual(check("dynamic.cjs"), []);
  });
  it("preserves released one-hop behavior", () => {
    assert.deepEqual(
      check("legacy.ts").map((m) => m.line),
      [4, 6, 7],
    );
  });
  it("honors suppression, explicit empty scope and owner files", () => {
    assert.deepEqual(
      check("suppressed.ts").map((m) => m.line),
      [7],
    );
    assert.deepEqual(check("file.ts"), []);
    assert.deepEqual(check("recall.ts", { importSpecifier: "", executorNames: [] }), []);
    assert.deepEqual(check("recall.ts", { owners: ["recall.ts"] }), []);
  });
  it("matches exact ESLint and real Oxlint diagnostics", () => {
    for (const name of [
      "recall.ts",
      "controls.ts",
      "legacy.ts",
      "suppressed.ts",
      "file.ts",
      "dynamic.cjs",
    ]) {
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
      const expected = check(name);
      assert.equal(result.status, expected.length ? 1 : 0, result.stderr);
      const actual = JSON.parse(result.stdout).diagnostics.filter((d) => d.code.includes(id));
      assert.deepEqual(
        actual
          .map((d) => [d.labels[0].span.line, d.labels[0].span.column, d.message])
          .sort((a, b) => a[0] - b[0]),
        expected.map((d) => [d.line, d.column, d.message]),
        name,
      );
    }
  });
});
