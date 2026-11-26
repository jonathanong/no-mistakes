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
import { __dirname, fixture } from "./helpers.mjs";

const fixtureDir = "ts-wrapper-imports";
const fixtureRoot = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", fixtureDir);

const registerNamed = {
  registrar: { source: "@example/app", imported: "register" },
  callback: "handler",
};
const registerDefault = {
  registrar: { source: "@example/app", imported: "default" },
  callback: "handler",
};
const runTarget = {
  targets: [{ sourceSpecifierPatterns: ["@example/jobs"], calleeNamePatterns: ["run"] }],
};
const importedCall = {
  targets: [
    {
      sourceSpecifierPatterns: ["@example/http"],
      calleeNamePatterns: ["default"],
      optionsPosition: 1,
      requiredProperties: ["timeout"],
    },
    {
      sourceSpecifierPatterns: ["@example/transport"],
      calleeNamePatterns: ["send", "request"],
      optionsPosition: 1,
      requiredProperties: ["agent"],
      allowedValueCallees: { agent: ["approvedAgent"] },
    },
    {
      sourceSpecifierPatterns: ["@example/agents"],
      calleeNamePatterns: ["createUnsafeAgent"],
      optionsPosition: 1,
      requiredProperties: ["agent"],
      allowedValueCallees: { agent: ["approvedAgent"] },
    },
  ],
};

