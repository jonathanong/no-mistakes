import { resolve } from "node:path";
import { expect, it } from "vitest";
import { __dirname, fixture, runOxlint } from "./helpers.mjs";
import { pluginFindings, snapshotCases, snapshotDocument } from "./diagnostic-snapshot.mjs";

const rule = "no-locale-less-date-format";
const ruleId = `no-mistakes/${rule}`;
const constructors = ["Intl.DateTimeFormat", "Intl.NumberFormat"];
const cases = [
  {
    name: "configured Intl constructors require a locale in calls and new expressions",
    file: "intl-constructors.mjs",
    options: { methods: [], constructors },
    lines: [1, 2, 3, 4],
    oxlint: true,
  },
  {
    name: "Intl format constructors used only for immediate resolvedOptions are exempt while stored or passed instances are reported",
    file: "intl-resolved-options.mjs",
    options: { methods: [], constructors },
    lines: [6, 8, 10, 11],
    oxlint: true,
  },
  {
    name: "Intl constructors are ignored by default",
    file: "intl-constructors.mjs",
    lines: [],
  },
  {
    name: "only configured Intl constructors are checked",
    file: "intl-constructors.mjs",
    options: { constructors: ["Intl.NumberFormat"] },
    lines: [2, 4],
  },
  {
    name: "allowUndefinedLocale permits undefined arguments but still rejects omitted arguments in Intl constructors",
    file: "intl-constructors.mjs",
    options: { constructors, allowUndefinedLocale: true },
    lines: [1, 4],
  },
  {
    name: "shadowed Intl constructors are ignored",
    file: "shadowed-intl.mjs",
    options: { methods: [], constructors },
    lines: [],
    oxlint: true,
  },
  {
    name: "default parameter global undefined is not shadowed by body var",
    file: "default-parameter.mjs",
    lines: [1],
    oxlint: true,
  },
  {
    name: "with and direct eval leave undefined references unknown",
    file: "dynamic-scope.js",
    sourceType: "script",
    lines: [],
    oxlint: true,
  },
  {
    name: "empty methods disables default method matching",
    file: "missing-locale.mjs",
    options: { methods: [] },
    lines: [],
  },
  {
    name: "matching methods include Number custom objects and unknown receivers",
    file: "matching-receivers.mjs",
    lines: [1, 2, 14, 15, 18],
  },
  {
    name: "void expressions are rejected while aliases and local bindings stay unknown",
    file: "known-undefined.mjs",
    lines: [1],
  },
  {
    name: "allowUndefinedLocale permits void expressions",
    file: "known-undefined.mjs",
    options: { allowUndefinedLocale: true },
    lines: [],
  },
  {
    name: "dynamic computed keys and first argument spreads stay unknown",
    file: "unknown-locale.mjs",
    lines: [],
  },
  {
    name: "TypeScript wrappers preserve method and locale classification",
    file: "wrapped.ts",
    lines: [1, 2, 3],
  },
  {
    name: "missing locale across default methods and optional or computed calls",
    file: "missing-locale.mjs",
    lines: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
    oxlint: true,
  },
  {
    name: "global undefined is rejected by default",
    file: "undefined-locale.mjs",
    lines: [1, 2, 3, 4, 5, 6],
    oxlint: true,
  },
  {
    name: "explicit false rejects global undefined",
    file: "undefined-locale.mjs",
    options: { allowUndefinedLocale: false },
    lines: [1, 2, 3, 4, 5, 6],
  },
  {
    name: "explicit locales arrays and direct helper functions are allowed",
    file: "explicit-locale.mjs",
    lines: [],
  },
  {
    name: "shadowed undefined is an explicit locale",
    file: "shadowed-undefined.mjs",
    lines: [],
  },
  {
    name: "allowUndefinedLocale permits global undefined",
    file: "undefined-locale.mjs",
    options: { allowUndefinedLocale: true },
    lines: [],
  },
  {
    name: "allowUndefinedLocale still rejects missing arguments",
    file: "missing-locale.mjs",
    options: { allowUndefinedLocale: true },
    lines: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
  },
  {
    name: "custom methods replace defaults",
    file: "custom-methods.mjs",
    options: { methods: ["formatDate"] },
    lines: [1, 2],
  },
  {
    name: "custom methods respect allowUndefinedLocale",
    file: "custom-methods.mjs",
    options: { methods: ["formatDate"], allowUndefinedLocale: true },
    lines: [1],
  },
  {
    name: "no-mistakes line and next-line suppression directives suppress findings",
    file: "suppressions.mjs",
    lines: [5],
    oxlint: true,
  },
  {
    name: "no-mistakes file suppression directive suppresses the entire file",
    file: "suppression-file.mjs",
    lines: [],
    oxlint: true,
  },
  {
    name: "unknownValues finding reports spread first arguments on matched methods",
    file: "unknown-locale.mjs",
    options: { unknownValues: "finding" },
    lines: [4, 5],
  },
  {
    name: "unknownValues finding reports undefined in dynamic with and eval scopes",
    file: "dynamic-scope.js",
    sourceType: "script",
    options: { unknownValues: "finding" },
    lines: [2, 6],
  },
  {
    name: "dynamic with scope leaves configured constructor namespace unknown by default",
    file: "dynamic-scope-intl.js",
    sourceType: "script",
    options: { methods: [], constructors: ["Intl.DateTimeFormat"] },
    lines: [],
  },
  {
    name: "unknownValues finding reports configured constructor in dynamic with scope",
    file: "dynamic-scope-intl.js",
    sourceType: "script",
    options: { methods: [], constructors: ["Intl.DateTimeFormat"], unknownValues: "finding" },
    lines: [2],
  },
  {
    name: "dynamic constructor spreads and computed keys stay unknown by default",
    file: "intl-unknown.mjs",
    options: { methods: [], constructors },
    lines: [],
  },
  {
    name: "unknownValues finding reports dynamic constructor spreads and computed keys while respecting shadowing and resolvedOptions",
    file: "intl-unknown.mjs",
    options: { methods: [], constructors, unknownValues: "finding" },
    lines: [2, 3, 4],
    oxlint: true,
  },
  {
    name: "unknownValues finding reports dynamic computed Intl constructor alongside missing locales",
    file: "intl-constructors.mjs",
    options: { methods: [], constructors, unknownValues: "finding" },
    lines: [1, 2, 3, 4, 9],
  },
];

