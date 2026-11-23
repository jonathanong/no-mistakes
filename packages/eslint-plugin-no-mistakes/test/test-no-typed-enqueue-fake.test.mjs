import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { expect, it } from "vitest";
import {
  oxlintFindings,
  pluginFindings,
  snapshotCases,
  snapshotDocument,
} from "./diagnostic-snapshot.mjs";
import { __dirname, fixture } from "./helpers.mjs";

const rule = "test-no-typed-enqueue-fake";
const file = "typed-fakes.test.ts";
const singleExportPattern = "^enqueue[A-Z]";
const moduleGlobs = ["./que*"];
const scoped = { singleExportPattern, moduleGlobs };

it("snapshots typed enqueue fake diagnostics", async () => {
  const cases = [
    { name: "selects matching exports", options: { singleExportPattern }, file },
    { name: "selects matching modules", options: { moduleGlobs }, file },
    { name: "uses both selectors and verifies line suppression", options: scoped, file },
    { name: "has no implicit targets", file },
    { name: "honors file suppression", options: scoped, file: "suppressed.test.ts" },
  ].map((item) => ({
    ...item,
    rule,
    findings: pluginFindings(fixture(`${rule}/${item.file}`), rule, item.options, item.file),
  }));
  expect(cases[2].findings.map(({ line }) => line)).toEqual([9, 10, 11, 12, 13, 14, 38]);
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
      resolve(root, "suppressed.test.ts"),
    ],
    { cwd: repo, encoding: "utf8", timeout: 4_000 },
  );
  expect(result.error, result.stderr).toBeUndefined();
  expect(result.status, result.stderr).toBe(1);
  const diagnostics = oxlintFindings(JSON.parse(result.stdout).diagnostics);
  expect(diagnostics.map(({ code, line }) => [code, line])).toEqual(
    [9, 10, 11, 12, 13, 14, 38].map((line) => [`no-mistakes(${rule})`, line]),
  );
});
