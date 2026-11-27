import assert from "node:assert/strict";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { pluginFindings, snapshotCases, snapshotDocument } from "./diagnostic-snapshot.mjs";
import { __dirname, fixture } from "./helpers.mjs";

function caseFor(rule, file) {
  const code = fixture(`${rule}/${file}`);
  return {
    name: `${rule} ${file}`,
    file,
    rule,
    findings: pluginFindings(code, rule, undefined, file),
  };
}

describe("untraceable Playwright bindings and attributes", () => {
  const cases = [
    caseFor("playwright-no-browser-new-page", "untraceable-import-equals.spec.ts"),
    caseFor("playwright-no-browser-new-page", "untraceable-reexport.spec.ts"),
    caseFor("playwright-no-browser-new-page", "untraceable-suppressed.spec.ts"),
    caseFor("playwright-consistent-attribute", "untraceable.tsx"),
    caseFor("playwright-consistent-attribute", "untraceable-suppressed.tsx"),
    caseFor("playwright-defaults", "untraceable.tsx"),
    caseFor("playwright-defaults", "untraceable-suppressed.tsx"),
  ];

  function ids(name) {
    return cases
      .find((item) => item.name === name)
      .findings.map((finding) => [finding.line, finding.messageId]);
  }

  it("reports untraceable test bindings and hidden attributes", () => {
    assert.deepEqual(ids("playwright-no-browser-new-page untraceable-import-equals.spec.ts"), [
      [3, "unanalyzable"],
    ]);
    assert.deepEqual(ids("playwright-no-browser-new-page untraceable-reexport.spec.ts"), [
      [11, "unanalyzable"],
      [15, "unanalyzable"],
      [23, "unanalyzable"],
      [25, "unanalyzable"],
    ]);
    assert.deepEqual(ids("playwright-no-browser-new-page untraceable-suppressed.spec.ts"), [
      [8, "unanalyzable"],
    ]);
    const direct = pluginFindings(
      fixture("playwright-no-browser-new-page/direct-and-aliases.spec.ts"),
      "playwright-no-browser-new-page",
      undefined,
      "direct-and-aliases.spec.ts",
    );
    assert.ok(direct.some((finding) => finding.messageId === "noBrowserNewPage"));
    assert.deepEqual(ids("playwright-consistent-attribute untraceable.tsx"), [
      [4, "unanalyzable"],
      [5, "unanalyzable"],
      [6, "attribute"],
      [8, "unanalyzable"],
    ]);
    assert.deepEqual(ids("playwright-consistent-attribute untraceable-suppressed.tsx"), [
      [6, "unanalyzable"],
    ]);
    assert.deepEqual(ids("playwright-defaults untraceable.tsx"), [
      [6, "unanalyzable"],
      [9, "unanalyzable"],
    ]);
    assert.deepEqual(ids("playwright-defaults untraceable-suppressed.tsx"), [[8, "unanalyzable"]]);
  });

  it("snapshots the untraceable diagnostics", async () => {
    await expect(snapshotDocument(snapshotCases(cases))).toMatchFileSnapshot(
      resolve(__dirname, "snapshots/playwright-untraceable-bindings.snap"),
    );
  });
});
