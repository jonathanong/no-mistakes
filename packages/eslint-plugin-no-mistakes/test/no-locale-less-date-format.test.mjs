import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { expect, it } from "vitest";
import { __dirname, fixture } from "./helpers.mjs";
import { pluginFindings, snapshotCases, snapshotDocument } from "./diagnostic-snapshot.mjs";

const rule = "no-locale-less-date-format";
const ruleId = `no-mistakes/${rule}`;
const cases = [
  {
    name: "default parameter global undefined is not shadowed by body var",
    file: "default-parameter.mjs",
    lines: [1],
  },
  {
    name: "with and direct eval leave undefined references unknown",
    file: "dynamic-scope.js",
    sourceType: "script",
    lines: [],
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
  },
  {
    name: "global undefined is rejected by default",
    file: "undefined-locale.mjs",
    lines: [1, 2, 3, 4, 5, 6],
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
  for (const file of [
    "missing-locale.mjs",
    "undefined-locale.mjs",
    "default-parameter.mjs",
    "dynamic-scope.js",
  ]) {
    const item = cases.find((item) => item.file === file && item.options === undefined);
    const result = spawnSync(
      process.execPath,
      [
        resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
        "--config",
        resolve(root, ".oxlintrc.json"),
        "--format",
        "json",
        resolve(root, file),
      ],
      { encoding: "utf8" },
    );
    expect(result.status, `${file}: ${result.stderr}`).toBe(item.lines.length ? 1 : 0);
    const diagnostics = JSON.parse(result.stdout).diagnostics;
    expect(
      diagnostics.map((finding) => finding.code),
      file,
    ).toEqual(item.lines.map(() => `no-mistakes(${rule})`));
    const lines = diagnostics
      .map((finding) => finding.labels[0].span.line)
      .sort((left, right) => left - right);
    expect(lines, file).toEqual(item.lines);
    const findings = pluginFindings(
      fixture(`${rule}/${file}`),
      rule,
      undefined,
      `${rule}/${file}`,
      {},
      item.sourceType,
    );
    expect(lines, file).toEqual(findings.map((finding) => finding.line));
    expect(
      findings.map((finding) => finding.ruleId),
      file,
    ).toEqual(item.lines.map(() => ruleId));
  }
});
