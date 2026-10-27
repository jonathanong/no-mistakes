import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import parser from "@typescript-eslint/parser";
import { expect, it } from "vitest";
import {
  oxlintFindings,
  pluginFindings,
  snapshotCases,
  snapshotDocument,
  snapshotSuite,
} from "./diagnostic-snapshot.mjs";
import { __dirname, fixture } from "./helpers.mjs";

const directory = "test-no-shared-state";
const ruleName = "test-no-shared-state";
const cases = [
  "forwarded",
  "forwarded-controls",
  "forwarded-invalid-intrinsics",
  "forwarded-shadowed-callbacks",
  "forwarded-suppressed",
  "forwarded-file",
  "forwarded-shadowed-serial",
  "forwarded-serial",
];
const code = (name) => fixture(`${directory}/${name}.test.ts`);
const lines = (name) =>
  code(name)
    .split("\n")
    .flatMap((line, index) => (line.includes("// report") ? [index + 1] : []));

const suite = {
  valid: cases
    .filter((name) => lines(name).length === 0)
    .map((name) => ({ code: code(name), filename: `${name}.test.ts` })),
  invalid: cases
    .filter((name) => lines(name).length > 0)
    .map((name) => ({
      code: code(name),
      filename: `${name}.test.ts`,
    })),
};

function lineList(findings) {
  return findings.map((finding) => finding.line).sort((left, right) => left - right);
}

it("snapshots test-no-shared-state diagnostics", async () => {
  const pluginLoader = snapshotCases(
    cases.map((name) => ({
      file: `${name}.test.ts`,
      findings: pluginFindings(code(name), ruleName, undefined, `${name}.test.ts`),
      name,
    })),
  );
  const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", directory);
  const oxlint = cases.map((name) => {
    const result = spawnSync(
      process.execPath,
      [
        resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
        "--config",
        resolve(root, ".oxlintrc-forwarded.json"),
        "--format",
        "json",
        resolve(root, `${name}.test.ts`),
      ],
      { encoding: "utf8" },
    );
    assert.equal(result.status, lines(name).length ? 1 : 0, result.stderr);
    const diagnostics = JSON.parse(result.stdout).diagnostics.filter((finding) =>
      finding.code.includes(ruleName),
    );
    const findings = oxlintFindings(diagnostics, `${name}.test.ts`);
    const loader = pluginLoader.find((item) => item.name === name);
    assert.deepEqual(lineList(findings), lineList(loader.findings));
    return { file: `${name}.test.ts`, findings, name, status: result.status };
  });
  await expect(
    snapshotDocument({
      oxlint,
      pluginLoader,
      ruleTester: snapshotSuite(ruleName, { parser }, suite),
    }),
  ).toMatchFileSnapshot("snapshots/shared-state-forwarding.snap");
});
