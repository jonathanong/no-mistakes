import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import {
  oxlintFindings,
  pluginFindings,
  snapshotCases,
  snapshotDocument,
} from "./diagnostic-snapshot.mjs";
import { __dirname, fixture, plugin } from "./helpers.mjs";

const rule = "playwright-no-browser-new-page";
const fixtureRoot = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", rule);

function eslintCase(name, file, options, sourceType = "module") {
  return {
    name,
    file,
    rule,
    ...(options === undefined ? {} : { options }),
    ...(sourceType === "module" ? {} : { sourceType }),
    findings: pluginFindings(fixture(`${rule}/${file}`), rule, options, file, {}, sourceType),
  };
}

function oxlintOn(files) {
  return spawnSync(
    process.execPath,
    [
      resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
      "--config",
      resolve(fixtureRoot, ".oxlintrc.json"),
      "--format",
      "json",
      ...files.map((file) => resolve(fixtureRoot, file)),
    ],
    { encoding: "utf8" },
  );
}

describe(rule, () => {
  it("snapshots ESLint and Oxlint diagnostics", async () => {
    const oxlintFiles = [
      "direct-and-aliases.spec.ts",
      "allowed.spec.ts",
      "unanalyzable.spec.ts",
      "custom-options.spec.ts",
      "suppressions.spec.ts",
      "suppressed-file.spec.ts",
    ];
    const oxlintResult = oxlintOn(oxlintFiles);
    assert.equal(oxlintResult.status, 1, oxlintResult.stderr);
    assert.equal(oxlintResult.stdout.includes("Error running JS plugin"), false);
    const oxlintParsed = JSON.parse(oxlintResult.stdout).diagnostics.filter((entry) =>
      entry.code?.includes(rule),
    );
    const oxlintByFile = oxlintFiles.map((file) => ({
      name: file,
      file,
      rule,
      findings: oxlintFindings(
        oxlintParsed.filter((entry) => entry.filename?.endsWith(file)),
        file,
      ),
    }));

    const cases = [
      eslintCase("direct and aliased calls", "direct-and-aliases.spec.ts", {
        fixtureName: "appPage",
      }),
      eslintCase("allowed browser and non-Playwright usage", "allowed.spec.ts"),
      eslintCase("unanalyzable default finding mode", "unanalyzable.spec.ts"),
      eslintCase("unanalyzable ignore mode", "unanalyzable.spec.ts", {
        unanalyzable: "ignore",
      }),
      eslintCase("custom fixture and registration package", "custom-options.spec.ts", {
        fixtureName: "appPage",
        browserFixtureName: "customBrowser",
        registrationPackages: ["./fixtures"],
      }),
      eslintCase("custom diagnostic message", "custom-options.spec.ts", {
        registrationPackages: ["./fixtures"],
        message: "Use the shared appPage fixture from ./fixtures.",
      }),
      eslintCase("line and next-line suppressions", "suppressions.spec.ts"),
      eslintCase("file-level suppression", "suppressed-file.spec.ts"),
      eslintCase("commonjs require provenance", "commonjs.spec.cjs", undefined, "script"),
    ];

    expect(
      cases.find((item) => item.name === "allowed browser and non-Playwright usage").findings,
    ).toEqual([]);
    expect(cases.find((item) => item.name === "unanalyzable ignore mode").findings).toEqual([]);
    expect(cases.find((item) => item.name === "file-level suppression").findings).toEqual([]);
    expect(
      cases
        .find((item) => item.name === "line and next-line suppressions")
        .findings.map((f) => f.line),
    ).toEqual([7]);
    expect(
      cases.find((item) => item.name === "commonjs require provenance").findings.map((f) => f.line),
    ).toEqual([6, 10, 14, 18]);

    await expect(
      snapshotDocument({
        eslint: snapshotCases(cases),
        oxlint: snapshotCases(oxlintByFile),
      }),
    ).toMatchFileSnapshot("snapshots/playwright-no-browser-new-page.snap");
  });

  it("rejects invalid configuration options and stays out of presets", () => {
    const code = fixture(`${rule}/allowed.spec.ts`);
    for (const invalid of [
      { fixtureName: "" },
      { fixtureName: "   " },
      { browserFixtureName: "" },
      { registrationPackages: [""] },
      { registrationPackages: ["./fixtures", "./fixtures"] },
      { unanalyzable: "warn" },
      { message: "" },
      { unknownOption: true },
    ]) {
      assert.throws(() => pluginFindings(code, rule, invalid, "allowed.spec.ts"));
    }
    assert.ok(plugin.rules[rule]);
    assert.equal(plugin.configs.recommended.rules[`no-mistakes/${rule}`], undefined);
    assert.equal(plugin.configs.strict.rules[`no-mistakes/${rule}`], undefined);
  });
});
