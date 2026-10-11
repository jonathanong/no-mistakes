import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { fixture, lint, plugin, __dirname } from "./helpers.mjs";
const rule = "playwright-test-timeout-cap";
const allFindings = (name, option = { unknownValues: "finding" }) =>
  lint(
    fixture(`${rule}/${name}.ts`),
    { [`no-mistakes/${rule}`]: ["error", option] },
    "fixture.test.ts",
  );
// Existing assertions isolate explicit carrier/slot diagnostics; inherited registration
// diagnostics are asserted independently below, not treated as qualified owner proofs.
const findings = (name, option) =>
  allFindings(name, option).filter((x) => x.messageId !== "effectiveSlot");
const positions = (name) => findings(name).map(({ line, messageId }) => [line, messageId]);
describe(rule, () => {
  it("keeps opaque project/fixture carriers visible and false conditions inert", () => {
    assert.deepEqual(positions("carrier-safety"), [
      [2, "unknown"],
      [3, "unknown"],
      [3, "unknown"],
      [4, "unknown"],
      [8, "unknown"],
      [11, "unknown"],
      [12, "unknown"],
    ]);
  });
  it("normalizes named SDK defaults and retains receiver-only bound callback parameters", () => {
    assert.deepEqual(positions("named-default-and-bound"), [
      [3, "timeout"],
      [5, "timeout"],
      [9, "unknown"],
      [10, "unknown"],
    ]);
    assert.deepEqual(
      allFindings("named-default-and-bound")
        .filter((x) => x.messageId === "effectiveSlot")
        .map((x) => x.line),
      [7, 9, 10, 12],
    );
  });
  it("retains SDK default identities and registration-bound named callback TestInfo", () => {
    assert.deepEqual(positions("default-and-named"), [
      [4, "timeout"],
      [5, "invalid"],
      [7, "timeout"],
      [10, "invalid"],
    ]);
    assert.deepEqual(
      allFindings("default-and-named")
        .filter((x) => x.messageId === "effectiveSlot")
        .map((x) => x.line),
      [9, 11, 13],
    );
  });
  it("reports every admitted registration's unresolved inherited slot", () => {
    assert.deepEqual(
      allFindings("effective-registration").map(({ line, messageId }) => [line, messageId]),
      [
        [2, "effectiveSlot"],
        [3, "effectiveSlot"],
        [4, "effectiveSlot"],
        [5, "unknown"],
      ],
    );
    assert.deepEqual(allFindings("effective-registration", { unknownValues: "ignore" }), []);
  });
  it("uses reviewed helper export roles without admitting assertions as registrars", () => {
    const option = {
      unknownValues: "finding",
      registrationPackages: ["./approved-test"],
      exportRoles: { "./approved-test": { expect: "assertion", ordinary: "ordinary" } },
    };
    assert.deepEqual(
      allFindings("helper-export-roles", option).map(({ line, messageId }) => [line, messageId]),
      [
        [5, "unknown"],
        [6, "effectiveSlot"],
        [7, "unknown"],
      ],
    );
  });
  it("rejects role configuration that erases proven SDK or helper registrar exports", () => {
    for (const source of ["@playwright/test", "./approved-test"])
      for (const key of ["test", "defineConfig", "mergeTests"])
        for (const role of ["ordinary", "assertion", "opaque"])
          assert.throws(
            () =>
              allFindings("helper-export-roles", {
                unknownValues: "finding",
                registrationPackages: ["./approved-test"],
                exportRoles: { [source]: { [key]: role } },
              }),
            /cannot override/,
          );
    assert.throws(
      () =>
        allFindings("effective-registration", {
          exportRoles: { "@playwright/test": { expect: "ordinary" } },
        }),
      /builtin SDK exports/,
    );
  });
  it("follows aliases, bound/destructured setters and real hook TestInfo parameters", () => {
    assert.deepEqual(positions("setters"), [
      [4, "timeout"],
      [6, "invalid"],
      [8, "invalid"],
      [12, "timeout"],
      [15, "invalid"],
      [20, "invalid"],
      [21, "timeout"],
      [22, "invalid"],
    ]);
  });
  it("models the SDK's idempotent slow latch and preserves false/short-slot semantics", () => {
    assert.deepEqual(positions("slow"), [
      [11, "timeout"],
      [14, "unknown"],
      [17, "unknown"],
      [22, "unknown"],
      [24, "unknown"],
    ]);
    assert.deepEqual(positions("suite-slow"), [
      [2, "unknown"],
      [5, "unknown"],
    ]);
  });
  it("reports opaque admitted config/suite/fixture/API carriers rather than fixture payload data", () => {
    assert.deepEqual(positions("carriers"), [
      [3, "invalid"],
      [3, "unknown"],
      [4, "timeout"],
      [5, "unknown"],
      [7, "timeout"],
      [8, "unknown"],
      [9, "unknown"],
    ]);
  });
  it("ignores ordinary/shadowed objects and honors positive finite cap boundaries", () => {
    assert.deepEqual(findings("controls"), []);
    assert.deepEqual(positions("suppression"), [[4, "invalid"]]);
    assert.deepEqual(
      findings("slow", { unknownValues: "ignore" }).map((x) => x.line),
      [11],
    );
    for (const max of [0, -1, Infinity, NaN]) assert.throws(() => findings("controls", { max }));
    assert.ok(!plugin.configs.strict.rules[`no-mistakes/${rule}`]);
    assert.ok(!plugin.configs.recommended.rules[`no-mistakes/${rule}`]);
  });
  it("handles namespace/extended/merged registrars, false aliases and configured plain roots", () => {
    assert.deepEqual(positions("namespace"), [
      [4, "timeout"],
      [7, "invalid"],
    ]);
    assert.deepEqual(findings("config-root"), []);
    assert.deepEqual(
      findings("config-root", { unknownValues: "finding", configFiles: ["fixture.test.ts"] }).map(
        ({ line, messageId }) => [line, messageId],
      ),
      [[2, "timeout"]],
    );
    assert.deepEqual(positions("opaque-spread"), [[2, "unknown"]]);
    assert.deepEqual(findings("suppression-file"), []);
    assert.deepEqual(
      findings("carriers", { unknownValues: "finding", fixtureMax: 60000 })
        .filter((x) => x.messageId === "timeout")
        .map((x) => x.line),
      [4],
    );
  });
  it("does not claim multiplication occurred when an inherited hook latch is unresolved", () => {
    assert.deepEqual(positions("inherited-slow"), [
      [2, "unknown"],
      [5, "unknown"],
    ]);
  });
  it("requires explicit helper admission and retains unknown effective helper slots", () => {
    assert.deepEqual(findings("approved-helper"), []);
    assert.deepEqual(
      findings("approved-helper", {
        unknownValues: "finding",
        registrationPackages: ["./approved-test"],
      }).map(({ line, messageId }) => [line, messageId]),
      [
        [2, "timeout"],
        [4, "unknown"],
      ],
    );
  });
  it("reports admitted unresolved helper/merge/fixture carriers instead of losing their provenance", () => {
    assert.deepEqual(
      findings("helper-carriers", {
        unknownValues: "finding",
        registrationPackages: ["./approved-test"],
      }).map(({ line, messageId }) => [line, messageId]),
      [
        [3, "unknown"],
        [4, "unknown"],
        [5, "unknown"],
        [6, "unknown"],
      ],
    );
  });
  it("retains exact locations through Oxlint", () => {
    const root = resolve(__dirname, `../../../test-cases/eslint-snippets/fixture/${rule}`);
    const run = spawnSync(
      process.execPath,
      [
        resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
        "--config",
        resolve(root, ".oxlintrc.json"),
        "--format",
        "json",
        resolve(root, "slow.ts"),
      ],
      { encoding: "utf8" },
    );
    assert.equal(run.status, 1);
    assert.deepEqual(
      JSON.parse(run.stdout)
        .diagnostics.filter(
          (x) => x.code?.includes(rule) && !x.message.includes("registration inherits"),
        )
        .map((x) => x.labels[0].span.line)
        .sort((a, b) => a - b),
      [11, 14, 17, 22, 24],
    );
  });
});