it("snapshots configured locale checks and scope boundaries", async () => {
  const results = cases.map((item) => {
    const file = `${rule}/${item.file}`;
    const findings = pluginFindings(fixture(file), rule, item.options, file, {}, item.sourceType);
    expect(
      findings.map((finding) => finding.ruleId),
      item.name,
    ).toEqual(item.lines.map(() => ruleId));
    expect(
      findings.map((finding) => finding.line),
      item.name,
    ).toEqual(item.lines);
    return { ...item, file, findings, rule };
  });
  await expect(snapshotDocument(snapshotCases(results))).toMatchFileSnapshot(
    "snapshots/no-locale-less-date-format.snap",
  );
});

it("keeps real Oxlint and ESLint scope diagnostics identical", () => {
  const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", rule);
  const oxlintCases = cases.filter((item) => item.oxlint);
  const result = runOxlint({
    config: resolve(root, ".oxlintrc.json"),
    files: oxlintCases.map(({ file }) => resolve(root, file)),
  });
  expect(result.status, result.stderr).toBe(1);
  for (const { file, options, lines: expectedLines, sourceType } of oxlintCases) {
    const diagnostics = result.forFile(resolve(root, file));
    expect(
      diagnostics.map((finding) => finding.code),
      file,
    ).toEqual(expectedLines.map(() => `no-mistakes(${rule})`));
    const lines = diagnostics
      .map((finding) => finding.labels[0].span.line)
      .sort((left, right) => left - right);
    expect(lines, file).toEqual(expectedLines);
    const findings = pluginFindings(
      fixture(`${rule}/${file}`),
      rule,
      options,
      `${rule}/${file}`,
      {},
      sourceType,
    );
    expect(lines, file).toEqual(findings.map((finding) => finding.line));
    expect(
      findings.map((finding) => finding.ruleId),
      file,
    ).toEqual(expectedLines.map(() => ruleId));
  }
});

it("rejects blank methods, non-dotted constructors, and invalid unknownValues", () => {
  const source = fixture(`${rule}/missing-locale.mjs`);
  for (const [invalidOption, pattern] of [
    [{ methods: [""] }, /should match pattern/],
    [{ methods: ["   "] }, /should match pattern/],
    [{ constructors: [""] }, /should match pattern/],
    [{ constructors: ["DateTimeFormat"] }, /should match pattern/],
    [{ constructors: ["Intl.DateTimeFormat.extra"] }, /should match pattern/],
    [{ unknownValues: "warn" }, /should be equal to one of the allowed values/],
  ]) {
    expect(
      () => pluginFindings(source, rule, invalidOption, `${rule}/missing-locale.mjs`),
      JSON.stringify(invalidOption),
    ).toThrow(pattern);
  }
});
