import { spawnSync } from "node:child_process";
import { dirname, resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { describe, expect, it } from "vitest";
import {
  languageOptionsFindings,
  oxlintFindings,
  snapshotCases,
  snapshotDocument,
} from "./diagnostic-snapshot.mjs";
import { __dirname, fixture, getLinter, linterCwd, oxlintBin, plugin } from "./helpers.mjs";

const rule = "test-no-unfailable-assertion";
const fixtureDir = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", rule);

function languageOptionsFor(filename, { typed = true, sourceType = "module" } = {}) {
  const isTypeScript = /\.[cm]?tsx?$/.test(filename);
  return {
    ecmaVersion: 2024,
    sourceType,
    ...(isTypeScript ? { parser: tsParser } : {}),
    parserOptions:
      typed && isTypeScript ? { projectService: true, tsconfigRootDir: dirname(filename) } : {},
  };
}

function runFixture(name, { typed = true, options, sourceType = "module" } = {}) {
  const filename = resolve(fixtureDir, name);
  return languageOptionsFindings({
    code: fixture(`${rule}/${name}`),
    rule,
    options: options === undefined ? undefined : [options],
    filename,
    languageOptions: languageOptionsFor(filename, { typed, sourceType }),
  });
}

function fixFixture(name, { typed = false, options, sourceType = "module" } = {}) {
  const filename = resolve(fixtureDir, name);
  const linter = getLinter(linterCwd(filename));
  const setting = options === undefined ? "error" : ["error", options];
  return linter.verifyAndFix(
    fixture(`${rule}/${name}`),
    {
      files: ["**"],
      languageOptions: languageOptionsFor(filename, { typed, sourceType }),
      plugins: { "no-mistakes": plugin },
      rules: { [`no-mistakes/${rule}`]: setting },
    },
    filename,
  );
}

describe(rule, () => {
  it("snapshots typed and untyped diagnostics across all six codes", async () => {
    const cases = [
      { name: "defined-by-type (typed)", file: "defined-by-type.test.ts", typed: true },
      { name: "defined-by-type (untyped)", file: "defined-by-type.test.ts", typed: false },
      { name: "same-value (typed)", file: "same-value.test.ts", typed: true },
      { name: "same-value (untyped)", file: "same-value.test.ts", typed: false },
      { name: "truthy-by-type (typed)", file: "truthy-by-type.test.ts", typed: true },
      { name: "truthy-by-type (untyped)", file: "truthy-by-type.test.ts", typed: false },
      { name: "throwing-query (typed)", file: "throwing-query.test.ts", typed: true },
      { name: "throwing-query (untyped)", file: "throwing-query.test.ts", typed: false },
      { name: "discarded-locator (typed)", file: "discarded-locator.test.ts", typed: true },
      { name: "discarded-locator (untyped)", file: "discarded-locator.test.ts", typed: false },
      { name: "defined-then-deref (typed)", file: "defined-then-deref.test.ts", typed: true },
      { name: "defined-then-deref (untyped)", file: "defined-then-deref.test.ts", typed: false },
      { name: "suppressions (typed)", file: "suppressions.test.ts", typed: true },
      { name: "suppressed-file (typed)", file: "suppressed-file.test.ts", typed: true },
      {
        name: "commonjs and switch cases (untyped)",
        file: "commonjs.test.cjs",
        typed: false,
        sourceType: "commonjs",
      },
      {
        name: "severities promotes throwing-query to error",
        file: "throwing-query.test.ts",
        typed: true,
        options: { severities: { "throwing-query": "error" }, report: "error" },
      },
      {
        name: "severities disables throwing-query so defined-by-type catches it",
        file: "throwing-query.test.ts",
        typed: true,
        options: { severities: { "throwing-query": "off" } },
      },
      {
        name: "report advisory omits error-severity codes",
        file: "same-value.test.ts",
        typed: true,
        options: { report: "advisory" },
      },
    ].map(({ name, file, typed, options, sourceType }) => ({
      name,
      file,
      rule,
      options,
      ...(sourceType ? { sourceType } : {}),
      findings: runFixture(file, { typed, options, sourceType }),
    }));

    const byName = Object.fromEntries(cases.map((item) => [item.name, item.findings]));
    expect(byName["defined-by-type (typed)"].map((f) => f.messageId)).toEqual(
      Array(7).fill("defined-by-type"),
    );
    expect(byName["defined-by-type (untyped)"]).toEqual([]);
    expect(byName["same-value (typed)"]).toEqual(byName["same-value (untyped)"]);
    expect(byName["same-value (typed)"].map((f) => f.messageId)).toEqual(
      Array(13).fill("same-value"),
    );
    expect(byName["truthy-by-type (typed)"].map((f) => f.messageId)).toEqual(
      Array(18).fill("truthy-by-type"),
    );
    expect(byName["truthy-by-type (untyped)"]).toEqual([]);
    expect(byName["throwing-query (typed)"]).toEqual(byName["throwing-query (untyped)"]);
    expect(byName["throwing-query (typed)"].map((f) => f.messageId)).toEqual(
      Array(27).fill("throwing-query"),
    );
    expect(byName["throwing-query (typed)"][0].message.startsWith("[advisory] ")).toBe(true);
    expect(
      byName["severities promotes throwing-query to error"][0].message.startsWith("[advisory] "),
    ).toBe(false);
    expect(byName["discarded-locator (typed)"].map((f) => f.messageId)).toEqual(
      Array(8).fill("discarded-locator"),
    );
    expect(byName["discarded-locator (untyped)"]).toEqual([]);
    expect(byName["defined-then-deref (typed)"]).toEqual(byName["defined-then-deref (untyped)"]);
    expect(byName["defined-then-deref (typed)"].map((f) => f.messageId)).toEqual(
      Array(8).fill("defined-then-deref"),
    );
    expect(byName["suppressions (typed)"].map((f) => f.line)).toEqual([12, 15]);
    expect(byName["suppressed-file (typed)"]).toEqual([]);
    expect(byName["report advisory omits error-severity codes"]).toEqual([]);

    await expect(snapshotDocument(snapshotCases(cases))).toMatchFileSnapshot(
      "snapshots/test-no-unfailable-assertion.snap",
    );
  });

  it("snapshots throwing-query autofixes with and without jest-dom, async, and non-async cases", async () => {
    const drop = fixFixture("autofix-drop-wrapper.test.ts");
    const directImport = fixFixture("autofix-jest-dom-import.test.ts");
    const cjsRequire = fixFixture("autofix-jest-dom-require.test.cjs", { sourceType: "commonjs" });
    const setupFile = fixFixture("jest-dom-setup/setup-fixture.test.ts");
    const pkgSetup = fixFixture("jest-dom-pkg/pkg-setup.test.ts");
    const typedSetup = fixFixture("jest-dom-typed/typed-jest-dom.test.ts", { typed: true });

    expect(drop.fixed).toBe(true);
    expect(drop.messages.map((m) => [m.line, m.messageId])).toEqual([
      [21, "throwing-query"],
      [25, "throwing-query"],
      [26, "throwing-query"],
      [27, "throwing-query"],
      [30, "throwing-query"],
    ]);

    expect(directImport.fixed).toBe(true);
    expect(directImport.messages.map((m) => [m.line, m.messageId])).toEqual([
      [27, "throwing-query"],
      [28, "throwing-query"],
    ]);

    expect(cjsRequire.fixed).toBe(true);
    expect(cjsRequire.messages).toEqual([]);
    expect(setupFile.fixed).toBe(true);
    expect(setupFile.messages).toEqual([]);
    expect(pkgSetup.fixed).toBe(true);
    expect(pkgSetup.messages).toEqual([]);
    expect(typedSetup.fixed).toBe(true);
    expect(typedSetup.messages).toEqual([]);

    await expect(
      snapshotDocument({
        dropWrapper: { output: drop.output, remainingLines: drop.messages.map((m) => m.line) },
        jestDomImport: {
          output: directImport.output,
          remainingLines: directImport.messages.map((m) => m.line),
        },
        jestDomRequire: { output: cjsRequire.output },
        jestDomSetupFile: { output: setupFile.output },
        jestDomPackageJson: { output: pkgSetup.output },
        jestDomTypeSetup: { output: typedSetup.output },
      }),
    ).toMatchFileSnapshot("snapshots/test-no-unfailable-assertion-autofix.snap");
  });

  it("skips type-based checks when noResolve, strictNullChecks=false, or TS node map misses", () => {
    const filename = resolve(fixtureDir, "defined-by-type.test.ts");
    const code = fixture(`${rule}/defined-by-type.test.ts`);
    const makeParser = (transform) => ({
      parseForESLint(source, parserOptions) {
        const parsed = tsParser.parseForESLint(source, parserOptions);
        return { ...parsed, services: transform(parsed.services) };
      },
    });
    const variants = [
      (services) => ({
        ...services,
        program: {
          ...services.program,
          getCompilerOptions: () => ({ ...services.program.getCompilerOptions(), noResolve: true }),
          getTypeChecker: () => services.program.getTypeChecker(),
        },
      }),
      (services) => ({
        ...services,
        program: {
          ...services.program,
          getCompilerOptions: () => ({
            ...services.program.getCompilerOptions(),
            strict: false,
            strictNullChecks: false,
          }),
          getTypeChecker: () => services.program.getTypeChecker(),
        },
      }),
      (services) => ({
        ...services,
        esTreeNodeToTSNodeMap: { get: () => undefined },
      }),
    ];
    for (const transform of variants) {
      const findings = languageOptionsFindings({
        code,
        rule,
        filename,
        languageOptions: {
          ecmaVersion: 2024,
          sourceType: "module",
          parser: makeParser(transform),
          parserOptions: { projectService: true, tsconfigRootDir: dirname(filename) },
        },
      });
      expect(findings).toEqual([]);
    }
  });

  it("runs untyped shapes cleanly through Oxlint", () => {
    const result = spawnSync(
      process.execPath,
      [
        oxlintBin,
        "--config",
        resolve(fixtureDir, ".oxlintrc.json"),
        "--format",
        "json",
        resolve(fixtureDir, "same-value.test.ts"),
        resolve(fixtureDir, "throwing-query.test.ts"),
        resolve(fixtureDir, "defined-then-deref.test.ts"),
        resolve(fixtureDir, "suppressed-file.test.ts"),
      ],
      { cwd: fixtureDir, encoding: "utf8", timeout: 10_000 },
    );
    expect(result.error, result.stderr).toBeUndefined();
    expect(result.status, result.stderr).toBe(1);
    const diagnostics = oxlintFindings(
      JSON.parse(result.stdout).diagnostics.filter((item) => item.code?.includes(rule)),
    );
    const countFor = (name) => diagnostics.filter((item) => item.file.endsWith(name)).length;
    expect(countFor("same-value.test.ts")).toBe(13);
    expect(countFor("throwing-query.test.ts")).toBe(27);
    expect(countFor("defined-then-deref.test.ts")).toBe(8);
    expect(countFor("suppressed-file.test.ts")).toBe(0);
  });

  it("exports an opt-in fixable rule with per-code severity schema", () => {
    expect(plugin.rules[rule]).toBeDefined();
    expect(plugin.rules[rule].meta.fixable).toBe("code");
    expect(plugin.configs.recommended.rules[`no-mistakes/${rule}`]).toBeUndefined();
    expect(plugin.configs.strict.rules[`no-mistakes/${rule}`]).toBeUndefined();
    expect(Object.keys(plugin.rules[rule].meta.schema[0].properties.severities.properties)).toEqual(
      [
        "defined-by-type",
        "same-value",
        "truthy-by-type",
        "throwing-query",
        "discarded-locator",
        "defined-then-deref",
      ],
    );
    expect(plugin.rules[rule].meta.schema[0].properties.report.enum).toEqual([
      "all",
      "error",
      "advisory",
    ]);
  });
});
