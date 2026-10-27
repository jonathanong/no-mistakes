import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { dirname, resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { fixture, lint, require, __dirname } from "./helpers.mjs";
import {
  oxlintFindings,
  pluginFindings,
  snapshotCases,
  snapshotDocument,
} from "./diagnostic-snapshot.mjs";

const source = () => fixture("playwright-test-timeout-cap/fixture-runtime-positions.ts");
const rules = {
  "no-mistakes/playwright-test-timeout-cap": ["error", { unknownValues: "finding" }],
};
const findings = () => lint(source(), rules, "fixture.test.ts");
const rule = "playwright-test-timeout-cap";

describe("Playwright runtime fixture positions", () => {
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
  it("snapshots playwright runtime positions and matches oxlint", async () => {
    const code = fixture("playwright-test-timeout-cap/growing-project-array.ts");
    const delegated = {
      unknownValues: "finding",
      configUnknownValues: "defer-to-runner",
    };
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
    const actual = diagnostics.map((item) => [item.labels[0].span.line, item.message]);
    const expected = findings().map((item) => [item.line, item.message]);
    const sort = (rows) => rows.map((row) => JSON.stringify(row)).toSorted();
    assert.deepEqual(sort(actual), sort(expected));
    await expect(
      snapshotDocument({
        eslint: snapshotCases([
          {
            name: "fixture runtime positions",
            file: "fixture.test.ts",
            rule,
            options: { unknownValues: "finding" },
            findings: pluginFindings(
              source(),
              rule,
              { unknownValues: "finding" },
              "fixture.test.ts",
            ),
          },
          {
            name: "growing project array",
            file: "fixture.test.ts",
            rule,
            options: { unknownValues: "finding" },
            findings: pluginFindings(code, rule, { unknownValues: "finding" }, "fixture.test.ts"),
          },
          {
            name: "growing project array defer-to-runner",
            file: "fixture.test.ts",
            rule,
            options: delegated,
            findings: pluginFindings(code, rule, delegated, "fixture.test.ts"),
          },
        ]),
        oxlint: oxlintFindings(diagnostics, "fixture-runtime-positions.ts"),
      }),
    ).toMatchFileSnapshot("snapshots/playwright-fixture-runtime-positions.snap");
  });
});
