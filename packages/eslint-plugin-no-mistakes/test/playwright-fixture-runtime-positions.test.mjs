import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { dirname, resolve } from "node:path";
import { describe, it } from "vitest";
import { fixture, lint, require, __dirname } from "./helpers.mjs";

const source = () => fixture("playwright-test-timeout-cap/fixture-runtime-positions.ts");
const rules = {
  "no-mistakes/playwright-test-timeout-cap": ["error", { unknownValues: "finding" }],
};
const findings = () => lint(source(), rules, "fixture.test.ts");

describe("Playwright runtime fixture positions", () => {
  it("checks the actual second slot with extra tails and preserved holes", () => {
    const rows = findings();
    for (const line of [4, 6, 9, 18, 19])
      assert.ok(rows.some((row) => row.line === line && row.messageId === "timeout"));
    for (const line of [5, 7]) assert.ok(!rows.some((row) => row.line === line));
  });
  it("retains unknown position, mutation, and opaque object-option findings", () => {
    const rows = findings();
    for (const line of [8, 15, 16, 17, 22, 23, 24, 25, 30, 33, 34, 38, 39])
      assert.ok(rows.some((row) => row.line === line && row.messageId === "unknown"));
  });
  it("recognizes primitive, missing, and function second slots as ordinary fixture data", () => {
    const rows = findings();
    for (const line of [10, 11, 12, 13, 14, 26, 27, 28, 29, 40])
      assert.ok(!rows.some((row) => row.line === line));
  });
  it("retains Vitest findings when primitive runtime facts refine an unknown middle option", () => {
    const rows = lint(
      fixture("vitest-timeout-cap/runtime-type-middle-options.ts"),
      { "no-mistakes/vitest-timeout-cap": ["error", { unknownValues: "finding" }] },
      "fixture.test.ts",
    );
    for (const line of [2, 3, 4])
      assert.ok(rows.some((row) => row.line === line && row.messageId === "unknown"));
  });
  it("does not mistake a mutated empty project array for a complete empty configuration", () => {
    const rows = lint(
      fixture("vitest-timeout-cap/growing-project-array.ts"),
      { "no-mistakes/vitest-timeout-cap": ["error", { unknownValues: "finding" }] },
      "fixture.test.ts",
    );
    assert.ok(rows.some((row) => row.messageId === "unknown"));
  });
  it("retains opaque Playwright project growth in configuration ownership checks", () => {
    const code = fixture("playwright-test-timeout-cap/growing-project-array.ts");
    const before = lint(code, rules, "fixture.test.ts");
    assert.ok(before.some((row) => row.line === 4 && row.messageId === "unknown"));
    assert.ok(before.some((row) => row.message.includes("projects carrier")));
    const delegated = lint(
      code,
      {
        "no-mistakes/playwright-test-timeout-cap": [
          "error",
          {
            unknownValues: "finding",
            configUnknownValues: "defer-to-runner",
          },
        ],
      },
      "fixture.test.ts",
    );
    assert.deepEqual(delegated, []);
  });
  it("preserves full located diagnostics in the public native CLI", () => {
    const packagePath = require.resolve("oxlint/package.json");
    const bin = resolve(dirname(packagePath), require(packagePath).bin.oxlint);
    const root = resolve(
      __dirname,
      "../../../test-cases/eslint-snippets/fixture/playwright-test-timeout-cap",
    );
    const run = spawnSync(
      process.execPath,
      [
        bin,
        "--config",
        resolve(root, ".oxlintrc.config-owner.json"),
        "--format",
        "json",
        resolve(root, "fixture-runtime-positions.ts"),
      ],
      { encoding: "utf8", timeout: 4000, killSignal: "SIGKILL" },
    );
    assert.equal(run.error, undefined);
    assert.equal(run.signal, null);
    assert.equal(run.status, 1, run.stderr);
    const { diagnostics } = JSON.parse(run.stdout);
    assert.ok(diagnostics.length > 0);
    assert.ok(
      diagnostics.every(
        (item) =>
          item.code === "no-mistakes(playwright-test-timeout-cap)" &&
          item.labels?.[0]?.span.line > 0,
      ),
    );
    const actual = diagnostics.map((item) => [item.labels[0].span.line, item.message]);
    const expected = findings().map((item) => [item.line, item.message]);
    const sort = (rows) => rows.map((row) => JSON.stringify(row)).toSorted();
    assert.deepEqual(sort(actual), sort(expected));
  });
});
