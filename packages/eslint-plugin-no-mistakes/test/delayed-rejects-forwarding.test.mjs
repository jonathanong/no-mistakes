import assert from "node:assert/strict";
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

const id = "test-no-delayed-rejects";
const messageRule = "test-no-error-message-matching";

function delayedCase(name, fixtureName) {
  const file = `${fixtureName}.ts`;
  return {
    name,
    file,
    rule: id,
    findings: pluginFindings(fixture(`${id}/${file}`), id, undefined, file),
  };
}

function oxlintDiagnostics(relativeFile) {
  const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture");
  const result = spawnSync(
    process.execPath,
    [
      resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
      "--config",
      resolve(root, id, ".oxlintrc-forwarding.json"),
      "--format",
      "json",
      resolve(root, relativeFile),
    ],
    { encoding: "utf8" },
  );
  let parsed;
  try {
    parsed = JSON.parse(result.stdout);
  } catch (error) {
    throw new Error(`${relativeFile} oxlint status ${result.status}: ${result.stderr}\n${error}`);
  }
  return { diagnostics: parsed.diagnostics, status: result.status, stderr: result.stderr };
}

it("snapshots delayed-rejects-forwarding diagnostics", async () => {
  const cases = [
    delayedCase("reads the promise argument after call/apply forwarding", "forwarding"),
    delayedCase(
      "preserves malformed calls, poll, unknown arguments, shadowing and early observers",
      "forwarding-controls",
    ),
    delayedCase(
      "honors file, line and next-line suppression: forwarding-suppressed",
      "forwarding-suppressed",
    ),
    delayedCase(
      "honors file, line and next-line suppression: forwarding-suppressed-file",
      "forwarding-suppressed-file",
    ),
    delayedCase(
      "requires the actual function receiver before counting nested early observers",
      "forwarding-receiver",
    ),
    {
      name: "shares proven nested aliases with error-message assertions, preserving shadowing",
      file: "nested-alias.ts",
      rule: messageRule,
      findings: pluginFindings(
        fixture(`${messageRule}/nested-alias.ts`),
        messageRule,
        undefined,
        "nested-alias.ts",
      ),
    },
  ];
  const oxlintFiles = [
    [`${id}/forwarding.ts`, "forwarding.ts"],
    [`${id}/forwarding-controls.ts`, "forwarding-controls.ts"],
    [`${id}/forwarding-suppressed.ts`, "forwarding-suppressed.ts"],
    [`${id}/forwarding-suppressed-file.ts`, "forwarding-suppressed-file.ts"],
    [`${id}/forwarding-receiver.ts`, "forwarding-receiver.ts"],
    [`${messageRule}/nested-alias.ts`, "nested-alias.ts"],
  ];
  for (const [file, eslintFile] of oxlintFiles) {
    const oxlint = oxlintDiagnostics(file);
    const eslint = cases.find((item) => item.file === eslintFile);
    assert.equal(oxlint.status, eslint.findings.length ? 1 : 0, oxlint.stderr);
    assert.equal(oxlint.diagnostics.length, eslint.findings.length, file);
    cases.push({
      name: `matches Oxlint for forwarded observers, controls and suppressions: ${file}`,
      file,
      rule: file.startsWith(`${messageRule}/`) ? messageRule : id,
      findings: oxlintFindings(oxlint.diagnostics, file),
    });
  }
  await expect(snapshotDocument(snapshotCases(cases))).toMatchFileSnapshot(
    "snapshots/delayed-rejects-forwarding.snap",
  );
});
