import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { expect, it } from "vitest";
import tsParser from "@typescript-eslint/parser";
import { eslintFindings, snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { fixture, lint, __dirname } from "./helpers.mjs";

const rule = "vitest-timeout-cap";
const finding = { unknownValues: "finding" };
const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};
const findings = (options = {}, name = "reasons") =>
  lint(
    fixture(`${rule}/${name}.ts`),
    { [`no-mistakes/${rule}`]: ["error", options] },
    "fixture.test.ts",
  );

function capCase(name, options = {}) {
  return {
    code: fixture(`${rule}/${name}.ts`),
    filename: "fixture.test.ts",
    languageOptions: { parser: tsParser },
    options: [options],
  };
}

const suite = {
  valid: [
    capCase("reason-adjacency", { requireReason: false }),
    capCase("reason-placeholders", { requireReason: false }),
  ],
  invalid: [
    // 17 reason, 1 timeout, 1 invalid. Local non-placeholder reasons keep diagnostic priority.
    capCase("reasons"),
    // 17 reason, 1 timeout, 1 invalid, 1 unknown.
    capCase("reasons", finding),
    // 1 timeout, 1 invalid. A higher defaultMax drops the reason findings.
    capCase("reasons", { defaultMax: 6000 }),
    // 17 reason, 2 timeout. Reasons stay local across hook contexts, spreads and defaults.
    capCase("reason-carriers"),
    // 9 reason. Blank comment lines and another call's syntax do not reuse a reason.
    capCase("reason-adjacency"),
    // 8 reason. Unfilled explanation and specific reason templates.
    capCase("reason-placeholders"),
    // Cap-only rollout retains cap, invalid-value and unknown-value diagnostics.
    capCase("reasons", { requireReason: false }),
    capCase("reasons", { requireReason: false, ...finding }),
    capCase("reason-carriers", { requireReason: false }),
  ],
};

it("snapshots vitest-timeout-reasons diagnostics", async () => {
  await expect(
    snapshotDocument({
      [rule]: snapshotSuite(rule, languageOptions, suite),
    }),
  ).toMatchFileSnapshot("snapshots/vitest-timeout-reasons.snap");
}, 30_000);

it("keeps reason enforcement enabled by default", async () => {
  assert.deepEqual(findings({ requireReason: true }), findings());
  await expect(
    snapshotDocument({
      default: eslintFindings(findings(), "reasons.ts"),
      capOnly: eslintFindings(findings({ requireReason: false }), "reasons.ts"),
    }),
  ).toMatchFileSnapshot("snapshots/vitest-timeout-require-reason.snap");
});

it("rejects non-boolean requireReason options", () => {
  for (const requireReason of [null, 0, 1, "false", {}, []])
    assert.throws(() => findings({ requireReason }), /should be boolean/u);
});
it("preserves reason and cap diagnostics through Oxlint for every carrier", () => {
  const root = resolve(__dirname, `../../../test-cases/eslint-snippets/fixture/${rule}`);
  for (const [options, config] of [
    [{}, ".oxlintrc.json"],
    [{ requireReason: false }, ".oxlintrc-cap-only.json"],
  ]) {
    for (const name of ["reasons", "reason-carriers", "reason-adjacency", "reason-placeholders"]) {
      const result = spawnSync(
        process.execPath,
        [
          resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
          "--config",
          resolve(root, config),
          "--format",
          "json",
          resolve(root, `${name}.ts`),
        ],
        { encoding: "utf8" },
      );
      assert.equal(result.status, findings(options, name).length ? 1 : 0, result.stderr);
      assert.deepEqual(
        JSON.parse(result.stdout)
          .diagnostics.filter((item) => item.code?.includes(rule))
          .map((item) => [item.labels[0].span.line, item.message])
          .sort((a, b) => a[0] - b[0]),
        findings(options, name).map(({ line, message }) => [line, message]),
      );
    }
  }
});
