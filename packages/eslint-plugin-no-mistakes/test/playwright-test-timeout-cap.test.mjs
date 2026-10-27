import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { fixture, lint, plugin, __dirname } from "./helpers.mjs";
import {
  oxlintFindings,
  pluginFindings,
  snapshotCases,
  snapshotDocument,
} from "./diagnostic-snapshot.mjs";

const rule = "playwright-test-timeout-cap";
const finding = { unknownValues: "finding" };
const tested = "fixture.test.ts";
const allFindings = (name, option = finding) =>
  lint(fixture(`${rule}/${name}.ts`), { [`no-mistakes/${rule}`]: ["error", option] }, tested);

function eslintCase(name, fixtureName, filename, options) {
  const setting = options === undefined ? undefined : { ...options };
  return {
    name,
    file: filename,
    rule,
    ...(setting === undefined ? {} : { options: setting }),
    findings: pluginFindings(fixture(`${rule}/${fixtureName}.ts`), rule, setting, filename),
  };
}

function oxlintOn(name) {
  const root = resolve(__dirname, `../../../test-cases/eslint-snippets/fixture/${rule}`);
  return spawnSync(
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
}

function ruleFindings(result, filename) {
  return oxlintFindings(
    JSON.parse(result.stdout).diagnostics.filter((item) => item.code?.includes(rule)),
    filename,
  );
}

describe(rule, () => {
  it("snapshots diagnostics", async () => {
    const computed = oxlintOn("null-computed-key.ts");
    const repeated = oxlintOn("null-computed-key.ts");
    for (const oxlint of [computed, repeated]) {
      assert.equal(oxlint.status, 1, oxlint.stderr);
      assert.equal(oxlint.stdout.includes("Error running JS plugin"), false);
    }
    const computedFindings = ruleFindings(computed, "null-computed-key.ts");
    assert.deepEqual(computedFindings, ruleFindings(repeated, "null-computed-key.ts"));
    const slow = oxlintOn("slow.ts");
    assert.equal(slow.status, 1);
    const eslintCases = [
      eslintCase("callable-timeout-sinks", "callable-timeout-sinks", tested, finding),
      eslintCase("callable-timeout-sinks empty option", "callable-timeout-sinks", tested, {}),
      eslintCase("carrier-safety", "carrier-safety", tested, finding),
      eslintCase("named-default-and-bound", "named-default-and-bound", tested, finding),
      eslintCase("default-and-named", "default-and-named", tested, finding),
      eslintCase("effective-registration", "effective-registration", tested, finding),
      eslintCase("effective-registration ignore unknown", "effective-registration", tested, {
        unknownValues: "ignore",
      }),
      eslintCase("helper-export-roles", "helper-export-roles", tested, {
        unknownValues: "finding",
        registrationPackages: ["./approved-test"],
        exportRoles: { "./approved-test": { expect: "assertion", ordinary: "ordinary" } },
      }),
      eslintCase("setters", "setters", tested, finding),
      eslintCase("slow", "slow", tested, finding),
      eslintCase("suite-slow", "suite-slow", tested, finding),
      eslintCase("carriers", "carriers", tested, finding),
      eslintCase("controls", "controls", tested, finding),
      eslintCase("suppression", "suppression", tested, finding),
      eslintCase("slow ignore unknown", "slow", tested, { unknownValues: "ignore" }),
      eslintCase("namespace", "namespace", tested, finding),
      eslintCase("config-root", "config-root", tested, finding),
      eslintCase("config-root config files", "config-root", tested, {
        unknownValues: "finding",
        configFiles: ["fixture.test.ts"],
      }),
      eslintCase("opaque-spread", "opaque-spread", tested, finding),
      eslintCase("suppression-file", "suppression-file", tested, finding),
      eslintCase("carriers fixture max", "carriers", tested, {
        unknownValues: "finding",
        fixtureMax: 60000,
      }),
      eslintCase("inherited-slow", "inherited-slow", tested, finding),
      eslintCase("approved-helper", "approved-helper", tested, finding),
      eslintCase("approved-helper registration packages", "approved-helper", tested, {
        unknownValues: "finding",
        registrationPackages: ["./approved-test"],
      }),
      eslintCase("helper-carriers", "helper-carriers", tested, {
        unknownValues: "finding",
        registrationPackages: ["./approved-test"],
      }),
      eslintCase("null-computed-key", "null-computed-key", "null-computed-key.ts"),
      eslintCase(
        "null-computed-key unknown values",
        "null-computed-key",
        "null-computed-key.ts",
        finding,
      ),
    ];
    await expect(
      snapshotDocument({
        eslint: snapshotCases(eslintCases),
        oxlint: snapshotCases([
          {
            name: "null-computed-key",
            file: "null-computed-key.ts",
            rule,
            findings: computedFindings,
          },
          {
            name: "slow",
            file: "slow.ts",
            rule,
            findings: ruleFindings(slow, "slow.ts"),
          },
        ]),
      }),
    ).toMatchFileSnapshot("snapshots/playwright-test-timeout-cap.snap");
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

  it("rejects invalid caps and stays out of presets", () => {
    for (const max of [0, -1, Infinity, NaN]) assert.throws(() => allFindings("controls", { max }));
    assert.ok(!plugin.configs.strict.rules[`no-mistakes/${rule}`]);
    assert.ok(!plugin.configs.recommended.rules[`no-mistakes/${rule}`]);
  });
});
