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

const rule = "test-no-delayed-rejects";

function oxlintAliasFindings(relativeFile) {
  const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture");
  const result = spawnSync(
    process.execPath,
    [
      resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
      "--config",
      resolve(root, rule, ".oxlintrc-forwarding.json"),
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
  const diagnostics = parsed.diagnostics.filter(
    (diagnostic) => diagnostic.code === `no-mistakes(${rule})`,
  );
  return {
    findings: oxlintFindings(diagnostics, relativeFile),
    status: result.status,
    stderr: result.stderr,
  };
}

it("snapshots delayed-rejects-promise-alias diagnostics", async () => {
  const cases = [];
  for (const name of [
    "promise-alias-invalid",
    "promise-alias-controls",
    "promise-alias-suppressed",
    "promise-alias-suppressed-file",
  ]) {
    const file = `${rule}/${name}.ts`;
    const findings = pluginFindings(fixture(file), rule, undefined, file);
    const oxlint = oxlintAliasFindings(file);
    assert.equal(oxlint.status, findings.length ? 1 : 0, oxlint.stderr);
    assert.deepEqual(
      oxlint.findings.map((item) => item.line),
      findings.map((item) => item.line),
      name,
    );
    cases.push({
      name: `keeps saved ${name} behavior`,
      file,
      rule,
      findings,
    });
    cases.push({
      name: `matches Oxlint on every saved alias case: ${name}`,
      file,
      rule,
      findings: oxlint.findings,
    });
  }
  const withFile = `${rule}/promise-alias-with.cjs`;
  const withFindings = pluginFindings(fixture(withFile), rule, undefined, withFile, {}, "script");
  const withOxlint = oxlintAliasFindings(withFile);
  assert.equal(withOxlint.status, withFindings.length ? 1 : 0, withOxlint.stderr);
  assert.deepEqual(
    withOxlint.findings.map((item) => item.line),
    withFindings.map((item) => item.line),
  );
  cases.push({
    name: "does not confuse with-object getters with lexical promise aliases in script mode",
    file: withFile,
    rule,
    sourceType: "script",
    findings: withFindings,
  });
  cases.push({
    name: "does not confuse with-object getters with lexical promise aliases in script mode: oxlint",
    file: withFile,
    rule,
    findings: withOxlint.findings,
  });
  await expect(snapshotDocument(snapshotCases(cases))).toMatchFileSnapshot(
    "snapshots/delayed-rejects-promise-alias.snap",
  );
});
