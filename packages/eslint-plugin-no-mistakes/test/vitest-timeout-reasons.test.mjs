import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { it } from "vitest";
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
it("preserves reason and cap diagnostics through Oxlint for every carrier", () => {
  const root = resolve(__dirname, `../../../test-cases/eslint-snippets/fixture/${rule}`);
  for (const name of ["reasons", "reason-carriers", "reason-adjacency"]) {
    const result = spawnSync(
      process.execPath,
      [
        resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
        "--config",
        resolve(root, ".oxlintrc.json"),
        "--format",
        "json",
        resolve(root, `${name}.ts`),
      ],
      { encoding: "utf8" },
    );
    assert.equal(result.status, 1, result.stderr);
    assert.deepEqual(
      JSON.parse(result.stdout)
        .diagnostics.filter((item) => item.code?.includes(rule))
        .map((item) => [item.labels[0].span.line, item.message])
        .sort((a, b) => a[0] - b[0]),
      findings({}, name).map(({ line, message }) => [line, message]),
    );
  }
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

it("does not reuse reasons across blank comment lines or intervening grouping syntax", () => {
  assert.deepEqual(
    findings({}, "reason-adjacency").map(({ line, messageId }) => [line, messageId]),
    [
      [6, "reason"],
      [11, "reason"],
    ],
  );
});
