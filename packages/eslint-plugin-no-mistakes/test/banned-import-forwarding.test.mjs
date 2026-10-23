import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { fixture, lint, __dirname } from "./helpers.mjs";
const id = "no-banned-import-outside-allowed-paths";
const options = {
  checkedPathPatterns: ["**/*"],
  bannedImports: [{ module: "@acme/rate-limit", names: ["invalidateAll", "call", "apply"] }],
};
function check(name, overrides = {}) {
  return lint(
    fixture(`${id}/${name}.mjs`),
    {
      [`no-mistakes/${id}`]: ["error", { ...options, ...overrides }],
    },
    `${name}.mjs`,
  );
}
describe("banned imported capability forwarding", () => {
  it("reports direct, comma, static call/apply and aliased calls with binding controls", () => {
    assert.deepEqual(
      check("forwarded-calls").map((m) => [m.line, m.messageId]),
      [2, 3, 4, 5, 6, 7, 9, 10, 16, 17, 27, 28, 29].map((line) => [line, "bannedImport"]),
    );
  });
  it("retains path narrowing and file suppression", () => {
    assert.deepEqual(check("forwarded-calls", { allowedPathPatterns: ["**/*"] }), []);
    assert.deepEqual(check("forwarded-suppressed"), []);
  });
  it("has real Oxlint parity including suppression and literal exported call/apply names", () => {
    const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", id);
    for (const name of ["forwarded-calls", "forwarded-suppressed"]) {
      const expected = check(name);
      const result = spawnSync(
        process.execPath,
        [
          resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
          "--config",
          resolve(root, ".oxlintrc-forwarded.json"),
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
});
