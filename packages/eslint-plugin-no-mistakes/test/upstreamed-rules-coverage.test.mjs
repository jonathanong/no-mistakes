import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { pluginFindings, snapshotCases, snapshotDocument } from "./diagnostic-snapshot.mjs";
import { __dirname, messages } from "./helpers.mjs";

function fixture(name) {
  return readFileSync(
    resolve(__dirname, "../../../test-cases/eslint-plugin/upstreamed-generic/fixture", name),
    "utf8",
  );
}

describe("upstreamed generic rule coverage", () => {
  it("covers additional branches in upstreamed rules", () => {
    const code = fixture("coverage.tsx");
    assert.deepEqual(messages(code, "await-array-methods", undefined, "coverage.tsx"), [
      "awaited",
      "awaited",
    ]);
    assert.deepEqual(messages(code, "no-delete-property", undefined, "coverage.tsx"), [
      "delete",
      "delete",
    ]);
    assert.deepEqual(
      messages(code, "nextjs-metadata-exports-location", undefined, "app/components/meta.tsx"),
      ["location", "location", "location"],
    );
    // test.sequential.each reports once, on the inner call.
    assert.deepEqual(messages(code, "no-vitest-sequential", undefined, "coverage.tsx"), [
      "sequential",
      "sequential",
      "sequential",
      "sequential",
      "sequential",
      "sequential",
    ]);
  });

  it("snapshots test and playwright diagnostics", async () => {
    const code = fixture("coverage.tsx");
    const timeoutCap = { max: 20000 };
    const cases = [
      {
        file: "coverage.tsx",
        findings: pluginFindings(code, "test-no-error-message-matching", undefined, "coverage.tsx"),
        name: "coverage.tsx test-no-error-message-matching",
        rule: "test-no-error-message-matching",
      },
      {
        file: "message-angle.ts",
        findings: pluginFindings(
          fixture("message-angle.ts"),
          "test-no-error-message-matching",
          undefined,
          "message-angle.ts",
        ),
        name: "message-angle.ts",
        rule: "test-no-error-message-matching",
      },
      {
        file: "coverage.tsx",
        findings: pluginFindings(code, "test-no-shared-state", undefined, "coverage.tsx"),
        name: "coverage.tsx test-no-shared-state",
        rule: "test-no-shared-state",
      },
      {
        file: "e2e.spec.tsx",
        findings: pluginFindings(code, "playwright-selector-priority", undefined, "e2e.spec.tsx"),
        name: "coverage.tsx e2e.spec.tsx playwright-selector-priority",
        rule: "playwright-selector-priority",
      },
      {
        file: "e2e.spec.tsx",
        findings: pluginFindings(
          code,
          "playwright-assertion-timeout-cap",
          undefined,
          "e2e.spec.tsx",
        ),
        name: "coverage.tsx e2e.spec.tsx playwright-assertion-timeout-cap",
        rule: "playwright-assertion-timeout-cap",
      },
      {
        file: "e2e.spec.tsx",
        findings: pluginFindings(
          code,
          "playwright-assertion-timeout-cap",
          timeoutCap,
          "e2e.spec.tsx",
        ),
        name: `coverage.tsx e2e.spec.tsx playwright-assertion-timeout-cap ${JSON.stringify(timeoutCap)}`,
        options: timeoutCap,
        rule: "playwright-assertion-timeout-cap",
      },
      {
        file: "unit.tsx",
        findings: pluginFindings(code, "playwright-assertion-timeout-cap", undefined, "unit.tsx"),
        name: "coverage.tsx unit.tsx playwright-assertion-timeout-cap",
        rule: "playwright-assertion-timeout-cap",
      },
      {
        file: "unit.tsx",
        findings: pluginFindings(code, "playwright-selector-priority", undefined, "unit.tsx"),
        name: "coverage.tsx unit.tsx playwright-selector-priority",
        rule: "playwright-selector-priority",
      },
    ];
    await expect(snapshotDocument(snapshotCases(cases))).toMatchFileSnapshot(
      "snapshots/upstreamed-rules-coverage.snap",
    );
  });
});
