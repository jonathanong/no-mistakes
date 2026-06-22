import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { __dirname, messages } from "./helpers.mjs";

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
  it("avoids resolving shadowed non-function callbacks", () => {
    assert.deepEqual(
      messages(
        fixture("shared-state-callback-shadow.valid.test.ts"),
        "test-no-shared-state",
        undefined,
        "shared-state-callback-shadow.valid.test.ts",
      ),
      [],
    );
  });

  it("respects shadowed local state in callbacks", () => {
    assert.deepEqual(
      messages(
        fixture("shared-state-shadowed-local-state.valid.test.ts"),
        "test-no-shared-state",
        undefined,
        "shared-state-shadowed-local-state.valid.test.ts",
      ),
      [],
    );
  });

  it("tracks test.extend aliases and chained test callees", () => {
    assert.deepEqual(
      messages(
        fixture("shared-state-extend-alias.invalid.test.ts"),
        "test-no-shared-state",
        undefined,
        "shared-state-extend-alias.invalid.test.ts",
      ),
      ["shared", "shared", "shared", "shared", "shared"],
    );
  });

  it("tracks exported test.extend aliases", () => {
    assert.deepEqual(
      messages(
        fixture("shared-state-exported-extend-alias.invalid.test.ts"),
        "test-no-shared-state",
        undefined,
        "shared-state-exported-extend-alias.invalid.test.ts",
      ),
      ["shared"],
    );
  });

  it("ignores shadowed test.extend aliases", () => {
    assert.deepEqual(
      messages(
        fixture("shared-state-shadowed-extend-alias.valid.test.ts"),
        "test-no-shared-state",
        undefined,
        "shared-state-shadowed-extend-alias.valid.test.ts",
      ),
      [],
    );
  });

  it("allows Playwright serial beforeAll setup assignments", () => {
    assert.deepEqual(
      messages(
        fixture("shared-state-serial-beforeall.valid.test.ts"),
        "test-no-shared-state",
        undefined,
        "shared-state-serial-beforeall.valid.test.ts",
        { expect: "readonly" },
      ),
      [],
    );
  });

  it("allows named beforeAll cleanup in serial suites", () => {
    assert.deepEqual(
      messages(
        fixture("shared-state-serial-named-beforeall.valid.test.ts"),
        "test-no-shared-state",
        undefined,
        "shared-state-serial-named-beforeall.valid.test.ts",
        { expect: "readonly" },
      ),
      [],
    );
  });

  it("tracks setup cleanup from extended test aliases", () => {
    assert.deepEqual(
      messages(
        fixture("shared-state-extend-setup.valid.test.ts"),
        "test-no-shared-state",
        undefined,
        "shared-state-extend-setup.valid.test.ts",
      ),
      [],
    );
  });

  it("keeps cleanup scoped for describe modifier suites", () => {
    assert.deepEqual(
      messages(
        fixture("shared-state-describe-modifier.invalid.test.ts"),
        "test-no-shared-state",
        undefined,
        "shared-state-describe-modifier.invalid.test.ts",
      ),
      ["shared"],
    );
  });

  it("honors Playwright configure-based serial suites", () => {
    assert.deepEqual(
      messages(
        fixture("shared-state-configure-serial.valid.test.ts"),
        "test-no-shared-state",
        undefined,
        "shared-state-configure-serial.valid.test.ts",
        { expect: "readonly" },
      ),
      [],
    );
  });

  it("honors serial configure calls after setup declarations", () => {
    assert.deepEqual(
      messages(
        fixture("shared-state-configure-serial-after-setup.valid.test.ts"),
        "test-no-shared-state",
        undefined,
        "shared-state-configure-serial-after-setup.valid.test.ts",
        { expect: "readonly" },
      ),
      [],
    );
  });

  it("honors Playwright file-level serial configure", () => {
    assert.deepEqual(
      messages(
        fixture("shared-state-file-configure-serial.valid.test.ts"),
        "test-no-shared-state",
        undefined,
        "shared-state-file-configure-serial.valid.test.ts",
        { expect: "readonly" },
      ),
      [],
    );
  });

  it("honors quoted Playwright serial configure keys", () => {
    assert.deepEqual(
      messages(
        fixture("shared-state-quoted-configure-serial.valid.test.ts"),
        "test-no-shared-state",
        undefined,
        "shared-state-quoted-configure-serial.valid.test.ts",
        { expect: "readonly" },
      ),
      [],
    );
  });

  it("ignores non-test configure calls when detecting serial suites", () => {
    assert.deepEqual(
      messages(
        fixture("shared-state-non-test-configure.invalid.test.ts"),
        "test-no-shared-state",
        undefined,
        "shared-state-non-test-configure.invalid.test.ts",
      ),
      ["shared"],
    );
  });

  it("tracks aliased it imports as test calls", () => {
    assert.deepEqual(
      messages(
        fixture("shared-state-it-alias.invalid.test.ts"),
        "test-no-shared-state",
        undefined,
        "shared-state-it-alias.invalid.test.ts",
      ),
      ["shared"],
    );
  });

  it("keeps aliased describe suites scoped", () => {
    assert.deepEqual(
      messages(
        fixture("shared-state-describe-alias-scope.invalid.test.ts"),
        "test-no-shared-state",
        undefined,
        "shared-state-describe-alias-scope.invalid.test.ts",
      ),
      ["shared"],
    );
  });

  it("ignores shadowed it aliases", () => {
    assert.deepEqual(
      messages(
        fixture("shared-state-shadowed-it-alias.valid.test.ts"),
        "test-no-shared-state",
        undefined,
        "shared-state-shadowed-it-alias.valid.test.ts",
      ),
      [],
    );
  });

  it("does not infer serial mode from the test alias name", () => {
    assert.deepEqual(
      messages(
        fixture("shared-state-serial-alias-name.invalid.test.ts"),
        "test-no-shared-state",
        undefined,
        "shared-state-serial-alias-name.invalid.test.ts",
      ),
      ["shared"],
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
