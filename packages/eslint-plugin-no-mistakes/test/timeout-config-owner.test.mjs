import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { dirname, resolve } from "node:path";
import { describe, it } from "vitest";
import { fixture, lint, require, __dirname } from "./helpers.mjs";

const options = { unknownValues: "finding", defaultMax: 30000, overrideMax: 30000 };
function findings(framework, file, configUnknownValues) {
  const rule = `${framework === "vitest" ? "vitest" : "playwright-test"}-timeout-cap`;
  const cap = framework === "vitest" ? options : { unknownValues: "finding" };
  const configured = configUnknownValues === undefined ? cap : { ...cap, configUnknownValues };
  return lint(
    fixture(`${rule}/${file}.ts`),
    { [`no-mistakes/${rule}`]: ["error", configured] },
    "fixture.test.ts",
  );
}
const positions = (rows) => rows.map(({ line, messageId }) => [line, messageId]);

describe("configuration timeout owner delegation", () => {
  for (const framework of ["vitest", "playwright"]) {
    it(`${framework} preserves default findings under explicit inheritance`, () => {
      for (const name of ["config-owner-mixed", "config-owner-known"])
        assert.deepEqual(findings(framework, name, "inherit"), findings(framework, name));
    });
  }
  it("delegates only unresolved Vitest configuration roles in a mixed module", () => {
    const before = findings("vitest", "config-owner-mixed");
    const after = findings("vitest", "config-owner-mixed", "defer-to-runner");
    assert.ok(before.some(({ line }) => line === 4));
    assert.ok(!after.some(({ line }) => line === 4));
    assert.deepEqual(positions(after), [
      [5, "unknown"],
      [6, "unknown"],
      [7, "unknown"],
      [8, "unknown"],
      [9, "timeout"],
    ]);
    assert.deepEqual(
      after,
      before.filter(({ line }) => line !== 4),
    );
  });
  it("retains known illegal Vitest config values and unknown framework calls", () => {
    const before = findings("vitest", "config-owner-known");
    const after = findings("vitest", "config-owner-known", "defer-to-runner");
    assert.deepEqual(after, before);
    assert.ok(after.some(({ line, messageId }) => line === 4 && messageId === "timeout"));
    assert.ok(after.some(({ line, messageId }) => line === 4 && messageId === "invalid"));
    assert.ok(after.some(({ line, messageId }) => line === 5 && messageId === "invalid"));
    assert.ok(after.some(({ line, messageId }) => line === 5 && messageId === "timeout"));
    assert.ok(after.some(({ line, messageId }) => line === 6 && messageId === "unknown"));
    assert.ok(after.some(({ line, messageId }) => line === 8 && messageId === "unknown"));
  });
  it("retains Playwright fixture, suite, setter and inherited-slot findings", () => {
    const before = findings("playwright", "config-owner-mixed");
    const after = findings("playwright", "config-owner-mixed", "defer-to-runner");
    assert.ok(before.some(({ line }) => line === 3));
    assert.ok(!after.some(({ line }) => line === 3));
    assert.deepEqual(
      after,
      before.filter(({ line }) => line !== 3),
    );
    assert.ok(after.some(({ line, messageId }) => line === 4 && messageId === "unknown"));
    for (const line of [5, 6, 7]) assert.ok(after.some((finding) => finding.line === line));
    assert.ok(after.some(({ line, messageId }) => line === 7 && messageId === "effectiveSlot"));
  });
  it("delegates opaque Playwright config merging without hiding known or mixed-origin risks", () => {
    const before = findings("playwright", "config-owner-known");
    const after = findings("playwright", "config-owner-known", "defer-to-runner");
    for (const line of [3, 8]) assert.ok(before.some((finding) => finding.line === line));
    assert.deepEqual(
      after,
      before.filter(({ line }) => line !== 3 && line !== 8),
    );
    assert.ok(after.some(({ line, messageId }) => line === 4 && messageId === "timeout"));
    assert.equal(
      after.filter(({ line, messageId }) => line === 4 && messageId === "invalid").length,
      2,
    );
    assert.ok(after.some(({ line, messageId }) => line === 5 && messageId === "invalid"));
    assert.ok(after.some(({ line, messageId }) => line === 6 && messageId === "unknown"));
    assert.ok(after.some(({ line, messageId }) => line === 7 && messageId === "unknown"));
  });
  for (const framework of ["vitest", "playwright"]) {
    it(`${framework} retains located native findings without plugin exceptions`, () => {
      const started = process.hrtime.bigint();
      const rule = `${framework === "vitest" ? "vitest" : "playwright-test"}-timeout-cap`;
      const packagePath = require.resolve("oxlint/package.json");
      const bin = resolve(dirname(packagePath), require(packagePath).bin.oxlint);
      const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", rule);
      for (const name of ["config-owner-mixed", "config-owner-known"]) {
        const remaining = 4000 - Math.ceil(Number(process.hrtime.bigint() - started) / 1_000_000);
        assert.ok(remaining > 0, "native controls exhausted their shared case budget");
        const run = spawnSync(
          process.execPath,
          [
            bin,
            "--config",
            resolve(root, ".oxlintrc.config-owner.json"),
            "--format",
            "json",
            resolve(root, `${name}.ts`),
          ],
          { encoding: "utf8", timeout: remaining, killSignal: "SIGKILL" },
        );
        assert.equal(run.error, undefined);
        assert.equal(run.signal, null);
        assert.equal(run.status, 1, run.stderr);
        const { diagnostics } = JSON.parse(run.stdout);
        assert.ok(diagnostics.length > 0);
        assert.ok(
          diagnostics.every(
            (item) => item.code === `no-mistakes(${rule})` && item.labels?.[0]?.span.line > 0,
          ),
          run.stdout,
        );
        const actual = diagnostics.map((item) => [item.labels[0].span.line, item.message]);
        const expected = findings(framework, name, "defer-to-runner").map((item) => [
          item.line,
          item.message,
        ]);
        const sort = (rows) => rows.map((row) => JSON.stringify(row)).toSorted();
        assert.deepEqual(sort(actual), sort(expected));
      }
    });
  }
});
