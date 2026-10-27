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

const RULE = "playwright-require-interactive-test-id";
const base = { interactiveComponents: ["Button"] };

function slotFindings(fixtureName, option) {
  return pluginFindings(fixture(`${RULE}/${fixtureName}.tsx`), RULE, option, "slot.tsx");
}

function slotCase(name, fixtureName, option) {
  return {
    name,
    file: "slot.tsx",
    rule: RULE,
    options: option,
    findings: slotFindings(fixtureName, option),
  };
}

it("snapshots require-interactive-test-id-slots diagnostics", async () => {
  const custom = {
    interactiveComponents: ["Button"],
    slotProps: ["renderChild"],
    selectorAttributes: ["data-qa"],
  };
  const dataQa = { interactiveComponents: ["Button"], selectorAttributes: ["data-qa"] };
  const disabled = { interactiveComponents: ["Button"], slotProps: [] };
  const native = { interactiveComponents: ["button", "my-button", "Native-Button"] };
  const anyPattern = { interactiveComponents: ["/.*/"] };
  const cases = [
    slotCase(
      "covers the single direct child through literal parent selectors",
      "slots-valid",
      base,
    ),
    slotCase(
      "reports absent or nonliteral slot selectors only on the parent",
      "slots-missing",
      base,
    ),
    slotCase(
      "retains checks for non-slots, ambiguous children and overrides",
      "slots-controls",
      base,
    ),
    slotCase(
      "replaces the default slot prop and respects custom selector attributes: renderChild",
      "slots-custom",
      custom,
    ),
    slotCase(
      "replaces the default slot prop and respects custom selector attributes: data-qa",
      "slots-custom",
      dataQa,
    ),
    slotCase("can disable slot recognition", "slots-valid", disabled),
    slotCase(
      "keeps a suppressed parent finding suppressed through the child",
      "slots-suppressed",
      base,
    ),
    slotCase("does not inherit through a shadow of a configured import", "slots-shadowed", base),
    slotCase(
      "does not treat configured intrinsic tags as slot components: native names",
      "slots-native",
      native,
    ),
    slotCase(
      "does not treat configured intrinsic tags as slot components: any pattern",
      "slots-native",
      anyPattern,
    ),
  ];

  const root = resolve(__dirname, `../../../test-cases/eslint-snippets/fixture/${RULE}`);
  const entries = [
    ["slots-valid", ".oxlintrc-slots.json", base],
    ["slots-missing", ".oxlintrc-slots.json", base],
    ["slots-controls", ".oxlintrc-slots.json", base],
    ["slots-suppressed", ".oxlintrc-slots.json", base],
    ["slots-shadowed", ".oxlintrc-slots.json", base],
    ["slots-custom", ".oxlintrc-slots-custom.json", custom],
  ];
  for (const [name, config, option] of entries) {
    const result = spawnSync(
      process.execPath,
      [
        resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
        "--config",
        resolve(root, config),
        "--format",
        "json",
        resolve(root, `${name}.tsx`),
      ],
      { encoding: "utf8" },
    );
    const eslintLines = slotFindings(name, option)
      .map((item) => item.line)
      .sort((left, right) => left - right);
    assert.equal(result.status, eslintLines.length ? 1 : 0, result.stderr);
    const diagnostics = JSON.parse(result.stdout).diagnostics.filter((diagnostic) =>
      diagnostic.code.includes(RULE),
    );
    const actual = diagnostics
      .map((diagnostic) => diagnostic.labels[0].span.line)
      .sort((left, right) => left - right);
    assert.deepEqual(actual, eslintLines, name);
    const relativeFile = `${RULE}/${name}.tsx`;
    cases.push({
      name: `matches real Oxlint on saved slot fixtures and custom options: ${name}`,
      file: relativeFile,
      rule: RULE,
      findings: oxlintFindings(diagnostics, relativeFile),
    });
  }

  await expect(snapshotDocument(snapshotCases(cases))).toMatchFileSnapshot(
    "snapshots/require-interactive-test-id-slots.snap",
  );
});
