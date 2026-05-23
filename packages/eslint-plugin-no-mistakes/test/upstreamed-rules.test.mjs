import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { __dirname, messages } from "./helpers.mjs";

function fixture(name) {
  return readFileSync(
    resolve(__dirname, "../../../fixtures/eslint-plugin/upstreamed-generic", name),
    "utf8",
  );
}

describe("upstreamed generic rules", () => {
  it("passes clean generic examples", () => {
    const code = fixture("valid.tsx");
    const checks = [
      ["await-array-methods", "valid.tsx"],
      ["no-delete-property", "valid.tsx"],
      ["no-placeholder-never-type-exports", "valid.tsx"],
      ["test-no-shared-state", "valid.tsx"],
      ["test-no-error-message-matching", "valid.tsx"],
      ["no-vitest-sequential", "valid.tsx"],
      ["react-no-use-promise-resolve", "valid.tsx"],
      ["react-no-iife-in-jsx", "valid.tsx"],
      ["nextjs-no-manual-script-tags", "valid.tsx"],
      ["nextjs-metadata-exports-location", "app/page.tsx"],
    ];

    for (const [rule, filename] of checks) {
      assert.deepEqual(messages(code, rule, undefined, filename), [], rule);
    }
  });

  it("reports generic invalid examples", () => {
    const code = fixture("invalid.tsx");
    const expected = [
      ["await-array-methods", ["awaited"]],
      ["no-delete-property", ["delete"]],
      ["no-placeholder-never-type-exports", ["placeholder"]],
      ["test-no-shared-state", ["shared"]],
      ["test-no-error-message-matching", ["message", "message", "message"]],
      ["no-vitest-sequential", ["sequential"]],
      ["react-no-use-promise-resolve", ["resolve"]],
      ["react-no-iife-in-jsx", ["iife"]],
      ["nextjs-no-manual-script-tags", ["script"]],
    ];

    for (const [rule, ids] of expected) {
      assert.deepEqual(messages(code, rule, undefined, "invalid.tsx"), ids, rule);
    }
  });

  it("reports import-only test aggregators", () => {
    assert.deepEqual(
      messages(
        fixture("import-only.invalid.test.ts"),
        "no-import-only-test-files",
        undefined,
        "import-only.invalid.test.ts",
      ),
      ["aggregate"],
    );
    assert.deepEqual(
      messages(
        fixture("import-only.valid.test.ts"),
        "no-import-only-test-files",
        undefined,
        "import-only.valid.test.ts",
      ),
      [],
    );
  });

  it("reports mock test filename mismatches", () => {
    assert.deepEqual(
      messages(
        fixture("mock-name.invalid.test.ts"),
        "vitest-mock-test-file-naming",
        undefined,
        "mock-name.invalid.test.ts",
      ),
      ["needsMock"],
    );
    assert.deepEqual(
      messages(
        fixture("mock-name.invalid.mock.test.ts"),
        "vitest-mock-test-file-naming",
        undefined,
        "mock-name.invalid.mock.test.ts",
      ),
      ["unnecessaryMock"],
    );
    assert.deepEqual(
      messages(
        fixture("mock-name.valid.mock.test.ts"),
        "vitest-mock-test-file-naming",
        undefined,
        "mock-name.valid.mock.test.ts",
      ),
      [],
    );
  });

  it("reports Playwright policy violations", () => {
    const code = fixture("playwright.invalid.ts");
    assert.deepEqual(messages(code, "playwright-assertion-timeout-cap", undefined, "e2e.ts"), [
      "timeout",
    ]);
    assert.deepEqual(messages(code, "playwright-selector-priority", undefined, "e2e.ts"), [
      "semantic",
      "heading",
      "text",
    ]);
    assert.deepEqual(messages(code, "playwright-no-set-timeout", undefined, "e2e.ts"), ["timeout"]);

    const valid = fixture("playwright.valid.ts");
    assert.deepEqual(messages(valid, "playwright-assertion-timeout-cap", undefined, "e2e.ts"), []);
    assert.deepEqual(messages(valid, "playwright-selector-priority", undefined, "e2e.ts"), []);
  });

  it("reports misplaced Next.js metadata exports", () => {
    assert.deepEqual(
      messages(
        fixture("metadata.invalid.ts"),
        "nextjs-metadata-exports-location",
        undefined,
        "lib/metadata.ts",
      ),
      ["location", "location"],
    );
    assert.deepEqual(
      messages(
        fixture("metadata.valid-page.tsx"),
        "nextjs-metadata-exports-location",
        undefined,
        "app/page.tsx",
      ),
      [],
    );
  });

  it("covers additional branches in upstreamed rules", () => {
    const code = fixture("coverage.tsx");
    assert.deepEqual(messages(code, "await-array-methods", undefined, "coverage.tsx"), []);
    assert.deepEqual(messages(code, "no-delete-property", undefined, "coverage.tsx"), [
      "delete",
      "delete",
    ]);
    assert.deepEqual(
      messages(code, "nextjs-metadata-exports-location", undefined, "components/meta.tsx"),
      ["location"],
    );
    assert.deepEqual(messages(code, "test-no-error-message-matching", undefined, "coverage.tsx"), [
      "message",
      "message",
      "message",
    ]);
    assert.deepEqual(messages(code, "test-no-shared-state", undefined, "coverage.tsx"), ["shared"]);
    assert.deepEqual(messages(code, "no-vitest-sequential", undefined, "coverage.tsx"), []);
    assert.deepEqual(messages(code, "playwright-selector-priority", undefined, "coverage.tsx"), [
      "semantic",
    ]);
    assert.deepEqual(
      messages(code, "playwright-assertion-timeout-cap", undefined, "coverage.tsx"),
      ["timeout", "timeout", "timeout"],
    );
    assert.deepEqual(
      messages(code, "playwright-assertion-timeout-cap", { max: 20000 }, "coverage.tsx"),
      ["timeout"],
    );
  });
});
