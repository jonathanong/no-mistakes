import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { fixture, messages } from "./helpers.mjs";

const snippet = (name) => fixture(name);

describe("test and react same-file recall", () => {
  it("reports vitest sequential through aliases and static keys", () => {
    assert.equal(
      messages(
        snippet("no-vitest-sequential/recall.test.ts"),
        "no-vitest-sequential",
        undefined,
        "recall.test.ts",
      ).length,
      14,
    );
    for (const name of [
      "renamed-fixture.test.ts",
      "default-import.test.ts",
      "module-function.test.ts",
      "module-let.test.ts",
      "module-var.test.ts",
    ]) {
      assert.deepEqual(
        messages(snippet(`no-vitest-sequential/${name}`), "no-vitest-sequential", undefined, name),
        ["sequential"],
      );
    }
  });

  it("treats an expression-free template require as a test import", () => {
    assert.deepEqual(
      messages(
        snippet("no-import-only-test-files/recall.test.js"),
        "no-import-only-test-files",
        undefined,
        "recall.test.js",
      ),
      ["aggregate"],
    );
    assert.deepEqual(
      messages(
        snippet("no-import-only-test-files/dynamic.test.js"),
        "no-import-only-test-files",
        undefined,
        "dynamic.test.js",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        snippet("no-import-only-test-files/suppressed.test.js"),
        "no-import-only-test-files",
        undefined,
        "suppressed.test.js",
      ),
      [],
    );
  });

  it("recognizes vitest namespace module mocks and ignores the default export", () => {
    assert.deepEqual(
      messages(
        snippet("vitest-mock-test-file-naming/recall.test.ts"),
        "vitest-mock-test-file-naming",
        undefined,
        "recall.test.ts",
      ),
      ["needsMock"],
    );
    assert.deepEqual(
      messages(
        snippet("vitest-mock-test-file-naming/default.mock.test.ts"),
        "vitest-mock-test-file-naming",
        undefined,
        "default.mock.test.ts",
      ),
      ["unnecessaryMock"],
    );
  });

  it("reports error message assertions through expect aliases and static keys", () => {
    assert.equal(
      messages(
        snippet("test-no-error-message-matching/recall.test.ts"),
        "test-no-error-message-matching",
        undefined,
        "recall.test.ts",
      ).length,
      14,
    );
    for (const name of [
      "playwright-expect.test.ts",
      "setup-expect.test.ts",
      "bun-expect.test.ts",
      "chai-expect.test.ts",
      "cjs-expect.test.ts",
      "local-expect.test.ts",
      "function-expect.test.ts",
      "let-expect.test.ts",
    ]) {
      assert.deepEqual(
        messages(
          snippet(`test-no-error-message-matching/${name}`),
          "test-no-error-message-matching",
          undefined,
          name,
        ),
        ["message"],
      );
    }
  });

  it("reports shared state inside aliased and namespace test calls", () => {
    assert.equal(
      messages(
        snippet("test-no-shared-state/recall.test.ts"),
        "test-no-shared-state",
        undefined,
        "recall.test.ts",
      ).length,
      5,
    );
    assert.equal(
      messages(
        snippet("test-no-shared-state/computed-describe.test.ts"),
        "test-no-shared-state",
        undefined,
        "computed-describe.test.ts",
      ).length,
      1,
    );
    assert.equal(
      messages(
        snippet("test-no-shared-state/alias-describe.test.ts"),
        "test-no-shared-state",
        undefined,
        "alias-describe.test.ts",
      ).length,
      1,
    );
    assert.equal(
      messages(
        snippet("test-no-shared-state/sequence-describe.test.ts"),
        "test-no-shared-state",
        undefined,
        "sequence-describe.test.ts",
      ).length,
      1,
    );
    assert.equal(
      messages(
        snippet("test-no-shared-state/shadow-namespace.test.ts"),
        "test-no-shared-state",
        undefined,
        "shadow-namespace.test.ts",
      ).length,
      2,
    );
    assert.deepEqual(
      messages(
        snippet("test-no-shared-state/serial-alias.test.ts"),
        "test-no-shared-state",
        undefined,
        "serial-alias.test.ts",
      ),
      [],
    );
  });

  it("reports delayed rejects through renamed expect and static keys", () => {
    assert.equal(
      messages(
        snippet("test-no-delayed-rejects/recall.ts"),
        "test-no-delayed-rejects",
        undefined,
        "recall.ts",
      ).length,
      3,
    );
  });
});