const cases = [
  {
    name: "restricted-api (#4339 #4433)",
    file: "restricted-api.ts",
    rule: "restricted-api-usage",
    options: {
      targets: [
        { source: "@example/api", importedNamePattern: "^danger$" },
        { importedNamePattern: "^banned$" },
        { constructNamePattern: "^Foo$" },
        { referenceNamePattern: "^safe$" },
      ],
    },
    lines: {
      "import { banned }": ["restrictedApi"],
      "const { danger } = require": ["restrictedApi", "restrictedApi"],
      "const mod = require": ["restrictedApi"],
      "banned()": ["restrictedApi"],
      "eq.danger()": ["restrictedApi"],
      "ns.danger()": ["restrictedApi"],
      "=danger();": ["restrictedApi"],
      "mod.danger()": ["restrictedApi"],
      "ns[name]()": ["unanalyzable"],
      "danger: exported": ["restrictedApi", "restrictedApi"],
      "danger as reexported": ["restrictedApi"],
      "new (Foo": ["restrictedApi"],
      "const seen = safe": ["restrictedApi"],
      "(safe<string>)()": [],
    },
  },
  {
    name: "unbatched-reads (#4340)",
    file: "unbatched-reads.ts",
    rule: "no-consecutive-unbatched-reads",
    options: {
      imports: [
        { source: "@example/cache", imported: "db" },
        { source: "@example/cache", imported: "default" },
      ],
      readMethods: ["get"],
    },
    lines: {
      "const b =": ["unbatchedReads"],
      "const d =": ["unbatchedReads"],
      "const f =": ["unbatchedReads"],
      "const h =": ["unbatchedReads"],
      "const j =": ["unbatchedReads"],
    },
  },
  {
    name: "registration-namespace (#4341)",
    file: "registration-namespace.ts",
    rule: "require-top-level-registration",
    options: registerNamed,
    lines: {},
  },
  {
    name: "registration-equals (#4341)",
    file: "registration-equals.ts",
    rule: "require-top-level-registration",
    options: registerNamed,
    lines: {},
  },
  {
    name: "registration-default (#4341)",
    file: "registration-default.ts",
    rule: "require-top-level-registration",
    options: registerDefault,
    lines: {},
  },
  {
    name: "registration-cjs (#4341)",
    file: "registration-cjs.ts",
    rule: "require-top-level-registration",
    options: registerNamed,
    lines: {},
  },
  {
    name: "registration-cjs-default (#4341)",
    file: "registration-cjs-default.ts",
    rule: "require-top-level-registration",
    options: registerDefault,
    lines: {},
  },
  {
    name: "registration-cast (#4436)",
    file: "registration-cast.ts",
    rule: "require-top-level-registration",
    options: registerNamed,
    lines: {},
  },
  {
    name: "registration-missing (#4341)",
    file: "registration-missing.ts",
    rule: "require-top-level-registration",
    options: registerNamed,
    lines: { "export const ready": ["registration"] },
  },
  {
    name: "argv-main (#4342 #4445)",
    file: "argv-main.ts",
    rule: "no-argv-main-guard",
    lines: {
      '"destructure"': ["argvMain"],
      '"equals"': ["argvMain"],
      '"index"': ["argvMain"],
      '"at"': ["argvMain"],
      '"url"': ["argvMain"],
    },
  },
  {
    name: "banned-import (#4343 #4460)",
    file: "banned-import.ts",
    rule: "no-banned-import-outside-allowed-paths",
    options: {
      checkedPathPatterns: ["**"],
      bannedImports: [{ module: "@example/evil", names: ["bad", "secret"] }],
    },
    lines: {
      'evil["bad" as const]()': ["bannedImport"],
      "=bad();": ["bannedImport"],
      "secret: fromEquals": ["bannedReExport"],
      "secret: fromRequire": ["bannedReExport"],
      "=g();": ["bannedImport"],
      '(require as NodeRequire)("@example/evil").bad()': ["bannedImport"],
      'require("@example/evil" as const).bad()': ["bannedImport"],
    },
  },
  {
    name: "object-mutation (#4344)",
    file: "object-mutation.ts",
    rule: "no-object-mutation",
    options: { module: false, imported: true },
    lines: {
      "box.value = 1": ["mutation"],
      "eq.value = 2": ["mutation"],
      "inner.value = 3": ["mutation"],
    },
  },
  {
    name: "top-level-await (#4359)",
    file: "top-level-await.ts",
    rule: "no-top-level-await",
    lines: { "Promise.resolve(2)": ["topLevelAwait"] },
  },
  {
    name: "disposition-iife (#4360)",
    file: "disposition-iife.ts",
    rule: "async-call-disposition",
    options: runTarget,
    lines: { "(() => run())": ["disposition"] },
  },
  {
    name: "disposition-handled (#4360)",
    file: "disposition-handled.ts",
    rule: "async-call-disposition",
    options: runTarget,
    lines: {},
  },
  {
    name: "disposition-expect (#4407)",
    file: "disposition-expect.ts",
    rule: "async-call-disposition",
    options: {
      targets: [{ sourceSpecifierPatterns: ["@example/jobs"], calleeNamePatterns: ["default"] }],
    },
    lines: { "enqueue();": ["disposition"] },
  },
  {
    name: "array-methods (#4361)",
    file: "array-methods.ts",
    rule: "await-array-methods",
    lines: {
      "xs.sort() as number[]": ["awaited"],
      'xs["sort" as const]()': ["awaited"],
      "ys.sort()": ["awaited"],
    },
  },
  {
    name: "import-only-test (#4375 #4472)",
    file: "suite.test.ts",
    filename: "suite.test.ts",
    rule: "no-import-only-test-files",
    lines: { 'import a = require("./a.test")': ["aggregate"] },
  },
  {
    name: "imported-call (#4382 #4514)",
    file: "imported-call.ts",
    rule: "require-options-on-imported-call",
    options: importedCall,
    lines: {
      "fetchItem()": ["missingOptions"],
      "namedDefault()": ["missingOptions"],
      "cjsAgent()": ["invalidOptionValue"],
      "agentsEq.otherAgent()": ["invalidOptionValue"],
      createUnsafeAgent: ["invalidOptionValue"],
      'request["call" as const]': ["missingOptions"],
      "req()": ["missingOptions"],
    },
  },
  {
    name: "function-aliases (#4412 #4528)",
    file: "function-aliases.ts",
    rule: "ts-no-function-aliases",
    lines: {
      "function wrap(a)": ["alias"],
      "function wrapCall(a)": ["alias"],
      "function viaComputed(x)": ["alias"],
      "function viaApply(x)": ["alias"],
      "function viaCast(x)": ["alias"],
      "export const aliased": ["alias"],
    },
  },
  {
    name: "null-defaults (#4420)",
    file: "null-defaults.ts",
    rule: "ts-preserve-null-option-defaults",
    lines: {
      "(options as { limit?: number | null }).limit": ["default"],
      "options!.limit satisfies": ["default"],
      "(limit as number | null | undefined)": ["default"],
      "options?.limit": ["default"],
    },
  },
  {
    name: "try-catch (#4450)",
    file: "try-catch.ts",
    rule: "async-try-catch-return-await",
    options: {
      handlers: [
        { sourceSpecifierPatterns: ["@example/rate"], calleeNamePatterns: ["handleRateLimit"] },
      ],
    },
    lines: { "return new (Promise": ["awaitReturn"] },
  },
  {
    name: "gh-api (#4441)",
    file: "gh-api.ts",
    rule: "gh-api-argument-safety",
    lines: {
      'execFileSync["call" as const]': ["endpointQuery"],
      '(require as NodeRequire)("node:child_process")': ["endpointQuery"],
    },
  },
  {
    name: "global-fetch (#4464)",
    file: "global-fetch.ts",
    rule: "no-global-fetch-outside-helper",
    options: { checkedPathPatterns: ["**"] },
    lines: {
      'globalThis["fetch" as const]': ["globalFetch"],
      'fetch["call" as const]': ["globalFetch"],
      'fetchAlias("/c")': ["globalFetch"],
      'fn("/d")': ["globalFetch"],
    },
  },
  {
    name: "delete-property (#4468)",
    file: "delete-property.ts",
    rule: "no-delete-property",
    lines: {
      "delete (obj.foo satisfies number)": ["delete"],
      "delete obj.fn<number>": ["delete"],
    },
  },
  {
    name: "noop-catch (#4476)",
    file: "noop-catch.ts",
    rule: "no-inline-noop-promise-catch",
    options: { allowedCalleeNamePatterns: ["cleanup"], checkedPathPatterns: ["**"] },
    lines: { 'work()["catch" as const]': ["noopCatch"] },
  },
];

