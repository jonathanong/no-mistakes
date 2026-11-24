import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { expect, it } from "vitest";
import {
  languageOptionsFindings,
  oxlintFindings,
  pluginFindings,
  snapshotCases,
  snapshotDocument,
} from "./diagnostic-snapshot.mjs";
import { __dirname, fixture } from "./helpers.mjs";

const rule = "test-no-typed-enqueue-fake";
const file = "typed-fakes.test.ts";
const extendedFile = "extended-forms.test.ts";
const singleExportPattern = "^enqueue[A-Z]";
const moduleGlobs = ["./que*"];
const scoped = { singleExportPattern, moduleGlobs };

function toPreV6TypeParameters(node, seen = new Set()) {
  if (!node || typeof node !== "object" || seen.has(node)) return;
  seen.add(node);
  if ("typeArguments" in node && node.typeArguments) {
    node.typeParameters = node.typeArguments;
    delete node.typeArguments;
  }
  for (const [key, value] of Object.entries(node)) {
    if (key === "parent") continue;
    if (Array.isArray(value)) {
      for (const item of value) toPreV6TypeParameters(item, seen);
    } else {
      toPreV6TypeParameters(value, seen);
    }
  }
}

const preV6TsParser = {
  parseForESLint(code, parserOptions) {
    const parsed = tsParser.parseForESLint(code, parserOptions);
    toPreV6TypeParameters(parsed.ast);
    const visitorKeys = Object.fromEntries(
      Object.entries(parsed.visitorKeys).map(([type, keys]) => [
        type,
        keys.map((key) => (key === "typeArguments" ? "typeParameters" : key)),
      ]),
    );
    return { ...parsed, visitorKeys };
  },
};

it("snapshots typed enqueue fake diagnostics", async () => {
  const cases = [
    { name: "selects matching exports", options: { singleExportPattern }, file },
    { name: "selects matching modules", options: { moduleGlobs }, file },
    { name: "uses both selectors and verifies line suppression", options: scoped, file },
    { name: "has no implicit targets", file },
    { name: "honors file suppression", options: scoped, file: "suppressed.test.ts" },
    {
      name: "reports namespace, wrapper-type, and vi.mocked forms",
      options: scoped,
      file: extendedFile,
    },
  ].map((item) => ({
    ...item,
    rule,
    findings: pluginFindings(fixture(`${rule}/${item.file}`), rule, item.options, item.file),
  }));
  const preV6Findings = languageOptionsFindings({
    code: fixture(`${rule}/${extendedFile}`),
    rule,
    options: [scoped],
    filename: extendedFile,
    languageOptions: { ecmaVersion: 2024, sourceType: "module", parser: preV6TsParser },
  });
  cases.push({
    name: "supports pre-v6 parser typeParameters",
    options: scoped,
    file: extendedFile,
    rule,
    findings: preV6Findings,
  });
  expect(cases[2].findings.map(({ line }) => line)).toEqual([9, 10, 11, 12, 13, 14, 38]);
  expect(cases[5].findings.map(({ line }) => line)).toEqual([
    9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
  ]);
  expect(preV6Findings).toEqual(cases[5].findings);
  await expect(snapshotDocument(snapshotCases(cases))).toMatchFileSnapshot(
    "snapshots/test-no-typed-enqueue-fake.snap",
  );
});

it("identifies an invalid export pattern in the rule configuration", () => {
  const source = fixture(`${rule}/${file}`);
  expect(() => pluginFindings(source, rule, { singleExportPattern: "[" }, file)).toThrow(
    "singleExportPattern must be a valid regular expression",
  );
});

it("reports the same saved cases through Oxlint", () => {
  const repo = resolve(__dirname, "../../..");
  const root = resolve(repo, "test-cases/eslint-snippets/fixture", rule);
  const result = spawnSync(
    process.execPath,
    [
      resolve(repo, "node_modules/oxlint/bin/oxlint"),
      "--config",
      resolve(root, ".oxlintrc.json"),
      "--format",
      "json",
      resolve(root, file),
      resolve(root, extendedFile),
      resolve(root, "suppressed.test.ts"),
    ],
    { cwd: repo, encoding: "utf8", timeout: 4_000 },
  );
  expect(result.error, result.stderr).toBeUndefined();
  expect(result.status, result.stderr).toBe(1);
  const diagnostics = oxlintFindings(JSON.parse(result.stdout).diagnostics);
  const byFile = (name) =>
    diagnostics.filter((item) => item.file.endsWith(name)).map(({ code, line }) => [code, line]);
  expect(byFile(file)).toEqual(
    [9, 10, 11, 12, 13, 14, 38].map((line) => [`no-mistakes(${rule})`, line]),
  );
  expect(byFile(extendedFile)).toEqual(
    [9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20].map((line) => [`no-mistakes(${rule})`, line]),
  );
});
