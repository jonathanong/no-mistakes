import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { it, expect } from "vitest";
import { eslintFindings, snapshotDocument } from "./diagnostic-snapshot.mjs";
import { fixture, lint, __dirname } from "./helpers.mjs";
const rule = "vitest-timeout-cap";
const expected = [2, 6, 8, 10, 12, 14, 16, 24, 24, 26, 28, 31, 35, 37, 40, 55, 69];
const findings = (options = {}, name = "reasons") =>
  lint(
    fixture(`${rule}/${name}.ts`),
    {
      [`no-mistakes/${rule}`]: ["error", options],
    },
    "fixture.test.ts",
  );
it("requires local non-placeholder reasons while retaining diagnostic priorities", () => {
  assert.deepEqual(
    findings()
      .filter((item) => item.messageId === "reason")
      .map((item) => item.line),
    expected,
  );
  assert.deepEqual(
    findings()
      .filter((item) => item.messageId !== "reason")
      .map(({ line, messageId }) => [line, messageId]),
    [
      [42, "timeout"],
      [44, "invalid"],
    ],
  );
  assert.deepEqual(
    findings({ unknownValues: "finding" })
      .filter((item) => item.messageId === "unknown")
      .map((item) => item.line),
    [47],
  );
  assert.deepEqual(
    findings({ defaultMax: 6000 }).filter((item) => item.messageId === "reason"),
    [],
  );
});
it("allows cap-only rollout without changing caps, invalid values or unknown policy", async () => {
  assert.deepEqual(findings({ requireReason: true }), findings());
  const capOnly = { requireReason: false };
  assert.deepEqual(
    findings(capOnly).map(({ line, messageId }) => [line, messageId]),
    [
      [42, "timeout"],
      [44, "invalid"],
    ],
  );
  assert.deepEqual(
    findings({ ...capOnly, unknownValues: "finding" }).map(({ line, messageId }) => [
      line,
      messageId,
    ]),
    [
      [42, "timeout"],
      [44, "invalid"],
      [47, "unknown"],
    ],
  );
  assert.deepEqual(
    findings(capOnly, "reason-carriers").map(({ line, messageId }) => [line, messageId]),
    [
      [32, "timeout"],
      [34, "timeout"],
    ],
  );
  for (const name of ["reason-adjacency", "reason-placeholders"])
    assert.deepEqual(findings(capOnly, name), []);
  await expect(
    snapshotDocument({
      default: eslintFindings(findings(), "reasons.ts"),
      capOnly: eslintFindings(findings(capOnly), "reasons.ts"),
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

it("rejects unfilled explanation and specific reason templates", () => {
  assert.deepEqual(
    findings({}, "reason-placeholders").map(({ line, messageId }) => [line, messageId]),
    [4, 6, 8, 10, 12, 14, 16, 18].map((line) => [line, "reason"]),
  );
});

it("keeps reasons local across hook contexts, effective spreads and config defaults", () => {
  assert.deepEqual(
    findings({}, "reason-carriers").map(({ line, messageId }) => [line, messageId]),
    [
      ...[3, 4, 5, 6, 7, 8, 22, 23].map((line) => [line, "reason"]),
      [32, "timeout"],
      [34, "timeout"],
      ...[36, 43, 47, 58, 60, 62, 64, 66, 68].map((line) => [line, "reason"]),
    ],
  );
});

it("does not reuse reasons across blank comment lines or another call's syntax", () => {
  assert.deepEqual(
    findings({}, "reason-adjacency").map(({ line, messageId }) => [line, messageId]),
    [6, 19, 21, 23, 26, 30, 47, 50, 53].map((line) => [line, "reason"]),
  );
});