function sourceLine(source, text) {
  const exact = text.startsWith("=");
  const needle = exact ? text.slice(1) : text;
  const line = source
    .split("\n")
    .findIndex((item) => (exact ? item.trim() === needle : item.includes(needle)));
  assert.notEqual(line, -1, text);
  return line + 1;
}

function assertLines(findings, source, expected) {
  const expectedByLine = new Map();
  for (const [text, ids] of Object.entries(expected)) {
    const line = sourceLine(source, text);
    assert.equal(expectedByLine.has(line), false, `anchor shares a line: ${text}`);
    expectedByLine.set(line, [...ids].sort());
  }
  const actual = new Map();
  for (const finding of findings) {
    assert.equal(finding.fatal ?? false, false);
    const list = actual.get(finding.line) ?? [];
    list.push(finding.messageId);
    actual.set(finding.line, list);
  }
  for (const [line, ids] of expectedByLine) {
    if (ids.length === 0) {
      expect(actual.get(line) ?? [], `line ${line}`).toEqual([]);
      actual.delete(line);
      continue;
    }
    expect([...(actual.get(line) ?? [])].sort(), `line ${line}`).toEqual(ids);
    actual.delete(line);
  }
  expect(
    [...actual.keys()].sort((left, right) => left - right),
    "unexpected lines",
  ).toEqual([]);
}

describe("TypeScript wrappers and CommonJS imports", () => {
  it("snapshots ESLint and Oxlint findings for every remaining rule", async () => {
    const sources = new Map(
      cases.map((item) => [item.file, fixture(`${fixtureDir}/${item.file}`)]),
    );
    const eslintResults = cases.map((item) => {
      const findings = pluginFindings(
        sources.get(item.file),
        item.rule,
        item.options,
        item.filename ?? item.file,
      );
      assertLines(findings, sources.get(item.file), item.lines);
      return {
        name: item.name,
        file: item.file,
        rule: item.rule,
        ...(item.options ? { options: item.options } : {}),
        findings,
      };
    });

    const oxlintResult = spawnSync(
      process.execPath,
      [
        resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
        "--config",
        resolve(fixtureRoot, ".oxlintrc.json"),
        "--format",
        "json",
        ...cases.map((item) => resolve(fixtureRoot, item.file)),
      ],
      { encoding: "utf8" },
    );
    assert.equal(
      oxlintResult.stdout.includes("Error running JS plugin"),
      false,
      oxlintResult.stderr,
    );
    assert.equal(oxlintResult.status, 1, oxlintResult.stderr);
    const parsed = JSON.parse(oxlintResult.stdout).diagnostics.filter((entry) =>
      entry.code?.startsWith("no-mistakes("),
    );
    const oxlintResults = cases.map((item) => {
      const findings = oxlintFindings(
        parsed.filter(
          (entry) =>
            entry.filename?.endsWith(`/${item.file}`) && entry.code === `no-mistakes(${item.rule})`,
        ),
        item.file,
      );
      expect(
        findings.map((finding) => finding.line),
        `oxlint ${item.name}`,
      ).toEqual(
        eslintResults
          .find((result) => result.file === item.file)
          .findings.map((finding) => finding.line),
      );
      return { name: item.name, file: item.file, rule: item.rule, findings };
    });

    await expect(
      snapshotDocument({
        eslint: snapshotCases(eslintResults),
        oxlint: snapshotCases(oxlintResults),
      }),
    ).toMatchFileSnapshot("snapshots/ts-wrapper-imports.snap");
  });
});
