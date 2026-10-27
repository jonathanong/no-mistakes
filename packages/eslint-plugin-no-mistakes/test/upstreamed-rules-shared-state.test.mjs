import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { pluginFindings, snapshotCases, snapshotDocument } from "./diagnostic-snapshot.mjs";
import { __dirname } from "./helpers.mjs";

const require = createRequire(import.meta.url);
const calleeHelpers = require("../src/rules/test-no-shared-state-callees");
const { createCleanupTracker } = require("../src/rules/test-no-shared-state-cleanup");

function fixture(name) {
  return readFileSync(
    resolve(__dirname, "../../../test-cases/eslint-plugin/upstreamed-generic/fixture", name),
    "utf8",
  );
}

describe("shared-state callbacks", () => {
  it("snapshots test-no-shared-state diagnostics", async () => {
    const specs = [
      ["shared-state-callback-shadow.valid.test.ts"],
      ["shared-state-shadowed-local-state.valid.test.ts"],
      ["shared-state-extend-alias.invalid.test.ts"],
      ["shared-state-exported-extend-alias.invalid.test.ts"],
      ["shared-state-shadowed-extend-alias.valid.test.ts"],
      ["shared-state-serial-beforeall.valid.test.ts", { expect: "readonly" }],
      ["shared-state-serial-named-beforeall.valid.test.ts", { expect: "readonly" }],
      ["shared-state-extend-setup.valid.test.ts"],
      ["shared-state-describe-modifier.invalid.test.ts"],
      ["shared-state-configure-serial.valid.test.ts", { expect: "readonly" }],
      ["shared-state-configure-serial-after-setup.valid.test.ts", { expect: "readonly" }],
      ["shared-state-file-configure-serial.valid.test.ts", { expect: "readonly" }],
      ["shared-state-quoted-configure-serial.valid.test.ts", { expect: "readonly" }],
      ["shared-state-non-test-configure.invalid.test.ts"],
      ["shared-state-it-alias.invalid.test.ts"],
      ["shared-state-describe-alias-scope.invalid.test.ts"],
      ["shared-state-shadowed-it-alias.valid.test.ts"],
      ["shared-state-serial-alias-name.invalid.test.ts"],
    ];
    const cases = specs.map(([file, globals]) => ({
      file,
      findings: pluginFindings(fixture(file), "test-no-shared-state", undefined, file, globals),
      name: file,
      rule: "test-no-shared-state",
    }));
    await expect(snapshotDocument(snapshotCases(cases))).toMatchFileSnapshot(
      "snapshots/upstreamed-rules-shared-state.snap",
    );
  });

  it("recognizes dynamic test callee helper shapes", () => {
    const testCallees = new Set(["test", "myTest"]);
    assert.equal(
      calleeHelpers.isKnownTestCallee(
        {
          computed: false,
          object: { name: "myTest", type: "Identifier" },
          property: { name: "describe", type: "Identifier" },
          type: "MemberExpression",
        },
        testCallees,
      ),
      true,
    );
    assert.equal(
      calleeHelpers.isKnownTestCallee(
        {
          computed: true,
          object: { name: "myTest", type: "Identifier" },
          property: { value: "describe", type: "Literal" },
          type: "MemberExpression",
        },
        testCallees,
      ),
      true,
    );
    assert.equal(
      calleeHelpers.isKnownTestCallee(
        {
          computed: true,
          object: { name: "myTest", type: "Identifier" },
          property: { name: "describe", type: "Identifier" },
          type: "MemberExpression",
        },
        testCallees,
      ),
      false,
    );
    assert.equal(
      calleeHelpers.setupCallbackKind(
        {
          callee: {
            computed: false,
            object: { name: "myTest", type: "Identifier" },
            property: { name: "afterAll", type: "Identifier" },
            type: "MemberExpression",
          },
        },
        testCallees,
      ),
      "once",
    );
  });

  it("promotes only matching pending beforeAll cleanup when a suite becomes serial", () => {
    const cleanup = createCleanupTracker();
    cleanup.enterSuite();
    cleanup.beginSetup("before-once");
    cleanup.rememberPendingBeforeAll("items");
    cleanup.endSetup();
    cleanup.exitSuite();
    cleanup.enterSuite();
    cleanup.beginSetup("before-once");
    cleanup.rememberPendingBeforeAll("other");
    cleanup.endSetup();
    cleanup.markCurrentSuiteSerial();
    assert.equal(cleanup.has("items", "0"), false);
    assert.equal(cleanup.has("other", "1"), true);
  });
});
